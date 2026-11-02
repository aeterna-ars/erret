extern crate proc_macro;

mod codegen;
mod parse;

use proc_macro::*;

#[derive(Debug, Default)]
struct Enum {
    pub name: String,
    pub impl_generics_debug: String,
    pub impl_generics_plain: String,
    pub type_generics: String,
    pub where_clause: String,
    pub vars: Vec<Var>,
}

#[derive(Debug, Default, Clone)]
struct Var {
    pub name: String,
    pub fields: Vec<FieldInfo>,
    pub error_arg: Option<String>,
}

#[derive(Debug, Default, Clone)]
struct FieldInfo {
    pub name: Option<String>,
    pub ty: String,
    pub has_from: bool,
}

#[proc_macro_derive(Error, attributes(error, from))]
pub fn error(item: TokenStream) -> TokenStream {
    let mut iter = item.into_iter().peekable();
    let en = parse_enum(&mut iter);
    codegen::generate_code(en)
}

fn parse_enum(iter: &mut std::iter::Peekable<token_stream::IntoIter>) -> Enum {
    let mut en = Enum::default();
    let mut seen_enum = false;

    while let Some(token) = iter.next() {
        match token {
            TokenTree::Ident(i) => {
                let s = i.to_string();
                if s == "enum" {
                    seen_enum = true;
                } else if seen_enum && en.name.is_empty() {
                    en.name = s;
                } else if s == "where" && seen_enum {
                    // Parse where clause until '{'
                    let mut where_tokens: Vec<TokenTree> = Vec::new();
                    while let Some(t) = iter.peek() {
                        if let TokenTree::Group(g) = t {
                            if g.delimiter() == Delimiter::Brace {
                                break;
                            }
                        }
                        where_tokens.push(iter.next().unwrap());
                    }
                    let ts: TokenStream = where_tokens.into_iter().collect();
                    en.where_clause = format!("where {}", ts.to_string().trim());
                }
            }
            TokenTree::Punct(p) if p.as_char() == '<' && !en.name.is_empty() => {
                // Parse generics until matching '>'
                let mut generic_tokens: Vec<TokenTree> = Vec::new();
                let mut depth = 1u32;
                while let Some(t) = iter.next() {
                    match &t {
                        TokenTree::Punct(pp) => {
                            match pp.as_char() {
                                '<' => { depth += 1; generic_tokens.push(t); }
                                '>' => {
                                    depth -= 1;
                                    if depth == 0 { break; }
                                    generic_tokens.push(t);
                                }
                                _ => generic_tokens.push(t),
                            }
                        }
                        _ => generic_tokens.push(t),
                    }
                }
                parse_generics(&generic_tokens, &mut en);
            }
            TokenTree::Group(g) if g.delimiter() == Delimiter::Brace => {
                en.vars = parse::parse_enum_body(g.stream());
                break;
            }
            _ => {}
        }
    }

    en
}

fn parse_generics(tokens: &[TokenTree], en: &mut Enum) {
    // Split by top-level comma
    let mut params: Vec<Vec<TokenTree>> = Vec::new();
    let mut cur = Vec::new();
    let mut depth = 0i32;

    for t in tokens {
        match t {
            TokenTree::Punct(p) => {
                match p.as_char() {
                    '<' | '(' | '[' => { depth += 1; cur.push(t.clone()); }
                    '>' | ')' | ']' => { depth -= 1; cur.push(t.clone()); }
                    ',' if depth == 0 => {
                        if !cur.is_empty() {
                            params.push(std::mem::take(&mut cur));
                        }
                    }
                    _ => cur.push(t.clone()),
                }
            }
            _ => cur.push(t.clone()),
        }
    }
    if !cur.is_empty() { params.push(cur); }

    let mut debug_parts: Vec<String> = Vec::new();
    let mut plain_parts: Vec<String> = Vec::new();
    let mut type_parts: Vec<String> = Vec::new();

    for param in params {
        let ts: TokenStream = param.iter().cloned().collect();
        let s = ts.to_string();

        let idents: Vec<String> = param.iter()
            .filter_map(|t| match t {
                TokenTree::Ident(i) => Some(i.to_string()),
                _ => None,
            })
            .collect();

        // const generic: `const N: usize`
        if idents.first().map(|s| s.as_str()) == Some("const") {
            debug_parts.push(s.clone());
            plain_parts.push(s);
            if idents.len() >= 2 {
                type_parts.push(idents[1].clone());
            }
            continue;
        }

        // type parameter
        if idents.is_empty() {
            continue;
        }
        let name = idents[0].clone();

        plain_parts.push(s.clone());

        let has_colon = param.iter()
            .any(|t| matches!(t, TokenTree::Punct(p) if p.as_char() == ':'));

        if has_colon {
            if s.contains("Debug") {
                debug_parts.push(s);
            } else {
                debug_parts.push(format!("{} + Debug", s));
            }
        } else {
            debug_parts.push(format!("{}: Debug", name));
        }

        type_parts.push(name);
    }

    en.impl_generics_debug = debug_parts.join(", ");
    en.impl_generics_plain = plain_parts.join(", ");
    en.type_generics = type_parts.join(", ");
}