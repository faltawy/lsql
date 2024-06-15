#[derive(Debug)]
pub enum LSQLCommand { 
    Select {
        
    },
    
    CD {
        to: String,
    },
    BACK,
}

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<lsql_tokenizer::Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<lsql_tokenizer::Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    pub fn from_tokens_str(tokens: &str) -> Self{
        let mut tokenizer = lsql_tokenizer::Tokenizer::new(tokens);
        let tokens = tokenizer.tokenize();
        Self::new(tokens)
    }

    pub fn walk(&self) -> Vec<LSQLCommand> {
        // Walk through the tokens and parse the command
        let mut commands: Vec<LSQLCommand> = Vec::new();
        for token in &self.tokens {
            match token.token_type {
                lsql_tokenizer::TokenType::Keyword => {
                    match token.literal.as_str() {
                        "SELECT" => {
                            // Parse the SELECT command
                        }
                        "BACK" => {
                            let command = LSQLCommand::BACK;
                            commands.push(command);
                        }
                        "CD" => {
                            // the path will be the next token
                            let path = self.tokens[self.position + 1].literal.clone();
                            let command = LSQLCommand::CD {
                                to: path,
                            };
                            commands.push(command);
                        }
                        _ => {
                            // Do nothing
                        }
                    }
                }

                _ => {
                    // Do nothing
                }

            }

        }
        return commands;
    }

}
