use crate::Node;
use crate::lexer::{SpannedToken, Token};
use crate::parser::node::Attribute;
#[derive(Debug, Clone, PartialEq)]

pub struct Parser {
    pub tokens: Vec<SpannedToken>,
    pub current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Node, String> {
        let mut root = Node {
            atributes: vec![],
            name: String::new(),
            children: vec![],
        };
        while !self.is_at_end() {
            if let Some(node) = self.parse_node()? {
                root.children.push(node);
            } else {
            }
        }
        Ok(root)
    }

    fn parse_node(&mut self) -> Result<Option<Node>, String> {
        let current = self.peek();

        match current {
            Some(token) => match token.token {
                Token::Ident => self.parse_tag().map(Some),
                Token::Eof => {
                    self.advance();
                    Ok(None)
                }
                _ => Err(format!(
                    "Unexpected token {:?} at line {}, column {}",
                    token.token, token.line, token.column
                )),
            },
            None => Err("Unexpected end of input".to_string()),
        }
    }

    fn parse_tag(&mut self) -> Result<Node, String> {
        let mut node = Node {
            atributes: vec![],
            name: String::new(),
            children: vec![],
        };

        if let Some(_token) = self.peek() {
            let name = self.consume(Token::Ident, "Expected tag name")?;
            node.name = name.slice.clone();

            if node.name == "text" {
                self.consume(Token::LParen, "Expected '(' after 'text'")?;
                let text_value = self.consume(
                    Token::StringLiteral,
                    "Expected string literal inside 'text'",
                )?;
                node.atributes.push(Attribute {
                    name: "value".to_string(),
                    value: text_value.slice.clone(),
                });
                self.consume(Token::RParen, "Expected ')' after string literal")?;
                self.consume(Token::Semicolon, "Expected ';' after 'text'")?;
                return Ok(node);
            }

            self.consume(Token::LParen, "Expected '(' after tag name")?;
            node.atributes = self.parse_attributes()?;
            self.consume(Token::RParen, "Expected ')' after tag attributes")?;

            if let Some(next_token) = self.peek() {
                match next_token.token {
                    Token::Semicolon => {
                        self.consume(Token::Semicolon, "Expected ';' after tag attributes")?;
                        return Ok(node);
                    }
                    Token::LBrace => {
                        node.children = self.parse_children()?;
                    }
                    _ => {
                        return Err(format!(
                            "Unexpected token {:?} after tag attributes at line {}, column {}",
                            next_token.token, next_token.line, next_token.column
                        ));
                    }
                }
            }
        }

        Ok(node)
    }
    fn parse_children(&mut self) -> Result<Vec<Node>, String> {
        self.consume(Token::LBrace, "Expected '{' to add body for statement.")?;
        let mut children = vec![];
        while let Some(token) = self.peek() {
            match token.token {
                Token::RBrace => {
                    self.consume(Token::RBrace, "Expected '}' to close body.")?;
                    break;
                }
                Token::Ident => {
                    if let Some(node) = self.parse_node()? {
                        children.push(node);
                    }
                }
                _ => {
                    return Err(format!(
                        "Unexpected token {:?} at line {}, column {}",
                        token.token, token.line, token.column
                    ));
                }
            }
        }
        Ok(children)
    }
    fn parse_attributes(&mut self) -> Result<Vec<Attribute>, String> {
        let mut attributes = vec![];

        while let Some(token) = self.peek() {
            match token.token {
                Token::Ident => {
                    let name_token = self.consume(Token::Ident, "Expected attribute name")?;
                    let name = name_token.slice;

                    self.consume(Token::Equals, "Expected '=' after attribute name")?;

                    let value_token = self.consume(
                        Token::StringLiteral,
                        "Expected string literal as attribute value",
                    )?;
                    let value = value_token.slice;

                    attributes.push(Attribute { name, value });
                }
                Token::Comma => {
                    self.advance();
                }
                Token::RParen => {
                    break;
                }
                _ => {
                    return Err(format!(
                        "Unexpected token {:?} at line {}, column {}",
                        token.token, token.line, token.column
                    ));
                }
            }
        }

        Ok(attributes)
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.current += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn peek(&self) -> Option<SpannedToken> {
        if self.is_at_end() {
            None
        } else {
            Some(self.tokens[self.current].clone())
        }
    }

    fn consume(&mut self, expected: Token, err_mess: &str) -> Result<SpannedToken, String> {
        if self.is_at_end() {
            return Err("Unexpected end of input".to_string());
        }

        let current = &self.tokens[self.current];
        if current.token == expected {
            self.advance();
            Ok(self.tokens[self.current - 1].clone())
        } else {
            Err(format!(
                "{}: expected {:?}, found {:?} at line {}, column {}",
                err_mess, expected, current.token, current.line, current.column
            ))
        }
    }
}
