use proc_macro::*;

use crate::{Var, FieldInfo};

pub fn parse_enum_body(stream: TokenStream) -> Vec<Var> {
    let mut iter = stream.into_iter().peekable();
    let mut parsed_vars = Vec::new();

    let mut current_var = Var::default();
    let mut variant_has_from = false;

    while let Some(token) = iter.next() {
        match token {
            TokenTree::Punct(p) if p.as_char() == '#' => {
                if let Some(TokenTree::Group(g)) = iter.next() {
                    if g.delimiter() == Delimiter::Bracket {
                        if let Some(TokenTree::Ident(id)) = g.stream().into_iter().next() {
                            match id.to_string().as_str() {
                                "from" => variant_has_from = true,
                                "error" => parse_attrs(g, &mut current_var),
                                _ => {}
                            }
                        }
                    }
                }
            }

            TokenTree::Ident(ident) => {
                current_var.name = ident.to_string();

                if let Some(TokenTree::Group(g)) = iter.peek() {
                    match g.delimiter() {
                        Delimiter::Parenthesis => {
                            let fields_stream = g.stream().to_string();
                            for ty in fields_stream.split(',') {
                                let clean_ty = ty.trim().to_string();
                                if !clean_ty.is_empty() {
                                    current_var.fields.push(FieldInfo {
                                        name: None,
                                        ty: clean_ty,
                                        has_from: variant_has_from,
                                    });
                                    variant_has_from = false;
                                }
                            }
                            iter.next();
                        }
                        Delimiter::Brace => {
                            let mut fields_iter = g.stream().into_iter().peekable();
                            let mut field_has_from = false;

                            while let Some(f_token) = fields_iter.next() {
                                match f_token {
                                    TokenTree::Punct(ref p) if p.as_char() == '#' => {
                                        if let Some(TokenTree::Group(attr_g)) = fields_iter.next() {
                                            if attr_g.delimiter() == Delimiter::Bracket {
                                                if let Some(TokenTree::Ident(id)) = attr_g.stream().into_iter().next() {
                                                    if id.to_string() == "from" {
                                                        field_has_from = true;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    TokenTree::Ident(f_ident) => {
                                        let f_name = f_ident.to_string();
                                        if let Some(TokenTree::Punct(p)) = fields_iter.next() {
                                            if p.as_char() == ':' {
                                                let mut ty_str = String::new();
                                                while let Some(next_t) = fields_iter.peek() {
                                                    if let TokenTree::Punct(punct) = next_t {
                                                        if punct.as_char() == ',' {
                                                            fields_iter.next();
                                                            break;
                                                        }
                                                    }
                                                    ty_str.push_str(&fields_iter.next().unwrap().to_string());
                                                }
                                                current_var.fields.push(FieldInfo {
                                                    name: Some(f_name.clone()),
                                                    ty: ty_str.trim().to_string(),
                                                    has_from: field_has_from,
                                                });
                                                field_has_from = false;
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            iter.next();
                        }
                        _ => {}
                    }
                }

                parsed_vars.push(std::mem::take(&mut current_var));
                variant_has_from = false;

                if let Some(TokenTree::Punct(p)) = iter.peek() {
                    if p.as_char() == ',' {
                        iter.next();
                    }
                }
            }
            _ => {}
        }
    }

    parsed_vars
}

pub fn parse_attrs(group: Group, var: &mut Var) {
    if group.delimiter() == Delimiter::Bracket {
        let mut inner_iter = group.stream().into_iter();
        if let Some(TokenTree::Ident(i)) = inner_iter.next() {
            if i.to_string() == "error" {
                if let Some(TokenTree::Group(args_g)) = inner_iter.next() {
                    if args_g.delimiter() == Delimiter::Parenthesis {
                        if let Some(TokenTree::Literal(lit)) = args_g.stream().into_iter().next() {
                            var.error_arg = Some(lit.to_string().trim_matches('"').to_string());
                        }
                    }
                }
            }
        }
    }
}