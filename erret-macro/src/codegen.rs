use crate::Enum;
use proc_macro::TokenStream;

pub fn generate_code(en: Enum) -> TokenStream {
    if en.name.is_empty() {
        return compile_error("#[derive(Error)] can only be applied to enums");
    }

    let enum_name = &en.name;

    let ig_debug = if en.impl_generics_debug.is_empty() {
        String::new()
    } else {
        format!("<{}>", en.impl_generics_debug)
    };
    let ig_plain = if en.impl_generics_plain.is_empty() {
        String::new()
    } else {
        format!("<{}>", en.impl_generics_plain)
    };
    let tg = if en.type_generics.is_empty() {
        String::new()
    } else {
        format!("<{}>", en.type_generics)
    };
    let wc = if en.where_clause.is_empty() {
        String::new()
    } else {
        format!(" {}", en.where_clause)
    };

    let mut display_arms: Vec<String> = Vec::new();
    let mut from_impls: Vec<String> = Vec::new();

    for var in &en.vars {
        let var_name = &var.name;
        let fmt_str = var.error_arg.as_deref().unwrap_or(var_name.as_str());
        let has_placeholder = fmt_str.contains('{');

        let (pattern, write_args) = build_pattern(enum_name, var, has_placeholder, fmt_str);

        if write_args.is_empty() {
            display_arms.push(format!("{} => write!(f, \"{}\"),", pattern, fmt_str));
        } else {
            display_arms.push(format!(
                "{} => write!(f, \"{}\", {}),",
                pattern, fmt_str, write_args
            ));
        }

        if let Some(from_field) = var.fields.iter().find(|f| f.has_from) {
            let ty = &from_field.ty;

            if let Some(from_field_name) = &from_field.name {
                let non_from: Vec<_> = var.fields.iter().filter(|f| !f.has_from).collect();
                if non_from.len() != 1 {
                    return compile_error(&format!(
                        "variant `{}` with #[from] must have exactly one non-from field, found {}",
                        var_name, non_from.len()
                    ));
                }
                let static_field = non_from[0];
                let static_field_name = static_field.name.as_ref().unwrap();
                from_impls.push(format!(
                    "impl{ig_plain} core::convert::From<{ty}> for {name}{tg}{wc} {{
                        fn from(err: {ty}) -> Self {{
                            {name}::{var_name} {{
                                {static_field_name}: \"{var_name}\",
                                {from_field_name}: err,
                            }}
                        }}
                    }}",
                    ig_plain = ig_plain,
                    ty = ty,
                    name = enum_name,
                    tg = tg,
                    wc = wc,
                    var_name = var_name,
                    static_field_name = static_field_name,
                    from_field_name = from_field_name
                ));
            } else {
                if var.fields.len() != 1 {
                    return compile_error(&format!(
                        "tuple variant `{}` with #[from] must have exactly one field, found {}",
                        var_name, var.fields.len()
                    ));
                }
                from_impls.push(format!(
                    "impl{ig_plain} core::convert::From<{ty}> for {name}{tg}{wc} {{
                        fn from(err: {ty}) -> Self {{
                            {name}::{var_name}(err)
                        }}
                    }}",
                    ig_plain = ig_plain,
                    ty = ty,
                    name = enum_name,
                    tg = tg,
                    wc = wc,
                    var_name = var_name
                ));
            }
        }
    }

    let display_arms_str = display_arms.join("\n");
    let from_impls_str = from_impls.join("\n");

    format!(
        "impl{ig_debug} core::fmt::Display for {name}{tg}{wc} {{
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {{
                match self {{
                    {arms}
                }}
            }}
        }}

        impl{ig_debug} core::error::Error for {name}{tg}{wc} {{}}

        {froms}",
        ig_debug = ig_debug,
        name = enum_name,
        tg = tg,
        wc = wc,
        arms = display_arms_str,
        froms = from_impls_str
    )
    .parse()
    .unwrap()
}

fn build_pattern(
    enum_name: &str,
    var: &crate::Var,
    has_placeholder: bool,
    fmt_str: &str,
) -> (String, String) {
    let var_name = &var.name;

    if var.fields.is_empty() {
        return (format!("{}::{}", enum_name, var_name), String::new());
    }

    let is_struct = var.fields[0].name.is_some();

    if !has_placeholder {
        if is_struct {
            return (format!("{}::{} {{ .. }}", enum_name, var_name), String::new());
        } else {
            return (format!("{}::{}(..)", enum_name, var_name), String::new());
        }
    }

    if is_struct {
        let mut pat = String::new();
        for (i, field) in var.fields.iter().enumerate() {
            let f_name = field.name.as_ref().unwrap();
            if i > 0 {
                pat.push_str(", ");
            }
            if fmt_uses_field(fmt_str, f_name) {
                pat.push_str(f_name);
            } else {
                pat.push_str(&format!("{}: _{}", f_name, f_name));
            }
        }
        (
            format!("{}::{} {{ {} }}", enum_name, var_name, pat),
            String::new(),
        )
    } else {
        let mut pat = String::new();
        let mut args = String::new();
        for i in 0..var.fields.len() {
            if i > 0 {
                pat.push_str(", ");
                args.push_str(", ");
            }
            pat.push_str(&format!("_v{}", i));
            args.push_str(&format!("_v{}", i));
        }
        (format!("{}::{}({})", enum_name, var_name, pat), args)
    }
}

fn fmt_uses_field(fmt: &str, name: &str) -> bool {
    let mut s = fmt;
    while let Some(pos) = s.find('{') {
        s = &s[pos + 1..];
        if s.starts_with('{') {
            s = &s[1..];
            continue;
        }
        if s.starts_with(name) {
            let rest = &s[name.len()..];
            if rest.starts_with('}') || rest.starts_with(':') {
                return true;
            }
        }
    }
    false
}

fn compile_error(msg: &str) -> TokenStream {
    format!("compile_error!({:?});", msg).parse().unwrap()
}