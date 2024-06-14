#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Keyword,
    Identifier,
    Literal,
    Unknown,
    Operator,
    Parenthesis,
    Comma,
    Path,
    Dot,
}

#[derive(Copy, Clone, Debug)]
pub struct Location {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub literal: String,
    pub location: Location,
}
pub struct Tokenizer<'a> {
    input: &'a str,
    words: Vec<&'a str>,
    current_word_index: usize,
}

const KEYWORDS: [&str; 7] = ["SELECT", "FROM", "WHERE", "MOVE", "DELETE", "TO", "CD"];
const OPERATORS: &str = "=<>!";
const PUNCTUATION: &str = "(),";

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        let words: Vec<&str> = input.split_whitespace().collect();
        Tokenizer {
            input,
            words,
            current_word_index: 0,
        }
    }

    fn advance(&mut self) {
        self.current_word_index += 1;
    }

    fn peek(&self) -> Option<&'a str> {
        self.words.get(self.current_word_index + 1).copied()
    }

    fn collect_identifier(&mut self) -> &'a str {
        let word = self.words[self.current_word_index];
        self.advance();
        word
    }
    fn collect_literal(&mut self) -> &'a str {
        let word = self.words[self.current_word_index];
        self.advance();
        word
    }
    fn collect_operator(&mut self) -> &'a str {
        let word = self.words[self.current_word_index];
        self.advance();
        word
    }

    fn collect_path(&mut self) -> &'a str {
        let word = self.words[self.current_word_index];
        self.advance();
        word
    }

    fn is_keyword(&self, identifier: &'a str) -> bool {
        KEYWORDS.contains(&identifier.to_uppercase().as_str())
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
        while self.current_word_index < self.words.len() {
            let word = self.words[self.current_word_index];

            if word.is_empty() {
                self.advance();
                continue;
            }

            if word.starts_with("--") {
                // Skip comment lines starting with '--'
                self.advance();
                while self.current_word_index < self.words.len()
                    && !self.words[self.current_word_index].contains('\n')
                {
                    self.advance();
                }
                continue;
            }

            if word.chars().all(char::is_alphabetic) {
                let token_type = if self.is_keyword(word) {
                    TokenType::Keyword
                } else {
                    TokenType::Identifier
                };
                tokens.push(Token {
                    token_type,
                    location: Location {
                        start: 0,
                        end: word.len(),
                    },
                    literal: word.to_string(),
                });
                self.advance();
                continue;
            }

            if word.chars().all(char::is_numeric) || word.starts_with('\'') || word.starts_with('"')
            {
                tokens.push(Token {
                    token_type: TokenType::Literal,
                    location: Location {
                        start: 0,
                        end: word.len(),
                    },
                    literal: word.to_string(),
                });
                self.advance();
                continue;
            }

            if OPERATORS.contains(word.chars().next().unwrap()) {
                tokens.push(Token {
                    token_type: TokenType::Operator,
                    location: Location {
                        start: 0,
                        end: word.len(),
                    },
                    literal: word.to_string(),
                });
                self.advance();
                continue;
            }

            // Check for path-like tokens
            if word.contains('/') || word.contains('\\') || word.contains('.') {
                tokens.push(Token {
                    token_type: TokenType::Path,
                    location: Location {
                        start: 0,
                        end: word.len(),
                    },
                    literal: word.to_string(),
                });
                self.advance();
                continue;
            }
            if PUNCTUATION.contains(word.chars().next().unwrap()) {
                tokens.push(Token {
                    token_type: match word.chars().next().unwrap() {
                        '(' | ')' => TokenType::Parenthesis,
                        ',' => TokenType::Comma,
                        _ => TokenType::Unknown,
                    },
                    literal: word.to_string(),
                    location: Location {
                        start: 0,
                        end: word.len(),
                    },
                });
                self.advance();
                continue;
            }
            self.advance();
        }
        return tokens;
    }
}
