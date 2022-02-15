use std::{str::{from_utf8_unchecked, from_utf8}, cell::RefCell};
use regex::{RegexSet, Regex};
#[macro_use]
extern crate lazy_static;



#[derive(Debug, Clone, PartialEq)]
enum PrimitiveType {
    Boolean(bool),
    String(String),
    Number(i32),
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Lpar,
    Rpar,
    Semicolumn,
    Column,
    Equal,
    Literal(PrimitiveType),
}

impl Token {
    fn boolean(val: bool) -> Self {
        Token::Literal(PrimitiveType::Boolean(val))
    }

    fn number(val: i32) -> Self {
        Token::Literal(PrimitiveType::Number(val))
    }

    fn string(val: String) -> Self {
        Token::Literal(PrimitiveType::String(val))
    }
}

#[derive(PartialEq)]
enum LexerState {
    InQuotes,
    OutQuotes,
}

fn flip_state(state: LexerState) -> LexerState {
    if state == LexerState::InQuotes {
        LexerState::OutQuotes
    } else {
        LexerState::InQuotes
    }
}

struct LiteralParsing {}

struct LiteralTokenMatch {
    end: usize,
    token: Token,
}

lazy_static! {
    static ref LIT_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"true").unwrap(),
        Regex::new(r"false").unwrap(),
        Regex::new(r"\d+").unwrap(),
    ];
    static ref LIT_CONSTRUCTORS: Vec<Box<dyn Fn(&str) -> Token + Sync>> = vec![
        Box::new(|_: &str| Token::boolean(true)),
        Box::new(|_: &str| Token::boolean(false)),
        Box::new(|s: &str| Token::number(s.parse::<i32>().unwrap())),
    ];
}

impl LiteralParsing {
    fn find(text: &str) -> Option<LiteralTokenMatch> {
        for (pattern, constructor) in LIT_PATTERNS.iter().zip(LIT_CONSTRUCTORS.iter()) {
            if let Some(m) = pattern.find(text) {
                let end = m.end();
                return Some(LiteralTokenMatch { end, token: constructor(&text[..end]) });
            }
        }
        None
    }
}

// const fn initialize_litteral_patterns() -> Vec<Regex> {
//     vec![
//         Regex::new(r"true").unwrap(),
//         Regex::new(r"false").unwrap(),
//         Regex::new(r"\d+").unwrap(),
//     ]
// }
// const fn initialize_litteral_constructors() -> Vec<Box<dyn Fn(&str) -> Token>> {
//     vec![
//         Box::new(|_: &str| Token::Boolean(true)),
//         Box::new(|_: &str| Token::Boolean(false)),
//         Box::new(|s: &str| Token::Number(s.parse::<i32>().unwrap())),
//     ]
// }

// const litteral_patterns: Vec<Regex> = initialize_litteral_patterns();
// const litteral_constructors: Vec<Box<dyn Fn(&str) -> Token>> = initialize_litteral_constructors();


fn tokenize(mut cgxcode: &[u8]) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut state = LexerState::OutQuotes;

    while cgxcode.len() > 0 {
        // println!("{}", from_utf8(cgxcode).unwrap());
        match state {
            LexerState::InQuotes => {
                let pos = cgxcode.iter().position(|&b| b == b'\'').unwrap();
                if let (word, [b'\'', remaining @ ..]) = cgxcode.split_at(pos) {
                    cgxcode = remaining;
                    tokens.push(Token::string(from_utf8(word).unwrap().into()));
                } else {
                    panic!("")
                }
                state = LexerState::OutQuotes;
            }
            LexerState::OutQuotes => {
                let (token, remaining, new_state) = match cgxcode {
                    [b'(',  remaining @ ..] => (Some(Token::Lpar),  remaining, LexerState::OutQuotes),
                    [b')',  remaining @ ..] => (Some(Token::Rpar),  remaining, LexerState::OutQuotes),
                    [b';',  remaining @ ..] => (Some(Token::Semicolumn),  remaining, LexerState::OutQuotes),
                    [b':',  remaining @ ..] => (Some(Token::Column),  remaining, LexerState::OutQuotes),
                    [b'=',  remaining @ ..] => (Some(Token::Equal), remaining, LexerState::OutQuotes ),
                    [b'\'', remaining @ ..] => (None,               remaining, LexerState::InQuotes ),
                    remaining => {
                        let LiteralTokenMatch { end, token } = LiteralParsing::find(from_utf8(remaining).unwrap()).unwrap();
                        (Some(token), &remaining[end..], LexerState::OutQuotes)
                    }
                };
                if let Some(token) = token {
                    tokens.push(token);
                }
                state = new_state;
                cgxcode = remaining;
            }
        }
    }
    tokens
}

#[derive(Debug)]
enum Element {
    BLOC(Vec<Element>),
    PRIMITIVE_TYPE(PrimitiveType),
    KEY_VALUE(String, Box<Element>),
}

fn parse_primitive_type(tokens: &mut &[Token]) -> Option<Element> {
    if tokens.len() == 0 {
        return None
    }
    if let Token::Literal(lit) = &tokens[0] {
        *tokens = &mut &tokens[1..];
        Some(Element::PRIMITIVE_TYPE(lit.clone()))
    } else {
        None
    }
}

fn parse_key_value(tokens: &mut &[Token]) -> Option<Element> {
    if tokens.len() == 0 {
        return None
    }
    match tokens {
        [
            Token::Literal(PrimitiveType::String(s)),
            Token::Equal,
            ..,
        ] => {
            let mut remaining = &tokens[2..];
            if let Some(rhs) = parse_primitive_type(&mut remaining)
                .or(parse_bloc(&mut remaining))
            {
                *tokens = remaining;
                // println!("{:?}, {:?}", s, rhs);

                Some(Element::KEY_VALUE(s.clone(), Box::new(rhs)))
            } else {
                None
            }
        },
        _ => {
            None
        }
    }
    // if let Token::Literal(PrimitiveType::String(s)) = &tokens[0] {
    //     *tokens = &mut &tokens[1..];
    //     Some(Element::PRIMITIVE_TYPE(lit.clone()))
    // } else {
    //     None
    // }
}

fn parse_bloc(tokens: &mut &[Token]) -> Option<Element> {
    if tokens[0] != Token::Lpar {
        return None
    }
    let mut remaining = &tokens[1..];
    let mut elts = Vec::new();
    while let Some(elt) = parse_element(&mut remaining) {
        println!("remaining = {:?}", remaining);
        elts.push(elt);
        if remaining.len() == 0 {
            return None;
        }
        match remaining[0] {
            Token::Semicolumn => {},
            Token::Rpar => {
                *tokens = &remaining[1..];
                return Some(Element::BLOC(elts));
            },
            _ => {
                // println!("{:?}", remaining[0]);
                return None;
            }
        }
        remaining = &remaining[1..];
    }
    None
}

fn parse_element(tokens: &mut &[Token]) -> Option<Element> {
    if tokens.len() == 0 {
        return None
    }
    let mut remaining = *tokens;
    let elt = parse_bloc(&mut remaining)
        .or(parse_key_value(&mut remaining))
        .or(parse_primitive_type(&mut remaining));
    *tokens = remaining;
    elt
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let s = b"(('k1'=1);('k2'=2))";
        println!("{:?}", tokenize(s));
    }

    #[test]
    fn test_parse() {
        let s = b"(('k1'=1);('k2'=2))";
        let tokens = tokenize(s);
        let mut tokens_slice = &tokens[..];
        println!("{:?}", parse_element(&mut tokens_slice));
        println!("{:?}", tokens_slice);
    }
}
