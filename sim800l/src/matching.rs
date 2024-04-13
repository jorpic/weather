pub struct Match<'a, T> {
    word: &'a [T],
    matched_len: usize
}

#[derive(Debug, PartialEq)]
pub enum MatchResult { Match, Progress, Reset } // check for overflow

impl<'a, T: PartialEq> Match<'a, T> {
    pub fn new(word: &'a [T]) -> Self {
        assert!(word.len() > 0);
        Match { word, matched_len: 0 }
    }

    pub fn reset(&mut self) {
        self.matched_len = 0;
    }

    pub fn skip_in<I: IntoIterator<Item=T>>(&mut self, input: I) -> bool {
        let mut inp = input.into_iter();
        loop {
            match inp.next() {
                None => return false,
                Some(b) => match self.add(b) {
                    MatchResult::Match => return true,
                    MatchResult::Progress => continue,
                    MatchResult::Reset => return false,
                }
            }
        }
    }

    pub fn find_in<I: IntoIterator<Item=T>>(&mut self, input: I) -> bool {
        for b in input {
            if self.add(b) == MatchResult::Match {
                return true;
            }
        }
        false
    }

    pub fn add(&mut self, b: T) -> MatchResult {
        if self.word.len() <= self.matched_len {
            self.matched_len = 0;
        }

        if b == self.word[self.matched_len] {
            self.matched_len += 1;
            if self.word.len() == self.matched_len {
                return MatchResult::Match;
            }
            return MatchResult::Progress;
        } else {
            self.matched_len = 0;
            return MatchResult::Reset;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_add() {
        let mut m = Match::new("hello!".as_bytes());
        assert_eq!(m.add(b'h'), MatchResult::Progress);
        assert_eq!(m.add(b'e'), MatchResult::Progress);
        assert_eq!(m.add(b'#'), MatchResult::Reset);
        for b in "hello".bytes() {
            m.add(b);
        }
        assert_eq!(m.add(b'!'), MatchResult::Match);
    }

    #[test]
    fn test_match_expect() {
        let mut m = Match::new("hello!".as_bytes());
        assert_eq!(true, m.skip_in("hello!".bytes()));
        m.reset();
        assert_eq!(true, m.skip_in("hello! world!".bytes()));
        m.reset();
        assert_eq!(false, m.skip_in("-hello!".bytes()));
        m.reset();
        assert_eq!(false, m.skip_in("hello".bytes()));
    }

    #[test]
    fn test_match_find() {
        let mut m = Match::new("hello!".as_bytes());
        assert_eq!(true, m.find_in("hello!".bytes()));
        m.reset();
        assert_eq!(true, m.find_in("--->hello!<----".bytes()));
        m.reset();
        assert_eq!(false, m.find_in("hello".bytes()));
    }
}
