use std::io::{self, BufRead, BufReader, Read};

use crate::{error::{self, Error, LexError}, token::{Token, TokenKind}};

pub struct Lexer<'t> {
    src: Box<dyn Iterator<Item = String> + 't>,
    line: usize,
    in_string: bool,
    comment_nest_lvl: u32
}

impl<'t> Lexer<'t> {
    pub fn new<R: Read + 't>(src: R) -> Self {
        Self {
            src: Box::new(BufReader::new(src).lines().map(Result::unwrap)),
            line: 1,
            in_string: false,
            comment_nest_lvl: 0
        }
    }

    pub fn lex(&mut self) -> error::Result<Vec<Token>> {
        let mut tokens = vec![];
        
        while let Some(line) = self.src.next() {
            self.lex_line(&mut tokens, &line);

            if !self.in_string && self.comment_nest_lvl == 0 {
                self.add_token(&mut tokens, TokenKind::EOL);
            }

            self.line += 1;
        }

        self.add_token(&mut tokens, TokenKind::EOF);

        if self.in_string {
            return Err(Error::Lexer(LexError::UnclosedString));
        }

        Ok(tokens)
    }

    pub fn lex_line(&mut self, tokens: &mut Vec<Token>, line: &str) {
        let mut i = 0;
        let mut chars = line.chars();
        let mut current = chars.next();
        let mut current_token = Token::default(); // Placeholder value

        while let Some(ch) = current.clone() {
            let mut next = || {
                current = chars.next();
                i += 1;
    
                current
            };

            if ch == '\n' {
                self.line += 1;
            }

            if self.comment_nest_lvl > 0 {
                if ch == '*' {
                    if let Some('/') = next() {
                        self.comment_nest_lvl -= 1;

                        next();
                    }

                    continue;
                } else if ch == '/' {
                    if let Some('*') = next() {
                        self.comment_nest_lvl += 1;

                        next();
                    }
                } else {
                    next();
                }

                println!("cc [{}]", ch);


                continue;
            }

            if self.in_string && !['\'', '"'].contains(&ch) {
                current_token.append_to_lexeme(ch);
                next();
                continue;
                // todo escape sequences
            } 
            
            if matches!(current_token.kind(), TokenKind::Number(_)) && !(ch.is_digit(10) || ch == '_') {
                tokens.push(current_token);
                current_token = Token::default();
            } 
            
            if matches!(current_token.kind(), TokenKind::Ident(_)) && !(ch.is_alphanumeric() || ch == '_') {
                tokens.push(current_token);
                current_token = Token::default();
            }

            match ch {
                '+' => self.add_token(tokens, TokenKind::Plus),
                '-' => self.add_token(tokens, TokenKind::Minus),
                '*' => {
                    self.add_token(tokens, TokenKind::Star)
                },
                '/' => {
                    let n = next();

                    if let Some('/') = n {
                        if current_token.kind() != &TokenKind::EOL {
                            tokens.push(current_token);
                            // current_token = Token::default();
                        }

                        return;
                    }

                    if let Some('*') = n {
                        if current_token.kind() != &TokenKind::EOL {
                            tokens.push(current_token);
                            current_token = Token::default()
                        }

                        self.comment_nest_lvl += 1;

                        next();
                        continue;
                    }

                    self.add_token(tokens, TokenKind::Slash);
                    continue;
                }
                '^' => self.add_token(tokens, TokenKind::Caret),
                '=' => {
                    let n = next();

                    if let Some('=') = n {
                        self.add_token(tokens, TokenKind::DblEq);
                    } else if let Some(':') = n {
                        self.add_token(tokens, TokenKind::EqColon);
                    } else {
                        self.add_token(tokens, TokenKind::Eq);
                        continue;
                    }
                }
                '!' => {
                    if let Some('=') = next() {
                        self.add_token(tokens, TokenKind::BangEq);
                    } else {
                        self.add_token(tokens, TokenKind::Bang);
                        continue;
                    }
                }
                '~' => self.add_token(tokens, TokenKind::Tilde),
                '|' => {
                    if let Some('|') = next() {
                        self.add_token(tokens, TokenKind::DblBar);
                    } else {
                        self.add_token(tokens, TokenKind::Bar);
                        continue;
                    }
                },
                '&' => {
                    if let Some('&') = next() {
                        self.add_token(tokens, TokenKind::DblAmp);
                    } else {
                        self.add_token(tokens, TokenKind::Amp);
                        continue;
                    }
                }
                '\\' => self.add_token(tokens, TokenKind::BackSlash),
                '<' => {
                    let n = next();

                    if let Some('=') = n {
                        if let Some(':') = next() {
                            self.add_token(tokens, TokenKind::LessEqColon);
                        } else {
                            self.add_token(tokens, TokenKind::LessEq);
                            continue;
                        }
                    } else if let Some(':') = n {
                        self.add_token(tokens, TokenKind::LessColon);
                    } else {
                        self.add_token(tokens, TokenKind::Less);
                        continue;
                    }
                }
                '>' => {
                    let n = next();

                    if let Some('=') = n {
                        if let Some(':') = next() {
                            self.add_token(tokens, TokenKind::GreaterEqColon);
                        } else {
                            self.add_token(tokens, TokenKind::GreaterEq);
                            continue;
                        }
                    } else if let Some(':') = n {
                        self.add_token(tokens, TokenKind::GreaterColon);
                    } else {
                        self.add_token(tokens, TokenKind::Greater);
                        continue;
                    }
                }
                '(' => self.add_token(tokens, TokenKind::OpenParen),
                ')' => self.add_token(tokens, TokenKind::CloseParen),
                '[' => self.add_token(tokens, TokenKind::OpenBracket),
                ']' => self.add_token(tokens, TokenKind::CloseBracket),
                '{' => self.add_token(tokens, TokenKind::OpenBrace),
                '}' => self.add_token(tokens, TokenKind::CloseBrace),
                ',' => self.add_token(tokens, TokenKind::Comma),
                '.' => self.add_token(tokens, TokenKind::Dot),
                ';' => self.add_token(tokens, TokenKind::Semicolon),
                ':' => self.add_token(tokens, TokenKind::Colon),
                '#' => self.add_token(tokens, TokenKind::Hash),
                '\n' => self.add_token(tokens, TokenKind::EOL),
                '_' => match current_token.kind() {
                    TokenKind::Ident(_)  => current_token.append_to_lexeme(ch),
                    TokenKind::Number(_) => current_token.append_to_lexeme(ch),
                    TokenKind::String(_) |
                    TokenKind::Char(_)   => unreachable!(),
                    _ => current_token = Token::new(TokenKind::Ident("_".to_owned()), self.line),
                },
                '\'' => {
                    if let TokenKind::Char(_) = current_token.kind() {
                        tokens.push(current_token);
                        current_token = Token::default();
                        self.in_string = false;
                    } else {
                        current_token = Token::new(TokenKind::Char(String::new()), self.line);
                        self.in_string = true;
                    }
                }
                '"' => {
                    if let TokenKind::String(_) = current_token.kind() {
                        tokens.push(current_token);
                        current_token = Token::default();
                        self.in_string = false;
                    } else {
                        current_token = Token::new(TokenKind::String(String::new()), self.line);
                        self.in_string = true;
                    }
                },
                _ => {
                    if ch.is_digit(10) {
                        if let TokenKind::Number(_) = current_token.kind() {
                            current_token.append_to_lexeme(ch);
                        } else if let TokenKind::Ident(_) = current_token.kind() {
                            current_token.append_to_lexeme(ch);
                        } else {
                            current_token = Token::new(TokenKind::Number(String::from(ch)), self.line);
                        }
                    } else if ch.is_alphabetic() {
                        if let TokenKind::Ident(_) = current_token.kind() {
                            current_token.append_to_lexeme(ch);
                        } else  {
                            current_token = Token::new(TokenKind::Ident(String::from(ch)), self.line);
                        }
                    }
                }
            }

            next();
        }

        if current_token.kind() != &TokenKind::EOL {
            tokens.push(current_token);
        }
    }


    pub fn add_token(&self, tokens: &mut Vec<Token>, kind: TokenKind) {
        tokens.push(Token::new(kind, self.line));
    }
}
