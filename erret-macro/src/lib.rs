extern crate proc_macro;

mod codegen;
mod parse;

use proc_macro::*;

#[derive(Debug, Default)]
struct Enum {
    pub name: String,
    pub has_generics: bool,
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
    while let Some(token) = iter.next() {
        match token {
            TokenTree::Ident(i) => {
                if i.to_string() == "enum" {
                    if let Some(TokenTree::Ident(n)) = iter.next() {
                        en.name = n.to_string();
                    }
                }
            }
            TokenTree::Punct(p) if p.as_char() == '<' => {
                en.has_generics = true;
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