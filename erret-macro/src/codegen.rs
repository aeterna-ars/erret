use proc_macro::TokenStream;

use crate::Enum;

pub fn generate_code(perechislenie: Enum) -> TokenStream {
    let enum_name = &perechislenie.name;
    let mut display_arms = String::new();
    let mut from_impls = String::new();

    for var in &perechislenie.vars {
        let var_name = &var.name;
        let fmt_str = var.error_arg.as_deref().unwrap_or(var_name.as_str());

        if var.fields.is_empty() {
            display_arms.push_str(&format!(
                "{}::{} => write!(f, \"{}\"),\n",
                enum_name, var_name, fmt_str
            ));
        } else {
            let is_struct = var.fields.first().and_then(|f| f.name.as_ref()).is_some();
            if is_struct {
                let mut match_pattern = String::new();
                let mut format_args = String::new();
                for field in &var.fields {
                    let f_name = field.name.as_ref().unwrap();
                    match_pattern.push_str(&format!("{}, ", f_name));
                    format_args.push_str(&format!(", {}", f_name));
                }
                display_arms.push_str(&format!(
                    "{}::{} {{ {} }} => write!(f, \"{}\" {}),\n",
                    enum_name, var_name, match_pattern, fmt_str, format_args
                ));
            } else {
                let mut match_pattern = String::new();
                let mut format_args = String::new();
                for i in 0..var.fields.len() {
                    match_pattern.push_str(&format!("_v{}, ", i));
                    format_args.push_str(&format!(", _v{}", i));
                }
                display_arms.push_str(&format!(
                    "{}::{} ( {} ) => write!(f, \"{}\" {}),\n",
                    enum_name, var_name, match_pattern, fmt_str, format_args
                ));
            }
        }

        if let Some(from_field) = var.fields.iter().find(|f| f.has_from) {
            let ty = &from_field.ty;
            if let Some(ref from_field_name) = from_field.name {
                // Из именованной структуры {}
                if let Some(static_field) = var.fields.iter().find(|f| !f.has_from) {
                    let static_field_name = static_field.name.as_ref().unwrap();
                    let default_value = &var.name;
                    from_impls.push_str(&format!(
                        "impl std::convert::From<{ty}> for {enum_name} {{
                            fn from(err: {ty}) -> Self {{
                                {enum_name}::{var_name} {{
                                    {static_field_name}: \"{default_value}\",
                                    {from_field_name}: err,
                                }}
                            }}
                        }}\n",
                        ty = ty, enum_name = enum_name, var_name = var_name,
                        static_field_name = static_field_name, default_value = default_value,
                        from_field_name = from_field_name
                    ));
                }
            } else {
                if var.fields.len() == 1 {
                    from_impls.push_str(&format!(
                        "impl std::convert::From<{ty}> for {enum_name} {{
                            fn from(err: {ty}) -> Self {{
                                {enum_name}::{var_name}(err)
                            }}
                        }}\n",
                        ty = ty, enum_name = enum_name, var_name = var_name
                    ));
                }
            }
        }
    }

    format!(
        "
        impl std::fmt::Display for {enum_name} {{
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{
                match self {{
                    {display_arms}
                }}
            }}
        }}
        impl std::error::Error for {enum_name} {{}}
        {from_impls}
        ",
        enum_name = enum_name, display_arms = display_arms, from_impls = from_impls
    ).parse().unwrap()
}