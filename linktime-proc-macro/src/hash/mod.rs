use proc_macro::{Delimiter, Group, Ident, Literal, Span, TokenStream, TokenTree};

use crate::tokens::{
    decode_literal_string, decode_literal_strings, expect_literal, expect_numeric_literal,
};

mod xx3;

/// Concatenate two identifiers.
pub(crate) fn ident_concat(item: TokenStream) -> TokenStream {
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
    output.extend([TokenTree::Ident(Ident::new(&name, Span::call_site()))]);
    output.extend(post_group.stream());
    output
}

/// If the input string is longer than the max length, replace the tail end of
/// the string with the hash of the string.
///
/// hash!(output (prefix) (name) (suffix) hash_length max_length valid_section_chars)
pub(crate) fn hash(item: TokenStream) -> TokenStream {
    let mut item = item.into_iter();

    let Some(TokenTree::Group(group)) = item.next() else {
        panic!("output: Expected a group");
    };
    let group = group.stream();

    let Some(prefix_group) = item.next() else {
        panic!("prefix: Expected a group");
    };
    let prefix = decode_literal_strings("prefix", prefix_group);

    let Some(input_group) = item.next() else {
        panic!("input: Expected an identifier");
    };
    let literal = decode_literal_strings("input", input_group);

    let Some(suffix_group) = item.next() else {
        panic!("suffix: Expected a group");
    };
    let suffix = decode_literal_strings("suffix", suffix_group);

    let hash_length = expect_numeric_literal(
        "hash_length",
        item.next().expect("hash_length: Missing argument"),
    );
    let max_length = expect_numeric_literal(
        "max_length",
        item.next().expect("max_length: Missing argument"),
    );

    let valid_section_chars = expect_literal(
        "valid_section_chars",
        item.next().expect("valid_section_chars: Missing argument"),
    );
    let valid_section_chars =
        decode_literal_string("valid_section_chars", valid_section_chars).into_bytes();

    // If the string is valid as-is, return it
    let output = if literal.len() < max_length
        && !literal
            .to_string()
            .contains(|c| c > '\u{007f}' || !valid_section_chars.contains(&(c as u8)))
    {
        format!("{prefix}{literal}{suffix}")
    } else {
        // Not valid, so we need to hash the string
        let mut output = String::with_capacity(max_length + prefix.len() + suffix.len());
        output.push_str(&prefix.to_string());
        let mut next = literal.chars();
        while output.len() < max_length - hash_length + prefix.len() {
            let Some(c) = next.next() else {
                break;
            };
            if c <= '\u{007f}' && valid_section_chars.contains(&(c as u8)) {
                output.push(c);
            }
        }

        let mut hash = xx3::xx3hash(&literal);
        while output.len() < max_length + prefix.len() {
            let c = valid_section_chars[hash as usize % valid_section_chars.len()];
            output.push(c as char);
            hash /= valid_section_chars.len() as u64;
        }
        output.push_str(&suffix);
        output
    };

    fn emit(tree: TokenStream, output: &str, found: &mut bool) -> TokenStream {
        if *found {
            return tree;
        }
        let mut stream = TokenStream::new();
        for input in tree.into_iter() {
            match input {
                _ if *found => stream.extend([input]),
                TokenTree::Ident(ident) if ident.to_string() == "__" => {
                    stream.extend([TokenTree::Literal(Literal::string(output))]);
                    *found = true;
                }
                TokenTree::Group(group) => stream.extend([TokenTree::Group(Group::new(
                    group.delimiter(),
                    emit(group.stream(), output, found),
                ))]),
                _ => stream.extend([input]),
            }
        }
        stream
    }

    let mut found = false;
    let stream = emit(group, &output, &mut found);
    if !found {
        panic!("output: Expected to find __");
    }
    TokenStream::from_iter([TokenTree::Group(Group::new(Delimiter::None, stream))])
}
