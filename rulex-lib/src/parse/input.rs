use std::{iter::Enumerate, ops::Range};

use logos::Logos;
use nom::{InputIter, InputLength, InputTake};

use crate::error::{ParseError, ParseErrorKind};

use super::token::Token;

#[derive(Clone)]
pub(crate) struct Input<'i, 'b> {
    source: &'i str,
    tokens: &'b [(Token, (usize, usize))],
}

impl<'i, 'b> Input<'i, 'b> {
    pub(super) fn tokenize(
        source: &'i str,
        buf: &'b mut Vec<(Token, (usize, usize))>,
    ) -> Result<Self, ParseError> {
        assert!(buf.is_empty());

        let lex = Token::lexer(source);
        buf.extend(lex.spanned().map(|(t, r)| (t, (r.start, r.end))));

        let error = buf.iter().find_map(|&(t, (start, end))| match t {
            Token::Error => Some((start, end, None)),
            Token::ErrorMsg(m) => Some((start, end, Some(m))),
            _ => None,
        });
        if let Some((start, end, msg)) = error {
            return match msg {
                Some(msg) => Err(ParseErrorKind::LexErrorWithMessage(msg).at(start..end)),
                None => Err(ParseErrorKind::LexError.at(start..end)),
            };
        }

        let tokens = &**buf;
        Ok(Input { source, tokens })
    }

    pub(super) fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub(crate) fn index(&self) -> Range<usize> {
        self.tokens
            .first()
            .map(|&(_, (start, end))| start..end)
            .unwrap_or_else(|| self.source.len()..self.source.len())
    }

    pub(super) fn peek(&self) -> Option<(Token, &'i str)> {
        self.iter_elements().next()
    }
}

impl<'i, 'b> PartialEq for Input<'i, 'b> {
    fn eq(&self, other: &Self) -> bool {
        Iterator::eq(self.iter_elements(), other.iter_elements())
    }
}

#[cfg(feature = "dbg")]
impl<'i, 'b> core::fmt::Debug for Input<'i, 'b> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        struct FmtHelper<'a>(Token, &'a str);

        impl core::fmt::Debug for FmtHelper<'_> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{:?} {:?}", self.0, self.1)
            }
        }

        let v: Vec<_> = self
            .tokens
            .iter()
            .map(|&(t, (start, end))| FmtHelper(t, &self.source[start..end]))
            .collect();

        v.fmt(f)
    }
}

impl<'i, 'b> Iterator for Input<'i, 'b> {
    type Item = (Token, &'i str);

    fn next(&mut self) -> Option<Self::Item> {
        match self.tokens.split_first() {
            Some((&(token, range), rest)) => {
                self.tokens = rest;
                Some((token, &self.source[range.0..range.1]))
            }
            None => None,
        }
    }
}

impl<'i, 'b> InputIter for Input<'i, 'b> {
    type Item = (Token, &'i str);

    type Iter = Enumerate<Self>;

    type IterElem = Self;

    fn iter_indices(&self) -> Self::Iter {
        self.iter_elements().enumerate()
    }

    fn iter_elements(&self) -> Self::IterElem {
        Input {
            source: self.source,
            tokens: self.tokens,
        }
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        let mut iter = self.iter_elements();
        Iterator::position(&mut iter, predicate)
    }

    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
        if count <= self.tokens.len() {
            Ok(count)
        } else {
            Err(nom::Needed::Size(
                (count - self.tokens.len()).try_into().unwrap(),
            ))
        }
    }
}

impl<'i, 'b> InputLength for Input<'i, 'b> {
    fn input_len(&self) -> usize {
        self.tokens.len()
    }
}

impl<'i, 'b> InputTake for Input<'i, 'b> {
    fn take(&self, count: usize) -> Self {
        let tokens = &self.tokens[..count];

        Input {
            source: self.source,
            tokens,
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let (left, right) = self.tokens.split_at(count);

        (
            Input {
                source: self.source,
                tokens: left,
            },
            Input {
                source: self.source,
                tokens: right,
            },
        )
    }
}
