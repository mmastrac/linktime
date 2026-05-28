use proc_macro::{Span, TokenStream, TokenTree};

pub(crate) mod xx3;

struct TokenTreeDeepIterator {
    stack: Vec<proc_macro::token_stream::IntoIter>,
}

impl Iterator for TokenTreeDeepIterator {
    type Item = Span;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let mut iter = self.stack.pop()?;
            let Some(token) = iter.next() else {
                continue;
            };
            self.stack.push(iter);
            match token {
                TokenTree::Group(group) => {
                    self.stack.push(group.stream().into_iter());
                    return Some(group.span());
                }
                _ => return Some(token.span()),
            }
        }
    }
}

pub(crate) fn location_hash(tokens: TokenStream) -> u64 {
    let iterator = TokenTreeDeepIterator {
        stack: vec![tokens.into_iter()],
    };

    // TODO: Can we avoid doing multiple hashes?
    let mut buffer = [0_u8; 1024];
    let mut last_hash = 0_u64;
    for span in iterator {
        let hash = last_hash.to_be_bytes();
        let line = crate::fallback::line(&span).to_be_bytes();
        let column = crate::fallback::column(&span).to_be_bytes();
        let file = crate::fallback::file(&span);
        let mut len = 0;

        for buf in [
            hash.as_slice(),
            line.as_slice(),
            column.as_slice(),
            file.as_bytes(),
        ] {
            buffer[len..len + buf.len()].copy_from_slice(buf);
            len += buf.len();
        }

        last_hash = crate::hash::xx3::xx3hash_bytes(&buffer[..len]);
    }

    last_hash
}
