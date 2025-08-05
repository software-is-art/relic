use crate::ast::*;
use crate::error::{Error, ParserError, Result};
use crate::lexer::{Lexer, Token};
use crate::types::Type;

pub struct Parser {
    lexer: Lexer,
    pub current_token: Token,
    line: usize,
    column: usize,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Result<Self> {
        let current_token = lexer.next_token()?;
        Ok(Parser {
            lexer,
            current_token,
            line: 1,
            column: 1,
        })
    }

    pub fn parse_program(&mut self) -> Result<Program> {
        let mut declarations = Vec::new();

        while self.current_token != Token::Eof {
            declarations.push(self.parse_declaration()?);
        }

        Ok(Program { declarations })
    }

    fn parse_declaration(&mut self) -> Result<Declaration> {
        match &self.current_token {
            Token::Value => Ok(Declaration::Value(self.parse_value_declaration()?)),
            Token::Fn => Ok(Declaration::Function(self.parse_function_declaration()?)),
            Token::Method => {
                // Treat 'method' as an alias for 'fn' - parse it as a function
                self.advance()?; // consume 'method' token
                let name = self.expect_identifier()?;
                self.expect(Token::LeftParen)?;
                
                let mut parameters = Vec::new();
                while self.current_token != Token::RightParen {
                    // Support parameter guards for unified syntax
                    let param_with_guard = self.parse_parameter_with_guard()?;
                    // For now, convert to regular Parameter (guards will be handled later)
                    parameters.push(Parameter {
                        name: param_with_guard.name,
                        ty: param_with_guard.ty,
                    });
                    if self.current_token == Token::Comma {
                        self.advance()?;
                    } else if self.current_token != Token::RightParen {
                        return Err(Error::Parser(ParserError {
                            message: "Expected ',' or ')' after parameter".to_string(),
                            line: self.line,
                            column: self.column,
                        }));
                    }
                }
                
                self.expect(Token::RightParen)?;
                self.expect(Token::ReturnArrow)?;
                let return_type = self.parse_type()?;
                self.expect(Token::LeftBrace)?;
                let body = self.parse_expression()?;
                self.expect(Token::RightBrace)?;
                
                Ok(Declaration::Function(FunctionDeclaration {
                    name,
                    parameters,
                    return_type,
                    body,
                }))
            },
            _ => Err(Error::Parser(ParserError {
                message: format!("Expected 'value', 'fn', or 'method' keyword, found {:?}", self.current_token),
                line: self.line,
                column: self.column,
            })),
        }
    }

    fn parse_value_declaration(&mut self) -> Result<ValueDeclaration> {
        self.expect(Token::Value)?;

        let name = self.expect_identifier()?;

        self.expect(Token::LeftParen)?;
        let parameter = self.parse_parameter()?;
        self.expect(Token::RightParen)?;

        self.expect(Token::LeftBrace)?;
        let body = self.parse_value_body()?;
        self.expect(Token::RightBrace)?;

        Ok(ValueDeclaration {
            name,
            parameter,
            body,
        })
    }

    fn parse_function_declaration(&mut self) -> Result<FunctionDeclaration> {
        self.expect(Token::Fn)?;
        let name = self.expect_identifier()?;
        self.expect(Token::LeftParen)?;
        
        let mut parameters = Vec::new();
        while self.current_token != Token::RightParen {
            // Support parameter guards for unified syntax
            let param_with_guard = self.parse_parameter_with_guard()?;
            // For now, convert to regular Parameter (guards will be handled later)
            parameters.push(Parameter {
                name: param_with_guard.name,
                ty: param_with_guard.ty,
            });
            if self.current_token == Token::Comma {
                self.advance()?;
            } else if self.current_token != Token::RightParen {
                return Err(Error::Parser(ParserError {
                    message: "Expected ',' or ')' after parameter".to_string(),
                    line: self.line,
                    column: self.column,
                }));
            }
        }
        
        self.expect(Token::RightParen)?;
        self.expect(Token::ReturnArrow)?;
        let return_type = self.parse_type()?;
        self.expect(Token::LeftBrace)?;
        let body = self.parse_expression()?;
        self.expect(Token::RightBrace)?;
        
        Ok(FunctionDeclaration {
            name,
            parameters,
            return_type,
            body,
        })
    }

