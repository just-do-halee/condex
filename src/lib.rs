// Copyright 2021 Hwakyeom Kim(=just-do-halee)

//! `collectfiles`
//! ## Example
//! ```ignore
//!    #[derive(Debug, Clone, Copy, PartialEq)]
//!    enum Token {
//!        TagName,
//!        NameType,
//!        Value,
//!        AllInOne,
//!    }
//!    impl TokenKind for Token {}
//!
//!    let mut builder = CondexBuilder::new(&[
//!        (Token::TagName, &["@-("]),
//!        (Token::NameType, &["[(,]  -  :  - [,=]"]),
//!        (Token::Value, &["=-[,)]"]),
//!        (Token::AllInOne, &["@-(", "[(,]  -  :  - [,=]", "=-[,)]"]),
//!    ]);
//!
//!    let source = "@hello-man(name: type = value, name2: type2, name3: type3 = value3)";
//!
//!    for (i, c) in source.char_indices() {
//!        builder.test(c, i);
//!    }
//!    let finals = builder.finalize_with_source(source);
//!    eprintln!("{:#?}", finals);
//! ```

use std::{
    fmt,
    iter::{Cycle, Peekable},
    ops::Range,
    str::Chars,
};

use rayon::prelude::*;

pub type CondexPair<'s, T> = (T, &'s [&'s str]);
pub trait TokenKind: fmt::Debug + Clone + Copy + PartialEq + Send + Sync {}

pub struct CondexBuilder<'s, T: TokenKind> {
    pub condexes: Vec<(&'s T, CondexComponent<'s>)>,
}

