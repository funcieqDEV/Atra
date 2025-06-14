use crate::Node;
use crate::lexer::{SpannedToken, Token};
use crate::parser::node::Attribute;
use crate::parser::error::ParseError;

#[derive(Debug, Clone, PartialEq)]
struct BracketInfo {
    bracket_type: char,
    span: (usize, usize),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parser {
    pub tokens: Vec<SpannedToken>,
    pub current: usize,
    bracket_stack: Vec<BracketInfo>,
    in_style_block: bool,
    in_script_block: bool,
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Parser { 
            tokens, 
            current: 0, 
            bracket_stack: Vec::new(),
            in_style_block: false,
            in_script_block: false,
        }
    }

    pub fn parse(&mut self) -> Result<Node, ParseError> {
        let mut root = Node {
            atributes: vec![],
            name: String::new(),
            children: vec![],
            arguments: vec![],
            is_special_function: false,
            local_styles: vec![],
        };

        while !self.is_at_end() {
            if let Some(node) = self.parse_node()? {
                root.children.push(node);
            }
        }


        self.validate_brackets()?;

        Ok(root)
    }

    fn push_bracket(&mut self, bracket_type: char, span: (usize, usize)) {
        self.bracket_stack.push(BracketInfo { bracket_type, span });
    }

    fn pop_bracket(&mut self, expected: char, span: (usize, usize)) -> Result<(), ParseError> {
        match self.bracket_stack.pop() {
            Some(bracket_info) => {
                let expected_closing = match bracket_info.bracket_type {
                    '[' => ']',
                    '{' => '}',
                    '(' => ')',
                    other => other,
                };

                if expected_closing != expected {
                    return Err(ParseError::mismatched_brackets(
                        expected_closing,
                        expected,
                        bracket_info.span,
                        span,
                    ));
                }
                Ok(())
            }
            None => Err(ParseError::unmatched_closing_bracket(expected, span)),
        }
    }

    fn validate_brackets(&self) -> Result<(), ParseError> {
        if let Some(unclosed) = self.bracket_stack.last() {
            return Err(ParseError::unclosed_block(
                unclosed.bracket_type,
                unclosed.span,
                unclosed.span,
            ));
        }
        Ok(())
    }

    fn validate_identifier(&self, name: &str, span: (usize, usize)) -> Result<(), ParseError> {
        if name.is_empty() {
            return Err(ParseError::invalid_identifier(name.to_string(), span));
        }

        let first_char = name.chars().next().unwrap();
        if !first_char.is_alphabetic() && first_char != '_' && first_char != '$' {
            return Err(ParseError::invalid_identifier(name.to_string(), span));
        }

        for c in name.chars().skip(1) {
            if !c.is_alphanumeric() && c != '_' && c != '$' {
                return Err(ParseError::invalid_identifier(name.to_string(), span));
            }
        }

        Ok(())
    }

    fn parse_node(&mut self) -> Result<Option<Node>, ParseError> {
        let current = self.peek();

        match current {
            Some(token) => match token.token {
                Token::Ident => self.parse_tag().map(Some),
                Token::SpecialFunction => self.parse_special_function().map(Some),
                Token::StyleBlockStart => self.parse_style_block().map(Some),
                Token::ScriptBlockStart => self.parse_script_block().map(Some),
                Token::Eof => {
                    self.advance();
                    Ok(None)
                }
                _ => Err(ParseError::unexpected_token(
                    token.token,
                    (token.span.start, token.span.end),
                    None,
                )),
            },
            None => {
                let pos = if self.tokens.is_empty() { 
                    (0, 0) 
                } else { 
                    let last = &self.tokens[self.tokens.len() - 1];
                    (last.span.start, last.span.end)
                };
                Err(ParseError::unexpected_eof("token", pos))
            },
        }
    }

