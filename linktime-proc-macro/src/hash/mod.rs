use proc_macro::{Delimiter, TokenStream, TokenTree};
use std::path::Path;

pub(crate) mod xx3;

struct TokenTreeDeepIterator {
    stack: Vec<proc_macro::token_stream::IntoIter>,
}

impl Iterator for TokenTreeDeepIterator {
    type Item = TokenTree;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let mut iter = self.stack.pop()?;
            let Some(token) = iter.next() else {
                continue;
            };
            self.stack.push(iter);
            if let TokenTree::Group(group) = &token {
                self.stack.push(group.stream().into_iter());
            }
            return Some(token);
        }
    }
}

/// A stable, position-independent identity for a single token: its own textual
/// content (for leaves) or its delimiter (for groups). Group contents are
/// visited separately by the deep iterator, so the delimiter alone is enough to
/// keep groups distinct without pulling in the (potentially large) subtree.
fn token_content(token: &TokenTree) -> String {
    match token {
        TokenTree::Group(group) => match group.delimiter() {
            Delimiter::Parenthesis => "(".to_string(),
            Delimiter::Brace => "{".to_string(),
            Delimiter::Bracket => "[".to_string(),
            Delimiter::None => String::new(),
        },
        TokenTree::Ident(ident) => ident.to_string(),
        TokenTree::Punct(punct) => punct.as_char().to_string(),
        TokenTree::Literal(literal) => literal.to_string(),
    }
}

/// Hashes the location (file/line/column) of every token within `tokens`.
///
/// If `ignore_base` is set, any token whose local source file lives under that
/// base directory contributes its textual content instead of its location. This
/// makes the hash stable against source movement within the "ignored" crate
/// (e.g. the tokens a declarative macro synthesizes at its own definition site),
/// while still distinguishing genuinely different tokens.
#[allow(clippy::unnecessary_map_or)]
pub(crate) fn location_hash(tokens: TokenStream, ignore_base: Option<&Path>) -> u64 {
    let iterator = TokenTreeDeepIterator {
        stack: vec![tokens.into_iter()],
    };

    // TODO: Can we avoid doing multiple hashes?
    let mut buffer: Vec<u8> = Vec::with_capacity(1024);
    let mut last_hash = 0_u64;
    for token in iterator {
        let span = token.span();
        buffer.clear();
        buffer.extend_from_slice(&last_hash.to_be_bytes());

        let ignored = ignore_base.map_or(false, |base| {
            crate::fallback::local_file(&span).map_or(false, |file| file.starts_with(base))
        });

        if ignored {
            // Content-only: neutralize file/line/column so edits to the ignored
            // crate's own source don't perturb the hash.
            buffer.extend_from_slice(token_content(&token).as_bytes());
        } else {
            let line = crate::fallback::line(&span);
            let column = crate::fallback::column(&span);
            let file = crate::fallback::file(&span);
            buffer.extend_from_slice(&line.to_be_bytes());
            buffer.extend_from_slice(&column.to_be_bytes());
            buffer.extend_from_slice(file.as_bytes());
        }

        last_hash = crate::hash::xx3::xx3hash_bytes(&buffer);
    }

    last_hash
}