    fn parse_method_declaration(&mut self) -> Result<MethodDeclaration> {
        self.expect(Token::Method)?;
        let name = self.expect_identifier()?;
        self.expect(Token::LeftParen)?;
        
        let mut parameters = Vec::new();
        while self.current_token != Token::RightParen {
            parameters.push(self.parse_parameter_with_guard()?);
            if self.current_token == Token::Comma {
                self.advance()?;
            } else if self.current_token != Token::RightParen {
                return Err(Error::Parser(ParserError {
                    message: "Expected ',' or ')' after parameter".to_string(),
                    line: self.line,
                    column: self.column,
                }));
            }
        }
        
        self.expect(Token::RightParen)?;
        self.expect(Token::ReturnArrow)?;
        let return_type = self.parse_type()?;
        self.expect(Token::LeftBrace)?;
        let body = self.parse_expression()?;
        self.expect(Token::RightBrace)?;
        
        Ok(MethodDeclaration {
            name,
            parameters,
            return_type,
            body,
        })
    }

    fn parse_parameter(&mut self) -> Result<Parameter> {
        let name = self.expect_identifier()?;
        self.expect(Token::Colon)?;
        let ty = self.parse_type()?;

        Ok(Parameter { name, ty })
    }

    fn parse_parameter_with_guard(&mut self) -> Result<ParameterWithGuard> {
        let name = self.expect_identifier()?;
        self.expect(Token::Colon)?;
        let ty = self.parse_type()?;
        
        let guard = if self.current_token == Token::Where {
            self.advance()?;
            Some(self.parse_expression()?)
        } else {
            None
        };

        Ok(ParameterWithGuard { name, ty, guard })
    }

    fn parse_type(&mut self) -> Result<Type> {
        match &self.current_token {
            Token::Identifier(name) => {
                let ty = match name.as_str() {
                    "String" => Type::String,
                    "Int" => Type::Int,
                    "Bool" => Type::Bool,
                    "Any" => Type::Any,
                    _ => Type::Value(name.clone()),
                };
                self.advance()?;
                Ok(ty)
            }
            _ => Err(Error::Parser(ParserError {
                message: format!("Expected type name, found {:?}", self.current_token),
                line: self.line,
                column: self.column,
            })),
        }
    }

    fn parse_value_body(&mut self) -> Result<ValueBody> {
        let mut validate = None;
        let mut normalize = None;
        let mut unique = None;

        while self.current_token != Token::RightBrace {
            match &self.current_token {
                Token::Validate => {
                    self.advance()?;
                    self.expect(Token::Colon)?;
                    validate = Some(self.parse_expression()?);
                }
                Token::Normalize => {
                    self.advance()?;
                    self.expect(Token::Colon)?;
                    normalize = Some(self.parse_expression()?);
                }
                Token::Unique => {
                    self.advance()?;
                    self.expect(Token::Colon)?;
                    unique = Some(self.parse_boolean()?);
                }
                _ => {
                    return Err(Error::Parser(ParserError {
                        message: format!(
                            "Expected 'validate', 'normalize', or 'unique', found {:?}",
                            self.current_token
                        ),
                        line: self.line,
                        column: self.column,
                    }))
                }
            }
        }

        Ok(ValueBody {
            validate,
            normalize,
            unique,
        })
    }

