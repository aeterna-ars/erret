use crate::{FieldInfo, Var};
use proc_macro::*;

pub fn parse_enum_body(stream: TokenStream) -> Vec<Var> {
    let mut iter = stream.into_iter().peekable();
    let mut vars = Vec::new();

    while iter.peek().is_some() {
        let mut variant_has_from = false;
        let mut error_arg: Option<String> = None;

        loop {
            match iter.peek() {
                Some(TokenTree::Punct(p)) if p.as_char() == '#' => {
                    iter.next();
                    if let Some(TokenTree::Group(g)) = iter.next() {
                        if g.delimiter() == Delimiter::Bracket {
                            let mut inner = g.stream().into_iter();
                            if let Some(TokenTree::Ident(id)) = inner.next() {
                                match id.to_string().as_str() {
                                    "from" => variant_has_from = true,
                                    "error" => {
                                        if let Some(TokenTree::Group(args)) = inner.next() {
                                            if args.delimiter() == Delimiter::Parenthesis {
                                                if let Some(TokenTree::Literal(lit)) =
                                                    args.stream().into_iter().next()
                                                {
                                                    let s = lit.to_string();
                                                    error_arg =
                                                        Some(s.trim_matches('"').to_string());
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                _ => break,
            }
        }

        let name = match iter.next() {
            Some(TokenTree::Ident(id)) => id.to_string(),
            Some(_) => continue,
            None => break,
        };

        let mut fields = Vec::new();

        match iter.peek() {
            Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis => {
                if let Some(TokenTree::Group(g)) = iter.next() {
                    fields = parse_tuple_fields(g.stream(), variant_has_from);
                }
            }
            Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => {
                if let Some(TokenTree::Group(g)) = iter.next() {
                    fields = parse_named_fields(g.stream(), variant_has_from);
                }
            }
            _ => {}
        }

        vars.push(Var {
            name,
            fields,
            error_arg,
        });

        if let Some(TokenTree::Punct(p)) = iter.peek() {
            if p.as_char() == ',' {
                iter.next();
            }
        }
    }

    vars
}

fn parse_tuple_fields(stream: TokenStream, variant_has_from: bool) -> Vec<FieldInfo> {
    let mut result = Vec::new();
    let mut current: Vec<TokenTree> = Vec::new();
    let mut depth: i32 = 0;
    let mut field_has_from = variant_has_from;
    let mut iter = stream.into_iter().peekable();

    while let Some(t) = iter.next() {
        match &t {
            TokenTree::Punct(p) if p.as_char() == '#' => {
                if let Some(TokenTree::Group(g)) = iter.next() {
                    if g.delimiter() == Delimiter::Bracket {
                        if let Some(TokenTree::Ident(id)) = g.stream().into_iter().next() {
                            if id.to_string() == "from" {
                                field_has_from = true;
                            }
                        }
                    }
                }
            }
            TokenTree::Punct(p) => {
                let c = p.as_char();
                match c {
                    '<' | '(' | '[' => {
                        depth += 1;
                        current.push(t.clone());
                    }
                    '>' | ')' | ']' => {
                        depth -= 1;
                        current.push(t.clone());
                    }
                    ',' if depth == 0 => {
                        flush_field(&mut result, &mut current, &mut field_has_from);
                    }
                    _ => current.push(t.clone()),
                }
            }
            _ => current.push(t.clone()),
        }
    }
    flush_field(&mut result, &mut current, &mut field_has_from);
    result
}

fn flush_field(result: &mut Vec<FieldInfo>, current: &mut Vec<TokenTree>, has_from: &mut bool) {
    if current.is_empty() {
        *has_from = false;
        return;
    }
    let tokens: TokenStream = current.drain(..).collect();
    let ty = tokens.to_string().trim().to_string();
    if !ty.is_empty() {
        result.push(FieldInfo {
            name: None,
            ty,
            has_from: *has_from,
        });
    }
    *has_from = false;
}

fn parse_named_fields(stream: TokenStream, variant_has_from: bool) -> Vec<FieldInfo> {
    let mut result = Vec::new();
    let mut iter = stream.into_iter().peekable();
    let mut field_has_from = variant_has_from;

    loop {
        loop {
            match iter.peek() {
                Some(TokenTree::Punct(p)) if p.as_char() == '#' => {
                    iter.next();
                    if let Some(TokenTree::Group(g)) = iter.next() {
                        if g.delimiter() == Delimiter::Bracket {
                            if let Some(TokenTree::Ident(id)) = g.stream().into_iter().next() {
                                if id.to_string() == "from" {
                                    field_has_from = true;
                                }
                            }
                        }
                    }
                }
                _ => break,
            }
        }

        let name = match iter.next() {
            Some(TokenTree::Ident(id)) => id.to_string(),
            Some(_) => continue,
            None => break,
        };

        match iter.next() {
            Some(TokenTree::Punct(p)) if p.as_char() == ':' => {}
            _ => break,
        }

        let mut ty_tokens: Vec<TokenTree> = Vec::new();
        let mut depth: i32 = 0;
        while let Some(t) = iter.next() {
            match &t {
                TokenTree::Punct(p) => {
                    let c = p.as_char();
                    match c {
                        '<' | '(' | '[' => {
                            depth += 1;
                            ty_tokens.push(t);
                        }
                        '>' | ')' | ']' => {
                            depth -= 1;
                            ty_tokens.push(t);
                        }
                        ',' if depth == 0 => break,
                        _ => ty_tokens.push(t),
                    }
                }
                _ => ty_tokens.push(t),
            }
        }

        let tokens: TokenStream = ty_tokens.into_iter().collect();
        let ty = tokens.to_string().trim().to_string();
        if !ty.is_empty() {
            result.push(FieldInfo {
                name: Some(name),
                ty,
                has_from: field_has_from,
            });
        }
        field_has_from = false;

        if iter.peek().is_none() {
            break;
        }
    }

    result
}