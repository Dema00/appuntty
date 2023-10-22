use crate::backend::node::{Node, WHRef};
use nom::{
    AsBytes, Compare, ExtendInto, FindSubstring, FindToken, InputIter, InputLength, InputTake,
    InputTakeAtPosition, Offset, ParseTo, Slice,
};

/// Custom input type for nom
// USELESS FOR NOW, TOO MUCH WORK, PROBABLY WILL DO WITHOUT IT
#[derive(Clone)]
pub struct Head<'i> {
    pub root: WHRef<Node>,
    pub depth: usize,
    pub input: &'i str,
}

impl<'i> Head<'i> {
    fn clone_new_str(&self, new_str: &'i str) -> Self {
        Head {
            root: std::rc::Weak::clone(&self.root),
            depth: self.depth,
            input: new_str,
        }
    }
}

impl<'i> AsBytes for Head<'i> {
    fn as_bytes(&self) -> &[u8] {
        self.input.as_bytes()
    }
}

impl<'i, 't> Compare<&Head<'t>> for Head<'i> {
    fn compare(&self, t: &Head<'t>) -> nom::CompareResult {
        self.input.compare(t.input)
    }

    fn compare_no_case(&self, t: &Head<'t>) -> nom::CompareResult {
        self.input.compare_no_case(t.input)
    }
}

impl<'i, 't> Compare<&'t str> for Head<'i> {
    fn compare(&self, t: &'t str) -> nom::CompareResult {
        self.input.compare(t)
    }

    fn compare_no_case(&self, t: &'t str) -> nom::CompareResult {
        self.input.compare_no_case(t)
    }
}


impl<'i> ExtendInto for Head<'i> {
    type Item = char;

    type Extender = String;

    fn new_builder(&self) -> Self::Extender {
        String::new()
    }

    fn extend_into(&self, acc: &mut Self::Extender) {
        acc.push_str(self.input);
    }
}

impl<'i, 't> FindSubstring<Head<'t>> for Head<'i> {
    fn find_substring(&self, substr: Head<'t>) -> Option<usize> {
        self.input.find_substring(substr.input)
    }
}

impl<'i, 't> FindToken<u8> for Head<'i> {
    fn find_token(&self, token: u8) -> bool {
        self.input.find_token(token)
    }
}

impl<'i> InputIter for Head<'i> {
    type Item = char;
    type Iter = std::str::CharIndices<'i>;
    type IterElem = std::str::Chars<'i>;

    fn iter_indices(&self) -> Self::Iter {
        self.input.char_indices()
    }

    fn iter_elements(&self) -> Self::IterElem {
        self.input.chars()
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.input.position(predicate)
    }

    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
        self.input.slice_index(count)
    }
}

impl<'i> InputLength for Head<'i> {
    fn input_len(&self) -> usize {
        self.input.input_len()
    }
}

impl<'i> InputTake for Head<'i> {
    fn take(&self, count: usize) -> Self {
        Head {
            root: std::rc::Weak::clone(&self.root),
            depth: self.depth,
            input: self.input.take(count),
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let (prefix, suffix) = self.input.split_at(count);

        (self.clone_new_str(suffix),
        self.clone_new_str(prefix))
    }
}
//  VVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVV
// Possibily extremely fucked, i have no idea if this is safe, i'm just copying the &str implementation for this custom input type
impl<'i> InputTakeAtPosition for Head<'i> {
    type Item = char;

    fn split_at_position<P, E: nom::error::ParseError<Self>>(&self, predicate: P) -> nom::IResult<Self, Self, E>
      where
        P: Fn(Self::Item) -> bool {

        match self.input.find(predicate) {
            // find() returns a byte index that is already in the slice at a char boundary
            Some(i) => unsafe { Ok((self.clone_new_str(self.input.get_unchecked(i..)), self.clone_new_str(self.input.get_unchecked(..i)))) },
            None => Err(nom::Err::Incomplete(nom::Needed::new(1))),
          }
    }

    fn split_at_position1<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
        e: nom::error::ErrorKind,
      ) -> nom::IResult<Self, Self, E>
      where
        P: Fn(Self::Item) -> bool {
            match self.input.find(predicate) {
                Some(0) => Err(nom::Err::Error(E::from_error_kind(self.clone(), e))),
                // find() returns a byte index that is already in the slice at a char boundary
                Some(i) => unsafe { Ok((self.clone_new_str(self.input.get_unchecked(i..)), self.clone_new_str(self.input.get_unchecked(..i)))) },
                None => Err(nom::Err::Incomplete(nom::Needed::new(1))),
              }
          
    }

    fn split_at_position_complete<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
      ) -> nom::IResult<Self, Self, E>
      where
        P: Fn(Self::Item) -> bool {
            match self.input.find(predicate) {
                // find() returns a byte index that is already in the slice at a char boundary
                Some(i) => unsafe { Ok((self.clone_new_str(self.input.get_unchecked(i..)), self.clone_new_str(self.input.get_unchecked(..i)))) },
                // the end of slice is a char boundary
                None => unsafe {
                  Ok((
                    self.clone_new_str(self.input.get_unchecked(self.input.len()..)),
                    self.clone_new_str(self.input.get_unchecked(..self.input.len())),
                  ))
                },
              }
    }

    fn split_at_position1_complete<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
        e: nom::error::ErrorKind,
      ) -> nom::IResult<Self, Self, E>
      where
        P: Fn(Self::Item) -> bool {
            match self.input.find(predicate) {
                Some(0) => Err(nom::Err::Error(E::from_error_kind(self.clone(), e))),
                // find() returns a byte index that is already in the slice at a char boundary
                Some(i) => unsafe { Ok((self.clone_new_str(self.input.get_unchecked(i..)), self.clone_new_str(self.input.get_unchecked(..i)))) },
                None => {
                  if self.input.is_empty() {
                    Err(nom::Err::Error(E::from_error_kind(self.clone(), e)))
                  } else {
                    // the end of slice is a char boundary
                    unsafe {
                      Ok((
                        self.clone_new_str(self.input.get_unchecked(self.input.len()..)),
                        self.clone_new_str(self.input.get_unchecked(..self.input.len())),
                      ))
                    }
                  }
                }
              }
          
    }
}

impl<'i> Offset for Head<'i> {
    fn offset(&self, second: &Self) -> usize {
        self.input.offset(second.input)
    }
}

impl<'i, R: std::str::FromStr> ParseTo<R> for Head<'i> {
    fn parse_to(&self) -> Option<R> {
        self.input.parse_to()
    }
}

impl<'i> Slice<std::ops::Range<usize>> for Head<'i> {
    fn slice(&self, range: std::ops::Range<usize>) -> Self {
        self.clone_new_str(self.input.slice(range))
    }
}

impl<'i> Slice<std::ops::RangeTo<usize>> for Head<'i> {
    fn slice(&self, range: std::ops::RangeTo<usize>) -> Self {
        self.clone_new_str(self.input.slice(range))
    }
}

impl<'i> Slice<std::ops::RangeFrom<usize>> for Head<'i> {
    fn slice(&self, range: std::ops::RangeFrom<usize>) -> Self {
        self.clone_new_str(self.input.slice(range))
    }
}

impl<'i> Slice<std::ops::RangeFull> for Head<'i> {
    fn slice(&self, range: std::ops::RangeFull) -> Self {
        self.clone_new_str(self.input.slice(range))
    }
}