    pub fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_pipeline_expression()
    }

    fn parse_pipeline_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_or_expression()?;

        while self.current_token == Token::Pipeline {
            self.advance()?;
            let right = self.parse_or_expression()?;
            left = Expression::Pipeline(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_or_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_and_expression()?;

        while self.current_token == Token::Or {
            self.advance()?;
            let right = self.parse_and_expression()?;
            left = Expression::Binary(BinaryOp::Or, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_and_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_comparison_expression()?;

        while self.current_token == Token::And {
            self.advance()?;
            let right = self.parse_comparison_expression()?;
            left = Expression::Binary(BinaryOp::And, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_comparison_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_additive_expression()?;

        loop {
            let op = match &self.current_token {
                Token::Equal => ComparisonOp::Equal,
                Token::NotEqual => ComparisonOp::NotEqual,
                Token::Less => ComparisonOp::Less,
                Token::Greater => ComparisonOp::Greater,
                Token::LessEqual => ComparisonOp::LessEqual,
                Token::GreaterEqual => ComparisonOp::GreaterEqual,
                Token::Contains => ComparisonOp::Contains,
                _ => break,
            };

            self.advance()?;
            let right = self.parse_additive_expression()?;
            left = Expression::Comparison(op, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_additive_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_multiplicative_expression()?;

        loop {
            let op = match &self.current_token {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Subtract,
                _ => break,
            };

            self.advance()?;
            let right = self.parse_multiplicative_expression()?;
            left = Expression::Binary(op, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_multiplicative_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_unary_expression()?;

        loop {
            let op = match &self.current_token {
                Token::Star => BinaryOp::Multiply,
                Token::Slash => BinaryOp::Divide,
                _ => break,
            };

            self.advance()?;
            let right = self.parse_unary_expression()?;
            left = Expression::Binary(op, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_unary_expression(&mut self) -> Result<Expression> {
        match &self.current_token {
            Token::Not => {
                self.advance()?;
                let expr = self.parse_unary_expression()?;
                Ok(Expression::Unary(UnaryOp::Not, Box::new(expr)))
            }
            Token::Minus => {
                self.advance()?;
                let expr = self.parse_unary_expression()?;
                Ok(Expression::Unary(UnaryOp::Minus, Box::new(expr)))
            }
            _ => self.parse_postfix_expression(),
        }
    }

    fn parse_postfix_expression(&mut self) -> Result<Expression> {
        let mut expr = self.parse_primary_expression()?;

        loop {
            match &self.current_token {
                Token::Dot => {
                    self.advance()?;
                    let member = self.expect_identifier()?;

                    if self.current_token == Token::LeftParen {
                        self.advance()?;
                        let mut args = Vec::new();

                        if self.current_token != Token::RightParen {
                            loop {
                                args.push(self.parse_expression()?);
                                if self.current_token == Token::Comma {
                                    self.advance()?;
                                } else {
                                    break;
                                }
                            }
                        }

                        self.expect(Token::RightParen)?;
                        expr = Expression::MethodCall(Box::new(expr), member, args);
                    } else {
                        expr = Expression::MemberAccess(Box::new(expr), member);
                    }
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary_expression(&mut self) -> Result<Expression> {
        match &self.current_token.clone() {
            Token::Let => {
                self.advance()?;
                let name = self.expect_identifier()?;
                self.expect(Token::Assign)?;
                let value = self.parse_expression()?;
                self.expect(Token::In)?;
                let body = self.parse_expression()?;
                Ok(Expression::Let(name, Box::new(value), Box::new(body)))
            }
            Token::Match => {
                self.advance()?;
                let expr = self.parse_expression()?;
                self.expect(Token::LeftBrace)?;
                
                let mut arms = Vec::new();
                while self.current_token != Token::RightBrace && self.current_token != Token::Eof {
                    arms.push(self.parse_match_arm()?);
                    
                    // Optional comma between arms
                    if self.current_token == Token::Comma {
                        self.advance()?;
                    }
                }
                
                self.expect(Token::RightBrace)?;
                Ok(Expression::Match(Box::new(expr), arms))
            }
            Token::String(s) => {
                self.advance()?;
                Ok(Expression::Literal(Literal::String(s.clone())))
            }
            Token::Integer(n) => {
                self.advance()?;
                Ok(Expression::Literal(Literal::Integer(*n)))
            }
            Token::True => {
                self.advance()?;
                Ok(Expression::Literal(Literal::Boolean(true)))
            }
            Token::False => {
                self.advance()?;
                Ok(Expression::Literal(Literal::Boolean(false)))
            }
            Token::Identifier(name) => {
                let func_name = name.clone();
                self.advance()?;
                
                // Check if this is a function call
                if self.current_token == Token::LeftParen {
                    self.advance()?;
                    let mut args = Vec::new();
                    
                    if self.current_token != Token::RightParen {
                        loop {
                            args.push(self.parse_expression()?);
                            if self.current_token == Token::Comma {
                                self.advance()?;
                            } else {
                                break;
                            }
                        }
                    }
                    
                    self.expect(Token::RightParen)?;
                    Ok(Expression::FunctionCall(func_name, args))
                } else {
                    Ok(Expression::Identifier(func_name))
                }
            }
            Token::LeftParen => {
                self.advance()?;
                let expr = self.parse_expression()?;
                self.expect(Token::RightParen)?;
                Ok(expr)
            }
            _ => Err(Error::Parser(ParserError {
                message: format!("Unexpected token in expression: {:?}", self.current_token),
                line: self.line,
                column: self.column,
            })),
        }
    }

    fn parse_match_arm(&mut self) -> Result<MatchArm> {
        // Parse pattern: ValueType(binding)
        let constructor = self.expect_identifier()?;
        self.expect(Token::LeftParen)?;
        let binding = self.expect_identifier()?;
        self.expect(Token::RightParen)?;
        
        self.expect(Token::Arrow)?;
        
        let body = self.parse_expression()?;
        
        Ok(MatchArm {
            pattern: Pattern::Constructor(constructor, binding),
            body,
        })
    }

    fn parse_boolean(&mut self) -> Result<bool> {
        match &self.current_token {
            Token::True => {
                self.advance()?;
                Ok(true)
            }
            Token::False => {
                self.advance()?;
                Ok(false)
            }
            _ => Err(Error::Parser(ParserError {
                message: format!("Expected boolean value, found {:?}", self.current_token),
                line: self.line,
                column: self.column,
            })),
        }
    }

    fn expect(&mut self, expected: Token) -> Result<()> {
        if self.current_token == expected {
            self.advance()?;
            Ok(())
        } else {
            Err(Error::Parser(ParserError {
                message: format!("Expected {:?}, found {:?}", expected, self.current_token),
                line: self.line,
                column: self.column,
            }))
        }
    }

    fn expect_identifier(&mut self) -> Result<String> {
        match &self.current_token.clone() {
            Token::Identifier(name) => {
                self.advance()?;
                Ok(name.clone())
            }
            _ => Err(Error::Parser(ParserError {
                message: format!("Expected identifier, found {:?}", self.current_token),
                line: self.line,
                column: self.column,
            })),
        }
    }

    fn advance(&mut self) -> Result<()> {
        self.current_token = self.lexer.next_token()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_expression() {
        let input = "value Transform(x: String) {
            validate: x |> toLowerCase |> length > 5
        }";

        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer).unwrap();
        let program = parser.parse_program().unwrap();

        assert_eq!(program.declarations.len(), 1);
        match &program.declarations[0] {
            Declaration::Value(v) => {
                assert_eq!(v.name, "Transform");
                assert!(v.body.validate.is_some());

                // Check that the validation expression contains a pipeline
                match v.body.validate.as_ref().unwrap() {
                    Expression::Pipeline(_, _) => {
                        // Success - found pipeline expression at top level
                        // The comparison is inside the pipeline
                    }
                    _ => panic!(
                        "Expected pipeline expression at top level, got: {:?}",
                        v.body.validate
                    ),
                }
            }
            _ => panic!("Expected value declaration")
        }
    }

    #[test]
    fn test_multiple_pipelines() {
        let input = "value Complex(s: String) {
            validate: s |> trim |> length |> isPositive
        }";

        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer).unwrap();
        let program = parser.parse_program().unwrap();

        assert_eq!(program.declarations.len(), 1);
        match &program.declarations[0] {
            Declaration::Value(v) => {
                assert_eq!(v.name, "Complex");
                assert!(v.body.validate.is_some());

                // Check for nested pipelines
                let expr = v.body.validate.as_ref().unwrap();
                match expr {
                    Expression::Pipeline(_, right) => match &**right {
                        Expression::Identifier(name) => assert_eq!(name, "isPositive"),
                        _ => panic!("Expected identifier at end of pipeline"),
                    },
                    _ => panic!("Expected pipeline expression"),
                }
            }
            _ => panic!("Expected value declaration")
        }
    }

    #[test]
    fn test_let_expression() {
        let input = "value WithLet(x: Int) {
            validate: let y = x + 10 in y > 20
        }";

        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer).unwrap();
        let program = parser.parse_program().unwrap();

        assert_eq!(program.declarations.len(), 1);
        match &program.declarations[0] {
            Declaration::Value(v) => {
                assert_eq!(v.name, "WithLet");
                assert!(v.body.validate.is_some());

                // Check that the validation expression contains a let
                match v.body.validate.as_ref().unwrap() {
                    Expression::Let(name, value, body) => {
                        assert_eq!(name, "y");
                        // Value should be x + 10
                        assert!(matches!(value.as_ref(), Expression::Binary(BinaryOp::Add, _, _)));
                        // Body should be y > 20
                        assert!(matches!(body.as_ref(), Expression::Comparison(ComparisonOp::Greater, _, _)));
                    }
                    _ => panic!(
                        "Expected let expression, got: {:?}",
                        v.body.validate
                    ),
                }
            }
            _ => panic!("Expected value declaration")
        }
    }

    #[test]
    fn test_nested_let_bindings() {
        let input = "value Temperature(celsius: Int) {
            validate: let fahrenheit = celsius * 9 / 5 + 32 in 
                      fahrenheit > -459 && fahrenheit < 1000
        }";

        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer).unwrap();
        let program = parser.parse_program().unwrap();

        assert_eq!(program.declarations.len(), 1);
        match &program.declarations[0] {
            Declaration::Value(v) => {
                assert_eq!(v.name, "Temperature");
                assert!(v.body.validate.is_some());

                // Check the let binding structure
                match v.body.validate.as_ref().unwrap() {
                    Expression::Let(name, value, body) => {
                        assert_eq!(name, "fahrenheit");
                        // Value should be a complex arithmetic expression
                        assert!(matches!(value.as_ref(), Expression::Binary(BinaryOp::Add, _, _)));
                        // Body should be an AND expression
                        assert!(matches!(body.as_ref(), Expression::Binary(BinaryOp::And, _, _)));
                    }
                    _ => panic!("Expected let expression"),
                }
            }
            _ => panic!("Expected value declaration")
        }
    }

    #[test]
    fn test_let_bindings_with_string_operations() {
        let input = "value Password(raw: String) {
            validate: let len = raw.length in
                      let hasUpperCase = raw contains \"A\" || raw contains \"B\" || raw contains \"C\" in
                      let hasNumber = raw contains \"0\" || raw contains \"1\" || raw contains \"2\" in
                      len > 8 && hasUpperCase && hasNumber
        }";

        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer).unwrap();
        let program = parser.parse_program().unwrap();

        assert_eq!(program.declarations.len(), 1);
        match &program.declarations[0] {
            Declaration::Value(v) => {
                assert_eq!(v.name, "Password");
                assert!(v.body.validate.is_some());

                // Check that it's a nested let expression
                match v.body.validate.as_ref().unwrap() {
                    Expression::Let(name1, value1, body1) => {
                        assert_eq!(name1, "len");
                        // value1 should be raw.length
                        assert!(matches!(value1.as_ref(), Expression::MemberAccess(_, _)));
                        
                        // body1 should be another let
                        match body1.as_ref() {
                            Expression::Let(name2, value2, body2) => {
                                assert_eq!(name2, "hasUpperCase");
                                // value2 should be an OR expression
                                assert!(matches!(value2.as_ref(), Expression::Binary(BinaryOp::Or, _, _)));
                                
                                // body2 should be yet another let
                                match body2.as_ref() {
                                    Expression::Let(name3, value3, body3) => {
                                        assert_eq!(name3, "hasNumber");
                                        // value3 should be an OR expression
                                        assert!(matches!(value3.as_ref(), Expression::Binary(BinaryOp::Or, _, _)));
                                        // body3 should be an AND expression
                                        assert!(matches!(body3.as_ref(), Expression::Binary(BinaryOp::And, _, _)));
                                    }
                                    _ => panic!("Expected third let expression"),
                                }
                            }
                            _ => panic!("Expected second let expression"),
                        }
                    }
                    _ => panic!("Expected let expression"),
                }
            }
            _ => panic!("Expected value declaration")
        }
    }

    #[test]
    fn test_match_expression() {
        let input = "value Process(status: Status) {
            validate: match status {
                Status(code) => code == 200
            }
        }";

        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer).unwrap();
        let program = parser.parse_program().unwrap();

        assert_eq!(program.declarations.len(), 1);
        match &program.declarations[0] {
            Declaration::Value(v) => {
                assert_eq!(v.name, "Process");
                assert!(v.body.validate.is_some());
                
                // Check that the validation expression contains a match
                match v.body.validate.as_ref().unwrap() {
                    Expression::Match(expr, arms) => {
                        // Check the matched expression
                        match expr.as_ref() {
                            Expression::Identifier(name) => assert_eq!(name, "status"),
                            _ => panic!("Expected identifier in match expression"),
                        }
                        
                        // Check we have one arm
                        assert_eq!(arms.len(), 1);
                        
                        // Check the pattern
                        match &arms[0].pattern {
                            Pattern::Constructor(name, binding) => {
                                assert_eq!(name, "Status");
                                assert_eq!(binding, "code");
                            }
                        }
                        
                        // Check the arm body
                        match &arms[0].body {
                            Expression::Comparison(ComparisonOp::Equal, left, right) => {
                                match (left.as_ref(), right.as_ref()) {
                                    (Expression::Identifier(id), Expression::Literal(Literal::Integer(200))) => {
                                        assert_eq!(id, "code");
                                    }
                                    _ => panic!("Unexpected comparison in match arm"),
                                }
                            }
                            _ => panic!("Expected comparison in match arm body"),
                        }
                    }
                    _ => panic!("Expected match expression, got: {:?}", v.body.validate),
                }
            }
            _ => panic!("Expected value declaration")
        }
    }
}
