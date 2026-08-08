mod parse;
mod codegen;

extern crate proc_macro;

use proc_macro::*;

#[derive(Debug)]
struct Enum {
    is_pub: bool,
    name: String,
    vars: Vec<Var>,
}

#[derive(Debug, Default)]
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

    let error = parse_enum(&mut iter);

    codegen::generate_code(error)
}

fn parse_enum(iter: &mut std::iter::Peekable<token_stream::IntoIter>) -> Enum {
    let mut perechislenie = Enum {
        is_pub: false,
        name: String::new(),
        vars: Vec::new(),
    };

    while let Some(token) = iter.next() {
        match token {
            TokenTree::Ident(i) => {
                let name = i.to_string();
                if name == "pub" {
                    perechislenie.is_pub = true;
                }
                if name == "enum" {
                    if let Some(TokenTree::Ident(name_ident)) = iter.next() {
                        perechislenie.name = name_ident.to_string();
                    }
                }
            }
            TokenTree::Group(g) if g.delimiter() == Delimiter::Brace => {
                perechislenie.vars = parse::parse_enum_body(g.stream());
                break;
            }
            _ => {}
        }
    }

    perechislenie
}