    fn parse_special_function(&mut self) -> Result<Node, ParseError> {
        let mut node = Node {
            atributes: vec![],
            name: String::new(),
            children: vec![],
            arguments: vec![],
            is_special_function: true,
            local_styles: vec![],
        };

        let name_token = self.consume(Token::SpecialFunction, "Expected special function name")?;
        node.name = name_token.slice.clone();

        self.consume(Token::LParen, "Expected '(' after special function name")?;


        node.arguments = self.parse_component_call_args()?;

        self.consume(Token::RParen, "Expected ')' after special function arguments")?;


        node.children = self.parse_children()?;

        Ok(node)
    }

    fn parse_tag(&mut self) -> Result<Node, ParseError> {
        let mut node = Node {
            atributes: vec![],
            name: String::new(),
            children: vec![],
            arguments: vec![],
            is_special_function: false,
            local_styles: vec![],
        };

        if let Some(_token) = self.peek() {
            let name = self.consume(Token::Ident, "Expected tag name")?;
            self.validate_identifier(&name.slice, (name.span.start, name.span.end))?;
            node.name = name.slice.clone();

            if node.name == "text" {
                self.consume(Token::LParen, "Expected '(' after 'text'")?;
                let text_value = self.consume(
                    Token::StringLiteral,
                    "Expected string literal inside 'text'",
                )?;

                if !text_value.slice.starts_with('"') || !text_value.slice.ends_with('"') || text_value.slice.len() < 2 {
                    return Err(ParseError::unclosed_string((text_value.span.start, text_value.span.end)));
                }

                node.atributes.push(Attribute {
                    name: "value".to_string(),
                    value: text_value.slice.clone(),
                });
                self.consume(Token::RParen, "Expected ')' after string literal")?;
                self.consume(Token::Semicolon, "Expected ';' after 'text'")?;
                return Ok(node);
            }

            self.consume(Token::LParen, "Expected '(' after tag name")?;


            if node.name.starts_with('$') {

                if let Some(next_token) = self.peek() {
                    match next_token.token {
                        Token::Ident => node.arguments = self.parse_component_definition_args()?,
                        Token::StringLiteral => {
                            node.arguments = self.parse_component_call_args()?
                        }
                        _ => {} 
                    }
                } else {
                    node.arguments = self.parse_component_definition_args()?;
                }
            } else {
                node.atributes = self.parse_attributes()?;
            }

            self.consume(Token::RParen, "Expected ')' after arguments/attributes")?;


            let mut parsed_children = false;
            let mut parsed_styles = false;

            while let Some(next_token) = self.peek() {
                match next_token.token {
                    Token::Semicolon => {
                        self.consume(Token::Semicolon, "Expected ';'")?;
                        return Ok(node);
                    }
                    Token::LBrace if !parsed_children => {
                        node.children = self.parse_children()?;
                        parsed_children = true;
                    }
                    Token::LBracket if !parsed_styles => {
                        node.local_styles = self.parse_local_styles()?;
                        parsed_styles = true;
                    }
                    _ => {

                        if !parsed_children && !parsed_styles {
                            return Err(ParseError::unexpected_token(
                                next_token.token,
                                (next_token.span.start, next_token.span.end),
                                Some("Expected ';', '{' for children, or '[' for local styles".to_string()),
                            ));
                        }

                        break;
                    }
                }
            }
        }

        Ok(node)
    }
    fn parse_children(&mut self) -> Result<Vec<Node>, ParseError> {
        // let brace_token = self.peek().ok_or_else(|| {
        //     let pos = if self.tokens.is_empty() { 
        //         (0, 0) 
        //     } else { 
        //         let last = &self.tokens[self.tokens.len() - 1];
        //         (last.span.start, last.span.end)
        //     };
        //     ParseError::unexpected_eof("'{' to start children block", pos)
        // })?;

        let open_brace = self.consume(Token::LBrace, "Expected '{' to add body for statement.")?;
        self.push_bracket('{', (open_brace.span.start, open_brace.span.end));

        let mut children = vec![];
        let mut found_content = false;

        while let Some(token) = self.peek() {
            match token.token {
                Token::RBrace => {
                    let close_brace = self.consume(Token::RBrace, "Expected '}' to close body.")?;
                    self.pop_bracket('}', (close_brace.span.start, close_brace.span.end))?;

                    if !found_content {
                        return Err(ParseError::empty_block("children", (open_brace.span.start, close_brace.span.end)));
                    }
                    break;
                }
                Token::Ident | Token::SpecialFunction | Token::StyleBlockStart | Token::ScriptBlockStart => {
                    if let Some(node) = self.parse_node()? {
                        children.push(node);
                        found_content = true;
                    }
                }
                _ => {
                    return Err(ParseError::invalid_token_in_context(
                        token.token,
                        "children block",
                        (token.span.start, token.span.end),
                    ));
                }
            }
        }

        if !found_content {
            return Err(ParseError::empty_block("children", (open_brace.span.start, open_brace.span.end)));
        }

        Ok(children)
    }
    fn parse_attributes(&mut self) -> Result<Vec<Attribute>, ParseError> {
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


                    if !value.starts_with('"') || !value.ends_with('"') || value.len() < 2 {
                        return Err(ParseError::unclosed_string((value_token.span.start, value_token.span.end)));
                    }

                    attributes.push(Attribute { name, value });
                }
                Token::Comma => {
                    self.advance();
                }
                Token::RParen => {
                    break;
                }
                _ => {
                    return Err(ParseError::unexpected_token(
                        token.token,
                        (token.span.start, token.span.end),
                        None,
                    ));
                }
            }
        }

        Ok(attributes)
    }

    fn parse_component_definition_args(&mut self) -> Result<Vec<String>, ParseError> {
        let mut args = vec![];

        while let Some(token) = self.peek() {
            match token.token {
                Token::Ident => {
                    let arg = self.consume(Token::Ident, "Expected argument name")?;
                    args.push(arg.slice);
                }
                Token::Comma => {
                    self.advance();
                }
                Token::RParen => {
                    break;
                }
                _ => {
                    return Err(ParseError::unexpected_token(
                        token.token,
                        (token.span.start, token.span.end),
                        None,
                    ));
                }
            }
        }

        Ok(args)
    }

    fn parse_component_call_args(&mut self) -> Result<Vec<String>, ParseError> {
        let mut args = vec![];

        while let Some(token) = self.peek() {
            match token.token {
                Token::Ident => {
                    let arg = self.consume(Token::Ident, "Expected argument value")?;
                    args.push(arg.slice);
                }
                Token::StringLiteral => {
                    let arg = self.consume(Token::StringLiteral, "Expected string argument")?;


                    if !arg.slice.starts_with('"') || !arg.slice.ends_with('"') || arg.slice.len() < 2 {
                        return Err(ParseError::unclosed_string((arg.span.start, arg.span.end)));
                    }

                    args.push(arg.slice);
                }
                Token::Number => {
                    let arg = self.consume(Token::Number, "Expected number argument")?;
                    args.push(arg.slice);
                }
                Token::Comma => {
                    self.advance();
                }
                Token::RParen => {
                    break;
                }
                _ => {
                    return Err(ParseError::unexpected_token(
                        token.token,
                        (token.span.start, token.span.end),
                        None,
                    ));
                }
            }
        }

        Ok(args)
    }
    fn parse_style_block(&mut self) -> Result<Node, ParseError> {
        if self.in_style_block {
            let token = self.peek().unwrap();
            return Err(ParseError::nested_blocks_not_allowed("style", (token.span.start, token.span.end)));
        }

        self.in_style_block = true;
        let start_token = self.consume(Token::StyleBlockStart, "Expected '@[' to start style block")?;
        self.push_bracket('[', (start_token.span.start, start_token.span.end));

        let mut node = Node {
            atributes: vec![],
            name: "style".to_string(),
            children: vec![],
            arguments: vec![],
            is_special_function: false,
            local_styles: vec![],
        };


        let mut css_content = String::new();
        let mut brace_depth = 0;

        while let Some(token) = self.peek() {
            match token.token {
                Token::RBracket if brace_depth == 0 => {
                    let close_token = self.consume(Token::RBracket, "Expected ']' to close style block")?;
                    self.pop_bracket(']', (close_token.span.start, close_token.span.end))?;
                    self.in_style_block = false;
                    break;
                }
                Token::LBrace => {
                    brace_depth += 1;
                    css_content.push('{');
                    self.advance();
                }
                Token::RBrace => {
                    brace_depth -= 1;
                    css_content.push('}');
                    self.advance();
                }
                Token::Dot => {
                    css_content.push('.');
                    self.advance();
                }
                Token::Asterisk => {
                    css_content.push('*');
                    self.advance();
                }
                Token::Hash => {
                    css_content.push('#');
                    self.advance();
                }
                Token::Greater => {
                    css_content.push('>');
                    self.advance();
                }
                Token::Plus => {
                    css_content.push('+');
                    self.advance();
                }
                Token::Tilde => {
                    css_content.push('~');
                    self.advance();
                }
                Token::SingleQuote => {
                    css_content.push('\'');
                    self.advance();
                }
                Token::Minus => {
                    css_content.push('-');
                    self.advance();
                }
                _ => {
                    if !css_content.is_empty() && !css_content.ends_with(' ') && !css_content.ends_with('\n') && !css_content.ends_with('.') {
                        css_content.push(' ');
                    }
                    css_content.push_str(&token.slice);
                    self.advance();
                }
            }
        }

        node.atributes.push(Attribute {
            name: "content".to_string(),
            value: css_content.trim().to_string(),
        });

        Ok(node)
    }

    fn parse_script_block(&mut self) -> Result<Node, ParseError> {
        if self.in_script_block {
            let token = self.peek().unwrap();
            return Err(ParseError::nested_blocks_not_allowed("script", (token.span.start, token.span.end)));
        }

        self.in_script_block = true;
        let start_token = self.consume(Token::ScriptBlockStart, "Expected '&[' to start script block")?;
        self.push_bracket('[', (start_token.span.start, start_token.span.end));

        let mut node = Node {
            atributes: vec![],
            name: "script".to_string(),
            children: vec![],
            arguments: vec![],
            is_special_function: false,
            local_styles: vec![],
        };

    
        let mut js_content = String::new();        let mut brace_depth = 0;

        while let Some(token) = self.peek() {
            match token.token {
                Token::RBracket if brace_depth == 0 => {
                    let close_token = self.consume(Token::RBracket, "Expected ']' to close script block")?;
                    self.pop_bracket(']', (close_token.span.start, close_token.span.end))?;
                    self.in_script_block = false;
                    break;
                }
                Token::LBrace => {
                    brace_depth += 1;
                    js_content.push('{');
                    self.advance();
                }
                Token::RBrace => {
                    brace_depth -= 1;
                    js_content.push('}');
                    self.advance();
                }
                Token::Dot => {
                    js_content.push('.');
                    self.advance();
                }
                Token::StringLiteral => {
      
                    if !js_content.is_empty() && !js_content.ends_with(' ') && !js_content.ends_with('\n') && !js_content.ends_with('.') {
                        js_content.push(' ');
                    }
                    js_content.push_str(&token.slice);
                    self.advance();
                }
                _ => {
                    if !js_content.is_empty() && !js_content.ends_with(' ') && !js_content.ends_with('\n') && !js_content.ends_with('.') {
                        js_content.push(' ');
                    }
                    js_content.push_str(&token.slice);
                    self.advance();
                }
            }
        }


        node.atributes.push(Attribute {
            name: "content".to_string(),
            value: js_content.trim().to_string(),
        });

        Ok(node)
    }

    fn parse_local_styles(&mut self) -> Result<Vec<Attribute>, ParseError> {
        let open_bracket = self.consume(Token::LBracket, "Expected '[' to start local styles")?;
        self.push_bracket('[', (open_bracket.span.start, open_bracket.span.end));
        let mut styles = vec![];

        while let Some(token) = self.peek() {
            match token.token {
                Token::Ident | Token::CodeBlock => {
                    let property_token = if token.token == Token::CodeBlock {
                        self.consume(Token::CodeBlock, "Expected CSS property name")?
                    } else {
                        self.consume(Token::Ident, "Expected CSS property name")?
                    };
                    let property = property_token.slice;

                    self.consume(Token::Colon, "Expected ':' after CSS property")?;

                    let value = match self.peek() {
                        Some(t) => match t.token {
                            Token::Ident => {
                                let val_token = self.consume(Token::Ident, "Expected CSS value")?;
                                val_token.slice
                            }
                            Token::CodeBlock => {
                                let val_token = self.consume(Token::CodeBlock, "Expected CSS value")?;
                                val_token.slice
                            }
                            Token::Number => {
             
                                let num_token = self.consume(Token::Number, "Expected CSS value")?;
                                let mut value = num_token.slice;

                
                                if let Some(next_token) = self.peek() {
                                    if matches!(next_token.token, Token::Ident | Token::CodeBlock) {
                                        let unit_token = if next_token.token == Token::CodeBlock {
                                            self.consume(Token::CodeBlock, "Expected unit")?
                                        } else {
                                            self.consume(Token::Ident, "Expected unit")?
                                        };
                                        value.push_str(&unit_token.slice);
                                    }
                                }
                                value
                            }
                            Token::StringLiteral => {
                                let val_token = self.consume(Token::StringLiteral, "Expected CSS value")?;

                                if !val_token.slice.starts_with('"') || !val_token.slice.ends_with('"') || val_token.slice.len() < 2 {
                                    return Err(ParseError::unclosed_string((val_token.span.start, val_token.span.end)));
                                }

                                val_token.slice
                            }
                            _ => {
                                let pos = if self.tokens.is_empty() { 
                                    (0, 0) 
                                } else { 
                                    let last = &self.tokens[self.tokens.len() - 1];
                                    (last.span.start, last.span.end)
                                };
                                return Err(ParseError::unexpected_eof("CSS value", pos));
                            },
                        }
                        None => {
                            let pos = if self.tokens.is_empty() { 
                                (0, 0) 
                            } else { 
                                let last = &self.tokens[self.tokens.len() - 1];
                                (last.span.start, last.span.end)
                            };
                            return Err(ParseError::unexpected_eof("CSS value", pos));
                        },
                    };

                    styles.push(Attribute {
                        name: property,
                        value,
                    });

                    if let Some(next_token) = self.peek() {
                        if next_token.token == Token::Semicolon {
                            self.advance();
                        }
                    }
                }
                Token::RBracket => {
                    let close_bracket = self.consume(Token::RBracket, "Expected ']' to close local styles")?;
                    self.pop_bracket(']', (close_bracket.span.start, close_bracket.span.end))?;
                    break;
                }
                Token::Semicolon => {
                    self.advance(); 
                }
                _ => {
                    return Err(ParseError::unexpected_token(
                        token.token,
                        (token.span.start, token.span.end),
                        None,
                    ));
                }
            }
        }

        Ok(styles)
    }

    #[allow(dead_code)]
    fn substitute_in_string(text: &str, params: &[String], args: &[String]) -> String {
        let mut result = text.to_string();

        for (i, param) in params.iter().enumerate() {
            if let Some(arg) = args.get(i) {

                let clean_arg = arg.trim_matches('"');
                result = result.replace(&format!("{{{}}}", param), clean_arg);
            }
        }

        result
    }

    // fn is_component_call(&self) -> bool {

    //     if let Some(token) = self.peek() {
    //         return token.token == Token::Ident || token.token == Token::StringLiteral;
    //     }
    //     false
    // }

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

    fn consume(&mut self, expected: Token, _err_mess: &str) -> Result<SpannedToken, ParseError> {
        if self.is_at_end() {
            let pos = if self.tokens.is_empty() { 
                (0, 0) 
            } else { 
                let last = &self.tokens[self.tokens.len() - 1];
                (last.span.start, last.span.end)
            };
            return Err(ParseError::unexpected_eof(&format!("{:?}", expected), pos));
        }

        let current = &self.tokens[self.current];
        if current.token == expected {
            self.advance();
            Ok(self.tokens[self.current - 1].clone())
        } else {
            Err(ParseError::expected_token(
                format!("{:?}", expected),
                current.token.clone(),
                (current.span.start, current.span.end),
            ))
        }
    }
}