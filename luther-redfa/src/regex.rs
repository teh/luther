// Copyright 2018 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

use std::iter::FromIterator;

use alphabet::Alphabet;
use partition::{PartitionSet, PartitionSetRangeIter};
use typed_arena::Arena;

/// A context for creating regular expressions.
///
/// The factory methods in `RegexContext` create different kinds of `Regex` but
/// also maintain those `Regex` in `≈-cannonical` form as this is defined in section
/// 4.1 of Owens et al. The need to maintain the regular expressions in cannonical form
/// is why there is no means of creating a `Regex` from a `RegexKind`.
///
/// # Type Parameter
/// - A: the alphabet over which the regular expression operates
pub struct RegexContext<A: Alphabet> {
    arena: Arena<RegexKind<A>>,
}

impl<A: Alphabet> RegexContext<A> {
    /// Create a new `RegexContext`.
    pub fn new() -> RegexContext<A> {
        RegexContext {
            arena: Arena::new(),
        }
    }

    /// Create an empty `Regex`.
    ///
    /// The empty regular expressions matches everything, including the empty
    /// string.
    pub fn empty(&self) -> Regex<A> {
        Regex {
            kind: self.arena.alloc(RegexKind::Empty),
        }
    }

    /// Create a character class `Regex`.
    ///
    /// The class regular expression matches a single character from one of the
    /// ranges specified by `ranges`. This factory can also create the empty set
    /// by passing in an empty iterator for `ranges`. The empty set does not
    /// match anything.
    pub fn class<I>(&self, ranges: I) -> Regex<A>
    where
        I: IntoIterator<Item = Range<A>>,
    {
        let class = ranges.into_iter().collect();
        Regex {
            kind: self.arena.alloc(RegexKind::Class(class)),
        }
    }

    pub fn concat(&self) -> Regex<A> {
        unimplemented!()
    }

    pub fn repetition(&self) -> Regex<A> {
        unimplemented!()
    }

    pub fn alteration(&self) -> Regex<A> {
        unimplemented!()
    }

    pub fn and(&self) -> Regex<A> {
        unimplemented!()
    }

    pub fn complement(&self) -> Regex<A> {
        unimplemented!()
    }
}

/// A regular expression.
///
/// A `Regex` is created by the factory methods in `RegexContext` and is
/// associated with that context. It is not possible to create a `Regex`
/// directly. It is also not possible to create a `Regex` from a `RegexKind` in
/// order to allow `RegexContext` to maintain certain regular expressions in
/// cannonical form.
pub struct Regex<'a, A: 'a + Alphabet> {
    kind: &'a RegexKind<A>,
}

impl<'a, A: Alphabet> Regex<'a, A> {
    /// Get the kind of the regular expression.
    pub fn kind(&self) -> &RegexKind<A> {
        &self.kind
    }
}

/// The kind of a regular expressions.
///
/// # Type Parameter
/// - A: the alphabet over which the regular expression operates
#[derive(Debug, PartialEq)]
pub enum RegexKind<A: Alphabet> {
    /// The empty regular expressions which matches everything, including the
    /// empty string.
    Empty,

    /// A regular expressions which matches one character from a (possibly empty)
    /// subset of the alphabet `A`.
    ///
    /// If the subset is empty then the resulting regular expression will match
    /// nothing.
    Class(Class<A>),
    Concat,
    Repetition,
    Alteration,
    And,
    Complement,
}

/// A (possibly empty) subset of the alphabet `A`.
#[derive(Debug, PartialEq)]
pub struct Class<A: Alphabet> {
    set: PartitionSet<A>,
}

impl<A: Alphabet> Class<A> {
    /// Get an iterator over the closed ranges that make up the `Class`.
    ///
    /// The ranges returned by the iterator will be non-overlapping ranges
    /// and will be in increasing order. Adjacent ranges will also be combined.
    pub fn ranges<'a>(&'a self) -> Ranges<'a, A> {
        Ranges {
            inner: self.set.into_iter(),
        }
    }
}

impl<A: Alphabet> FromIterator<Range<A>> for Class<A> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Range<A>>,
    {
        Class {
            set: iter.into_iter().collect(),
        }
    }
}

/// An iterator over the closed ranges of a class.
///
/// This is the return type of the `Class<A>::ranges()` method.
pub struct Ranges<'a, A: 'a + Alphabet> {
    inner: PartitionSetRangeIter<'a, A>,
}

impl<'a, A: Alphabet> Iterator for Ranges<'a, A> {
    type Item = Range<A>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

/// An inclusive range of charaters from the alphabet `A`.
#[derive(Debug, PartialEq, Clone)]
pub struct Range<A: Alphabet> {
    start: A,
    end: A,
}

impl<A: Alphabet> Range<A> {
    /// Creates a new range of characters.
    ///
    /// If `end` is less than the `start` then they will be reversed.
    pub fn new(start: A, end: A) -> Range<A> {
        if end < start {
            Range {
                start: end,
                end: start,
            }
        } else {
            Range { start, end }
        }
    }

    /// The start of the range of characters.
    ///
    /// The start is included in the range.
    pub fn start(&self) -> A {
        self.start.clone()
    }

    /// The end of the range of characters.
    ///
    /// The end is included in the range.
    pub fn end(&self) -> A {
        self.end.clone()
    }

    pub(crate) fn coalesce(&self, other: &Self) -> Result<Self, (Self, Self)> {
        let (anchor, comp) = if self.start <= other.start {
            (self, other)
        } else {
            (other, self)
        };

        comp.start
            .decrement()
            .and_then(|start| {
                if start <= anchor.end {
                    Some(Range::new(anchor.start.clone(), comp.end.clone()))
                } else {
                    None
                }
            })
            .ok_or_else(|| (self.clone(), other.clone()))
    }
}

impl<'a, A: Alphabet> PartialEq<Range<A>> for &'a Range<A> {
    fn eq(&self, other: &Range<A>) -> bool {
        self.start == other.start && self.end == other.end
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty_regex_has_kind_empty() {
        let ctx = RegexContext::<char>::new();

        let sut = ctx.empty();

        assert_eq!(sut.kind(), &RegexKind::Empty);
    }

    #[test]
    fn class_regex_has_kind_class() {
        let ctx = RegexContext::new();
        let ranges = vec![Range::new('a', 'c'), Range::new('g', 'h')];

        let sut = ctx.class(ranges);

        assert_matches!(sut.kind(), &RegexKind::Class(_));
    }

    #[test]
    fn class_regex_round_trips_simple_ranges() {
        let ctx = RegexContext::new();
        let expected = vec![Range::new('a', 'c'), Range::new('g', 'h')];

        let sut = ctx.class(expected.clone());

        assert_matches!(sut.kind(), &RegexKind::Class(ref class) => {
            let ranges: Vec<Range<_>> = class.ranges().collect();
            assert_eq!(ranges, expected);
        });
    }
}