impl<'s, T: TokenKind> CondexBuilder<'s, T> {
    #[inline]
    pub fn new(condexes: &'s [CondexPair<T>]) -> Self {
        let mut vec = Vec::with_capacity(condexes.len());
        for (kind, condex) in condexes {
            vec.push((kind, Condex::new(condex)));
        }
        Self { condexes: vec }
    }
    #[inline]
    pub fn test(&mut self, c: char, i: usize) {
        self.condexes
            .par_iter_mut()
            .for_each(|(_, condex)| condex.par_iter_mut().for_each(|con| con.test(c, i)));
    }
    #[inline]
    pub fn finalize(self) -> Vec<(T, Vec<CondexResult>)> {
        self.condexes
            .into_par_iter()
            .map(|(&kind, condex)| {
                let results = condex.into_par_iter().flat_map(|con| con.results).collect();
                (kind, results)
            })
            .collect()
    }
    #[inline]
    pub fn finalize_with_source(self, source: &'s str) -> Vec<(T, Vec<CondexResultStr<'s>>)> {
        self.condexes
            .into_par_iter()
            .map(|(&kind, condex)| {
                let results = condex
                    .into_par_iter()
                    .flat_map(|con| {
                        con.results
                            .into_par_iter()
                            .map(|result| {
                                result
                                    .into_par_iter()
                                    .map(|span| source.get(span).unwrap().trim())
                                    .collect()
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect();
                (kind, results)
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CondexState {
    Await,
    Record,
}
pub type Span = Range<usize>;
pub type CondexComponent<'s> = Vec<Condex<'s>>;
pub type CondexResult = Vec<Span>;
pub type CondexResultStr<'s> = Vec<&'s str>;

#[derive(Clone)]
pub struct Condex<'s> {
    condex: Peekable<Cycle<Chars<'s>>>,
    current_state: CondexState,
    prev_i: usize,
    condex_len: usize,
    result_len: usize,
    result: CondexResult,
    results: Vec<CondexResult>,
    or_conditions: Vec<char>,
}
impl<'s> std::fmt::Debug for Condex<'s> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "state: {:?}\npending index: {}\nresult len: {}\npending: {:#?}\nresults: {:#?}\nor conditions: {:#?}",
            self.current_state, self.prev_i, self.result_len, self.result, self.results, self.or_conditions
        )
    }
}

impl<'s> Condex<'s> {
    #[inline]
    pub fn new(condexes: &[&'s str]) -> CondexComponent<'s> {
        let mut vec = Vec::with_capacity(condexes.len());
        for condex in condexes {
            vec.push(Self::_new(condex));
        }
        vec
    }
    #[inline]
    fn _new(condex: &'s str) -> Self {
        let temp = condex.chars();
        let mut result_len = 0;
        let mut condex_len = 0;
        for c in temp {
            if c == '-' {
                result_len += 1;
            }
            condex_len += 1;
        }
        Self {
            condex: condex.chars().cycle().peekable(),
            current_state: CondexState::Await,
            prev_i: 0,
            condex_len,
            result_len,
            result: Vec::with_capacity(result_len),
            results: Vec::new(),
            or_conditions: Vec::new(),
        }
    }
    #[inline]
    pub fn test(&mut self, c: char, i: usize) {
        if c == ' ' {
            // skip a space
            return;
        }
        let target_c = self.next();
        if match self.current_state {
            CondexState::Await => {
                if self.or_conditions.is_empty() {
                    c == target_c
                } else {
                    self.or_conditions.contains(&c)
                }
            }
            CondexState::Record => {
                if if self.or_conditions.is_empty() {
                    c == target_c
                } else {
                    self.or_conditions.contains(&c)
                } {
                    self.result.push(self.prev_i..i);
                    if self.result.len() >= self.result_len {
                        self.results.push(self.result.clone());
                        self.result.clear();
                    }
                    true
                } else {
                    false
                }
            }
        } {
            self.prev_i = i + 1;
            self.or_conditions.clear();
            self.reset_state();
            self.condex_next();
        } else {
            let _ = self.condex.by_ref().skip(self.condex_len);
        }
    }
    #[inline]
    fn next(&mut self) -> char {
        let mut c = self.condex_peek();
        if c == ' ' {
            // skip a space
            loop {
                c = self.condex_peek();
                if c != ' ' {
                    break;
                } else {
                    self.condex_next();
                }
            }
        }

        match c {
            '-' => {
                self.set_state(CondexState::Record);
                self.condex_next();
                self.next()
            }
            '[' => {
                self.or_conditions.clear();
                self.condex_next();

                loop {
                    match self.condex_peek() {
                        ']' => break,
                        target_c => {
                            self.or_conditions.push(target_c);
                            self.condex_next();
                        }
                    }
                }

                if self.or_conditions.is_empty() {
                    self.condex_next()
                } else {
                    ']'
                }
            }
            _ => c,
        }
    }
    #[inline]
    fn condex_peek(&mut self) -> char {
        *self.condex.peek().unwrap()
    }
    #[inline]
    fn condex_next(&mut self) -> char {
        self.condex.next().unwrap()
    }
    #[inline]
    fn reset_state(&mut self) {
        self.set_state(CondexState::Await);
    }
    #[inline]
    fn set_state(&mut self, state: CondexState) {
        self.current_state = state;
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum Token {
        TagName,
        NameType,
        Value,
        AllInOne,
    }
    impl TokenKind for Token {}

    #[test]
    fn it_works() {
        // let test = r"#-(    {-:-[=,)]-[,)]} \{";
        let mut builder = CondexBuilder::new(&[
            (Token::TagName, &["@-("]),
            (Token::NameType, &["[(,]  -  :  - [,=]"]),
            (Token::Value, &["=-[,)]"]),
            (Token::AllInOne, &["@-(", "[(,]  -  :  - [,=]", "=-[,)]"]),
        ]);

        let source = "@hello-man(name: type = value, name2: type2, name3: type3 = value3)";

        for (i, c) in source.char_indices() {
            builder.test(c, i);
        }
        let finals = builder.finalize_with_source(source);
        eprintln!("{:#?}", finals);
    }
}
