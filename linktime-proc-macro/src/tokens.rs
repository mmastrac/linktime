use proc_macro::{Delimiter, Literal, TokenTree};

pub(crate) fn decode_literal_string(name: &str, literal: Literal) -> String {
    let literal = literal.to_string();
    let Some(literal) = literal.strip_prefix('"') else {
        panic!("{}: Expected a literal string", name);
    };
    let Some(literal) = literal.strip_suffix('"') else {
        panic!("{}: Expected a literal string", name);
    };
    if !literal.contains('\\') {
        literal.to_string()
    } else {
        let mut output = String::with_capacity(literal.len());
        let mut iter = literal.chars();
        while let Some(c) = iter.next() {
            if c == '\\' {
                match iter.next() {
                    Some('n') => output.push('\n'),
                    Some('r') => output.push('\r'),
                    Some('t') => output.push('\t'),
                    Some('\\') => output.push('\\'),
                    Some('"') => output.push('"'),
                    Some('\'') => output.push('\''),
                    Some('0') => output.push('\0'),
                    Some('x') => {
                        let Some(c) = iter.next() else {
                            panic!("{}: Expected a hexadecimal character", name);
                        };
                        let Some(c2) = iter.next() else {
                            panic!("{}: Expected a hexadecimal character", name);
                        };
                        let Ok(c) = format!("{}{}", c, c2).parse::<u8>() else {
                            panic!("{}: Expected a hexadecimal character", name);
                        };
                        output.push(char::from(c));
                    }
                    Some(_) => panic!("{}: Expected a valid escape sequence", name),
                    None => break,
                }
            } else {
                output.push(c);
            }
        }
        output
    }
}

pub(crate) fn decode_literal_strings(name: &str, item: TokenTree) -> String {
    let mut output = String::new();
    match item {
        TokenTree::Literal(literal) => {
            output.push_str(&decode_literal_string(name, literal));
        }
        TokenTree::Group(group) => {
            for token in group.stream().into_iter() {
                output.push_str(&decode_literal_strings(name, token));
            }
        }
        TokenTree::Punct(_) => {
            // Ignore punctuation
        }
        TokenTree::Ident(ident) => {
            output.push_str(&ident.to_string());
        }
    }
    output
}

pub(crate) fn expect_literal(name: &str, item: TokenTree) -> Literal {
    match item {
        TokenTree::Literal(literal) => literal,
        TokenTree::Group(group) => {
            if group.delimiter() != Delimiter::None {
                panic!(
                    "{}: Expected a single literal, got `{:?}` group",
                    name,
                    group.delimiter()
                );
            }
            let tokens = group.stream().into_iter().collect::<Vec<_>>();
            if tokens.len() != 1 {
                panic!(
                    "{}: Expected a single literal, got `{}`",
                    name,
                    tokens.len()
                );
            }
            expect_literal(name, tokens.into_iter().next().unwrap())
        }
        token => {
            panic!("{}: Expected a literal, got `{token}`", name);
        }
    }
}

pub(crate) fn expect_numeric_literal(name: &str, item: TokenTree) -> usize {
    let literal = expect_literal(name, item).to_string();
    let Ok(literal) = literal.parse::<usize>() else {
        panic!("{}: Expected a literal integer, got `{literal}`", name);
    };
    literal
}
