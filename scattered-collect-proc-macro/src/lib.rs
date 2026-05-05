use std::iter::FromIterator;

use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};

#[proc_macro_attribute]
pub fn scatter(attribute: TokenStream, item: TokenStream) -> TokenStream {
    generate("scatter", "scattered_collect", attribute, item)
}

#[proc_macro_attribute]
pub fn gather(attribute: TokenStream, item: TokenStream) -> TokenStream {
    generate("gather", "scattered_collect", attribute, item)
}

#[allow(unknown_lints, tail_expr_drop_order)]
fn generate(
    macro_type: &str,
    macro_crate: &str,
    attribute: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let mut inner = TokenStream::new();

    let mut crate_path = None;
    let mut tokens = attribute.clone().into_iter().peekable();

    while let Some(token) = tokens.next() {
        if let TokenTree::Ident(ident) = &token {
            if ident.to_string() == "crate_path" {
                if let Some(TokenTree::Punct(punct)) = tokens.next() {
                    if punct.as_char() == '=' {
                        let mut path = TokenStream::new();
                        while let Some(token) = tokens.peek() {
                            match token {
                                TokenTree::Punct(p) if p.as_char() == ',' => {
                                    tokens.next();
                                    break;
                                }
                                _ => {
                                    path.extend(std::iter::once(tokens.next().unwrap()));
                                }
                            }
                        }
                        crate_path = Some(path);
                        break;
                    }
                }
            }
        }
    }

    if attribute.is_empty() {
        inner.extend([
            TokenTree::Punct(Punct::new('#', Spacing::Alone)),
            TokenTree::Group(Group::new(
                Delimiter::Bracket,
                TokenStream::from_iter([TokenTree::Ident(Ident::new(
                    macro_type,
                    Span::call_site(),
                ))]),
            )),
        ]);
    } else {
        inner.extend([
            TokenTree::Punct(Punct::new('#', Spacing::Alone)),
            TokenTree::Group(Group::new(
                Delimiter::Bracket,
                TokenStream::from_iter([
                    TokenTree::Ident(Ident::new(macro_type, Span::call_site())),
                    TokenTree::Group(Group::new(Delimiter::Parenthesis, attribute)),
                ]),
            )),
        ]);
    }

    inner.extend(item);

    let mut invoke = crate_path.unwrap_or_else(|| {
        TokenStream::from_iter([
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new(macro_crate, Span::call_site())),
        ])
    });

    invoke.extend([
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("__support", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new(
            &format!("{macro_type}_parse"),
            Span::call_site(),
        )),
        TokenTree::Punct(Punct::new('!', Spacing::Alone)),
        TokenTree::Group(Group::new(Delimiter::Parenthesis, inner)),
        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
    ]);

    invoke
}

/// Concatenate two identifiers.
#[proc_macro]
pub fn ident_concat(item: TokenStream) -> TokenStream {
    let mut item = item.into_iter();
    let Some(TokenTree::Group(pre_group)) = item.next() else {
        panic!("pre_group: Expected a group");
    };
    let Some(TokenTree::Group(name_group)) = item.next() else {
        panic!("name_group: Expected a group");
    };
    let Some(TokenTree::Group(post_group)) = item.next() else {
        panic!("post_group: Expected a group");
    };

    let mut item = name_group.stream().into_iter();
    let mut name = String::new();
    while let Some(TokenTree::Ident(ident)) = item.next() {
        name.push_str(&ident.to_string());
    }

    let mut output = pre_group.stream();
    output.extend([TokenTree::Ident(Ident::new(
        &name,
        Span::call_site(),
    ))]);
    output.extend(post_group.stream());
    output
}
