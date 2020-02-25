use std::str::CharIndices;

fn is_symbol(ch: char) -> bool {
    match ch {
        '!' | ':' | ',' | '.' | '=' | '/' | '>' | '<' | '-' | '+' | ';' | '*' => true,
        _ => false,
    }
}

fn is_ident_start(ch: char) -> bool {
    match ch {
        'a'..='z' | 'A'..='Z' => true,
        _ => false,
    }
}

fn is_ident_continue(ch: char) -> bool {
    match ch {
        '0'..='9' | '_' => true,
        ch => is_ident_start(ch),
    }
}

fn is_dec_digit(ch: char) -> bool {
    ch.is_digit(10)
}

/// An error that occurred while lexing the source file
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    /// The location where the lexer error occured
    pub location: usize,
    /// Unexpected character
    pub char: Option<char>,
}

fn error<T>(location: usize, char: Option<char>) -> Result<T, Error> {
    Err(Error { location, char })
}

/// A token in the source file, to be emitted by the `Lexer`
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token<'input> {
    // Data
    Ident(&'input str),
    StringValue(&'input str),
    DecLiteral(i32),

    // Keywords
    If,
    Else,
    Function,

    // Data types
    I32,
    Boolean,
    String,

    // true false
    True,
    False,

    // Symbols
    Bang,         // !
    BangEqual,    // !=
    Colon,        // :
    Comma,        // ,
    Equal,        // =
    EqualEqual,   // ==
    EqualGreater, // =>
    ForwardSlash, // /
    Greater,      // >
    GreaterEqual, // >=
    Less,         // <
    LessEqual,    // <=
    Minus,        // -
    Plus,         // +
    Semi,         // ;
    Star,         // *

    // Delimiters
    LParen, // (
    RParen, // )
    LBrace, // {
    RBrace, // }
}

/// An iterator over a source string that yeilds `Token`s for subsequent use by the parser
pub struct Lexer<'input> {
    src: &'input str,
    chars: CharIndices<'input>,
    lookahead: Option<(usize, char)>,
}

impl<'input> Lexer<'input> {
    pub fn new(src: &'input str) -> Self {
        let mut chars = src.char_indices();

        Lexer {
            src,
            lookahead: chars.next(),
            chars,
        }
    }

    /// Return the next character in the source string
    fn lookahead(&self) -> Option<(usize, char)> {
        self.lookahead.map(|(index, ch)| (index, ch))
    }

    /// Bump the current position in the source string by one character,
    /// returning the current character and byte position.
    fn bump(&mut self) -> Option<(usize, char)> {
        let current = self.lookahead();
        self.lookahead = self.chars.next();
        current
    }

    /// Return a slice of the source string
    fn slice(&self, start: usize, end: usize) -> &'input str {
        &self.src[start..end]
    }

    /// Consume characters while the predicate matches for the current
    /// character, then return the consumed slice and the end byte
    /// position.
    fn take_while<F>(&mut self, start: usize, mut keep_going: F) -> (usize, &'input str)
    where
        F: FnMut(char) -> bool,
    {
        self.take_until(start, |ch| !keep_going(ch))
    }

    /// Consume characters until the predicate matches for the next character
    /// in the lookahead, then return the consumed slice and the end byte
    /// position.
    fn take_until<F>(&mut self, start: usize, mut terminate: F) -> (usize, &'input str)
    where
        F: FnMut(char) -> bool,
    {
        while let Some((end, ch)) = self.lookahead() {
            if terminate(ch) {
                return (end, self.slice(start, end));
            } else {
                self.bump();
            }
        }

        let eof = self.src.len();
        (eof, self.slice(start, eof))
    }

    /// Consume an string token
    fn string(&mut self, start: usize) -> (usize, Token<'input>, usize) {
        let (end, content) = self.take_until(start + 1, |ch| ch == '"'); // skip first '"'
        self.bump(); // skip remaining '"'
        (start, Token::StringValue(content), end)
    }

    /// Consume an identifier token
    fn ident(&mut self, start: usize) -> (usize, Token<'input>, usize) {
        let (end, ident) = self.take_while(start, is_ident_continue);
        let token = match ident {
            "if" => Token::If,
            "else" => Token::Else,
            "fn" => Token::Function,
            "i32" => Token::I32,
            "bool" => Token::Boolean,
            "String" => Token::String,
            "true" => Token::True,
            "false" => Token::False,
            ident => Token::Ident(ident),
        };
        (start, token, end)
    }
    /// Consume a decimal literal token
    fn dec_literal(&mut self, start: usize) -> (usize, Token<'input>, usize) {
        let (end, src) = self.take_while(start, is_dec_digit);
        let int = i32::from_str_radix(src, 10).unwrap();
        (start, Token::DecLiteral(int), end)
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Result<(usize, Token<'input>, usize), Error>;

    fn next(&mut self) -> Option<Result<(usize, Token<'input>, usize), Error>> {
        while let Some((start, ch)) = self.bump() {
            let end = start + 1;

            return Some(match ch {
                ch if is_symbol(ch) => {
                    let (end, symbol) = self.take_while(start, is_symbol);

                    match symbol {
                        "!" => Ok((start, Token::Bang, end)),
                        "!=" => Ok((start, Token::BangEqual, end)),
                        ":" => Ok((start, Token::Colon, end)),
                        "," => Ok((start, Token::Comma, end)),
                        "=" => Ok((start, Token::Equal, end)),
                        "==" => Ok((start, Token::EqualEqual, end)),
                        "=>" => Ok((start, Token::EqualGreater, end)),
                        "/" => Ok((start, Token::ForwardSlash, end)),
                        ">" => Ok((start, Token::Greater, end)),
                        ">=" => Ok((start, Token::GreaterEqual, end)),
                        "<" => Ok((start, Token::Less, end)),
                        "<=" => Ok((start, Token::LessEqual, end)),
                        "-" => Ok((start, Token::Minus, end)),
                        "+" => Ok((start, Token::Plus, end)),
                        ";" => Ok((start, Token::Semi, end)),
                        "*" => Ok((start, Token::Star, end)),
                        symbol if symbol.starts_with("//") => {
                            // Line comments
                            self.take_until(start, |ch| ch == '\n');
                            continue;
                        }
                        s => error(start, s.chars().nth(0)),
                    }
                }
                '(' => Ok((start, Token::LParen, end)),
                ')' => Ok((start, Token::RParen, end)),
                '{' => Ok((start, Token::LBrace, end)),
                '}' => Ok((start, Token::RBrace, end)),
                '"' => Ok(self.string(start)),
                ch if is_dec_digit(ch) => Ok(self.dec_literal(start)),
                ch if is_ident_start(ch) => Ok(self.ident(start)),
                ch if ch.is_whitespace() => continue,
                ch => error(start, Some(ch)),
            });
        }

        None
    }
}
