use apl_core::{
    Assignment, AssignmentOperator, BinaryOperator, ElseIfBranch, Expression, FunctionDecl,
    IfStatement, InfoAssignment, ListDecl, ListElement, PickStatement, Program, ProtectionLevel,
    Statement, UnaryOperator, VTimeDecl, Value, ValueType, VariableDecl, WhileStatement,
};

pub fn parse_program(source: &str) -> Result<Program, ParseError> {
    let tokenized = tokenize(source)?;
    let mut parser = Parser {
        tokens: tokenized.tokens,
        offsets: tokenized.offsets,
        source,
        cursor: 0,
    };
    parser.parse_program()
}

struct Parser<'a> {
    tokens: Vec<Token>,
    offsets: Vec<usize>,
    source: &'a str,
    cursor: usize,
}

impl Parser<'_> {
    fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut statements = Vec::new();

        while !self.is_done() {
            statements.push(self.parse_statement()?);
        }

        Ok(Program::new(statements))
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        if self.check_keyword("if") {
            return self.parse_if().map(Statement::If);
        }

        if self.check_keyword("func") {
            return self.parse_function().map(Statement::FunctionDecl);
        }

        if self.check_keyword("while") {
            return self.parse_while().map(Statement::While);
        }

        if self.check_keyword("pick") {
            return self.parse_pick().map(Statement::Pick);
        }

        if self.eat_keyword("List") {
            let name = self.expect_ident()?;
            self.expect_symbol(Symbol::Assign)?;
            let elements = self.parse_list_literal_elements()?;
            return Ok(Statement::ListDecl(ListDecl { name, elements }));
        }

        if self.check_ident("add") {
            return self.parse_add_to_list();
        }

        if self.eat_keyword("break") {
            return Ok(Statement::Break);
        }

        if self.eat_keyword("continue") {
            return Ok(Statement::Continue);
        }

        if self.eat_keyword("return") {
            return self.parse_expression().map(Statement::Return);
        }

        if self.eat_keyword("secretup") {
            self.expect_symbol(Symbol::LeftParen)?;
            let name = self.expect_ident()?;
            self.expect_symbol(Symbol::RightParen)?;
            return Ok(Statement::SecretUp(name));
        }

        if self.eat_keyword("stop") {
            let reason = if self.next_starts_statement_or_block_end() {
                None
            } else {
                Some(self.parse_expression()?)
            };
            return Ok(Statement::Stop(reason));
        }

        if self.eat_keyword("fail") {
            return self.parse_expression().map(Statement::Fail);
        }

        if self.eat_keyword("out") {
            return self.parse_expression().map(Statement::Out);
        }

        if self.eat_keyword("VTime") {
            let name = self.expect_ident()?;
            self.expect_symbol(Symbol::Assign)?;
            let initial_value = self.parse_expression()?;
            return Ok(Statement::VTimeDecl(VTimeDecl {
                name,
                initial_value,
            }));
        }

        if let Some((kind, value_type)) = self.peek_decl_keyword() {
            self.next();
            let name = self.expect_ident()?;
            self.expect_symbol(Symbol::Assign)?;
            let initial_value = self.parse_expression()?;

            return Ok(Statement::VariableDecl(VariableDecl {
                kind,
                value_type,
                name,
                initial_value,
            }));
        }

        if self.looks_like_info_assignment() {
            return self.parse_info_assignment().map(Statement::InfoAssignment);
        }

        self.parse_assignment().map(Statement::Assignment)
    }

    fn parse_function(&mut self) -> Result<FunctionDecl, ParseError> {
        self.expect_keyword("func")?;
        let name = self.expect_ident()?;
        self.expect_symbol(Symbol::LeftParen)?;
        let mut params = Vec::new();

        if !self.check_symbol(Symbol::RightParen) {
            loop {
                params.push(self.expect_ident()?);
                if !self.eat_symbol(Symbol::Comma) {
                    break;
                }
            }
        }

        self.expect_symbol(Symbol::RightParen)?;
        let body = self.parse_block()?;

        Ok(FunctionDecl { name, params, body })
    }

    fn parse_while(&mut self) -> Result<WhileStatement, ParseError> {
        self.expect_keyword("while")?;
        self.expect_symbol(Symbol::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect_symbol(Symbol::RightParen)?;
        self.expect_symbol(Symbol::LeftParen)?;
        let limit = self.expect_int_literal()?;
        self.expect_symbol(Symbol::RightParen)?;
        let body = self.parse_block()?;

        Ok(WhileStatement {
            condition,
            limit,
            body,
        })
    }

    fn parse_pick(&mut self) -> Result<PickStatement, ParseError> {
        self.expect_keyword("pick")?;
        self.expect_symbol(Symbol::LeftParen)?;
        let value = self.parse_expression()?;
        self.expect_symbol(Symbol::RightParen)?;
        self.expect_symbol(Symbol::Colon)?;
        let item_name = self.expect_ident()?;
        let body = self.parse_block()?;

        Ok(PickStatement {
            value,
            item_name,
            body,
        })
    }

    fn looks_like_info_assignment(&self) -> bool {
        matches!(
            (
                self.tokens.get(self.cursor),
                self.tokens.get(self.cursor + 1),
                self.tokens.get(self.cursor + 2),
                self.tokens.get(self.cursor + 3),
            ),
            (
                Some(Token::Ident(_)),
                Some(Token::Symbol(Symbol::Comma)),
                Some(Token::Ident(_)),
                Some(Token::Symbol(Symbol::Assign)),
            )
        )
    }

    fn parse_info_assignment(&mut self) -> Result<InfoAssignment, ParseError> {
        let type_target = self.expect_ident()?;
        self.expect_symbol(Symbol::Comma)?;
        let protection_target = self.expect_ident()?;
        self.expect_symbol(Symbol::Assign)?;
        self.expect_keyword("info")?;
        self.expect_symbol(Symbol::LeftParen)?;
        let source = self.expect_ident()?;
        self.expect_symbol(Symbol::RightParen)?;

        Ok(InfoAssignment {
            type_target,
            protection_target,
            source,
        })
    }

    fn parse_add_to_list(&mut self) -> Result<Statement, ParseError> {
        self.expect_ident()?;
        self.expect_symbol(Symbol::LeftParen)?;
        let list_name = self.expect_ident()?;
        self.expect_symbol(Symbol::Comma)?;
        let element = self.parse_list_element()?;
        self.expect_symbol(Symbol::RightParen)?;

        Ok(Statement::AddToList { list_name, element })
    }

    fn parse_if(&mut self) -> Result<IfStatement, ParseError> {
        self.expect_keyword("if")?;
        let condition = self.parse_expression()?;
        let body = self.parse_block()?;
        let mut else_if_branches = Vec::new();
        let mut else_body = None;

        while self.eat_keyword("else") {
            if self.eat_keyword("if") {
                let condition = self.parse_expression()?;
                let body = self.parse_block()?;
                else_if_branches.push(ElseIfBranch { condition, body });
            } else {
                else_body = Some(self.parse_block()?);
                break;
            }
        }

        Ok(IfStatement {
            condition,
            body,
            else_if_branches,
            else_body,
        })
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>, ParseError> {
        self.expect_symbol(Symbol::LeftBrace)?;
        let mut statements = Vec::new();

        while !self.eat_symbol(Symbol::RightBrace) {
            if self.is_done() {
                return Err(self.error("expected `}`"));
            }
            statements.push(self.parse_statement()?);
        }

        Ok(statements)
    }

    fn parse_assignment(&mut self) -> Result<Assignment, ParseError> {
        let name = self.expect_ident()?;
        let operator = match self.next() {
            Some(Token::Symbol(Symbol::Assign)) => AssignmentOperator::Assign,
            Some(Token::Symbol(Symbol::AddAssign)) => AssignmentOperator::AddAssign,
            Some(Token::Symbol(Symbol::SubAssign)) => AssignmentOperator::SubAssign,
            Some(Token::Symbol(Symbol::MulAssign)) => AssignmentOperator::MulAssign,
            Some(Token::Symbol(Symbol::DivAssign)) => AssignmentOperator::DivAssign,
            Some(_) => return Err(self.error_at_previous("expected assignment operator")),
            None => return Err(self.error("expected assignment operator")),
        };
        let value = self.parse_expression()?;

        Ok(Assignment {
            name,
            operator,
            value,
        })
    }

    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expression, ParseError> {
        let mut expression = self.parse_and()?;

        while self.eat_keyword("or") {
            self.require_grouped_logic_operand(&expression)?;
            let right = self.parse_and()?;
            self.require_grouped_logic_operand(&right)?;
            expression = Expression::Binary {
                left: Box::new(expression),
                operator: BinaryOperator::Or,
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    fn parse_and(&mut self) -> Result<Expression, ParseError> {
        let mut expression = self.parse_comparison()?;

        while self.eat_keyword("and") {
            self.require_grouped_logic_operand(&expression)?;
            let right = self.parse_comparison()?;
            self.require_grouped_logic_operand(&right)?;
            expression = Expression::Binary {
                left: Box::new(expression),
                operator: BinaryOperator::And,
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    fn require_grouped_logic_operand(&self, expression: &Expression) -> Result<(), ParseError> {
        match expression {
            Expression::Grouped(_)
            | Expression::Variable(_)
            | Expression::Literal(Value::Bool(_)) => Ok(()),
            _ => {
                Err(self.error("logical operands with `and`/`or` must be grouped with parentheses"))
            }
        }
    }

    fn parse_comparison(&mut self) -> Result<Expression, ParseError> {
        let left = self.parse_term()?;

        let Some(operator) = self.peek_comparison_operator() else {
            return Ok(left);
        };

        self.next();
        let right = self.parse_term()?;

        Ok(Expression::Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }

    fn parse_term(&mut self) -> Result<Expression, ParseError> {
        let mut expression = self.parse_factor()?;

        loop {
            let operator = if self.eat_symbol(Symbol::Plus) {
                BinaryOperator::Add
            } else if self.eat_symbol(Symbol::Minus) {
                BinaryOperator::Sub
            } else {
                break;
            };

            let right = self.parse_factor()?;
            expression = Expression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    fn parse_factor(&mut self) -> Result<Expression, ParseError> {
        let mut expression = self.parse_unary()?;

        loop {
            let operator = if self.eat_symbol(Symbol::Star) {
                BinaryOperator::Mul
            } else if self.eat_symbol(Symbol::Slash) {
                BinaryOperator::Div
            } else {
                break;
            };

            let right = self.parse_unary()?;
            expression = Expression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    fn parse_unary(&mut self) -> Result<Expression, ParseError> {
        if self.eat_symbol(Symbol::Minus) {
            return Ok(Expression::Unary {
                operator: UnaryOperator::Negate,
                expression: Box::new(self.parse_unary()?),
            });
        }

        if self.eat_keyword("not") {
            return Ok(Expression::Unary {
                operator: UnaryOperator::Not,
                expression: Box::new(self.parse_unary()?),
            });
        }

        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expression, ParseError> {
        let mut expression = self.parse_primary()?;

        while self.eat_symbol(Symbol::LeftBracket) {
            expression = self.parse_index_or_slice(expression)?;
        }

        Ok(expression)
    }

    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        match self.next() {
            Some(Token::Int(value)) => Ok(Expression::Literal(Value::Int(value))),
            Some(Token::Float(value)) => Ok(Expression::Literal(Value::Float(value))),
            Some(Token::Str(value)) => Ok(Expression::Literal(Value::Str(value))),
            Some(Token::Keyword(value)) if value == "true" => {
                Ok(Expression::Literal(Value::Bool(true)))
            }
            Some(Token::Keyword(value)) if value == "false" => {
                Ok(Expression::Literal(Value::Bool(false)))
            }
            Some(Token::Keyword(value)) if value == "NONE" => Ok(Expression::None),
            Some(Token::Keyword(value)) if value == "input" => Ok(Expression::Input),
            Some(Token::Keyword(value)) if value == "secret" => {
                self.expect_keyword("input")?;
                Ok(Expression::SecretInput)
            }
            Some(Token::Symbol(Symbol::LeftBracket)) => {
                self.cursor -= 1;
                self.parse_list_literal_elements()
                    .map(Expression::ListLiteral)
            }
            Some(Token::Ident(name)) => {
                if self.eat_symbol(Symbol::SelfCheck) {
                    Ok(Expression::SelfCheck(name))
                } else if self.eat_symbol(Symbol::LeftParen) {
                    let args = self.parse_call_args()?;
                    Ok(Expression::Call { name, args })
                } else {
                    Ok(Expression::Variable(name))
                }
            }
            Some(Token::Symbol(Symbol::LeftParen)) => {
                let expression = self.parse_expression()?;
                self.expect_symbol(Symbol::RightParen)?;
                Ok(Expression::Grouped(Box::new(expression)))
            }
            Some(_) => Err(self.error_at_previous("expected expression")),
            None => Err(self.error("expected expression")),
        }
    }

    fn parse_index_or_slice(&mut self, target: Expression) -> Result<Expression, ParseError> {
        if self.eat_symbol(Symbol::Colon) {
            let end = if self.check_symbol(Symbol::Colon) || self.check_symbol(Symbol::RightBracket)
            {
                None
            } else {
                Some(Box::new(self.parse_expression()?))
            };
            let step = if self.eat_symbol(Symbol::Colon) {
                if self.check_symbol(Symbol::RightBracket) {
                    None
                } else {
                    Some(Box::new(self.parse_expression()?))
                }
            } else {
                None
            };
            self.expect_symbol(Symbol::RightBracket)?;
            return Ok(Expression::Slice {
                target: Box::new(target),
                start: None,
                end,
                step,
            });
        }

        let first = self.parse_expression()?;

        if !self.eat_symbol(Symbol::Colon) {
            self.expect_symbol(Symbol::RightBracket)?;
            return Ok(Expression::Index {
                target: Box::new(target),
                index: Box::new(first),
            });
        }

        let end = if self.check_symbol(Symbol::Colon) || self.check_symbol(Symbol::RightBracket) {
            None
        } else {
            Some(Box::new(self.parse_expression()?))
        };
        let step = if self.eat_symbol(Symbol::Colon) {
            if self.check_symbol(Symbol::RightBracket) {
                None
            } else {
                Some(Box::new(self.parse_expression()?))
            }
        } else {
            None
        };
        self.expect_symbol(Symbol::RightBracket)?;

        Ok(Expression::Slice {
            target: Box::new(target),
            start: Some(Box::new(first)),
            end,
            step,
        })
    }

    fn parse_list_literal_elements(&mut self) -> Result<Vec<ListElement>, ParseError> {
        self.expect_symbol(Symbol::LeftBracket)?;
        let mut elements = Vec::new();

        if self.eat_symbol(Symbol::RightBracket) {
            return Ok(elements);
        }

        loop {
            elements.push(self.parse_list_element()?);
            if self.eat_symbol(Symbol::Comma) {
                continue;
            }

            self.expect_symbol(Symbol::RightBracket)?;
            break;
        }

        Ok(elements)
    }

    fn parse_list_element(&mut self) -> Result<ListElement, ParseError> {
        let value = self.parse_expression()?;
        let protection = if self.eat_symbol(Symbol::Colon) {
            Some(self.expect_protection_level()?)
        } else {
            None
        };

        Ok(ListElement { value, protection })
    }

    fn parse_call_args(&mut self) -> Result<Vec<Expression>, ParseError> {
        let mut args = Vec::new();

        if self.eat_symbol(Symbol::RightParen) {
            return Ok(args);
        }

        loop {
            args.push(self.parse_expression()?);
            if self.eat_symbol(Symbol::Comma) {
                continue;
            }

            self.expect_symbol(Symbol::RightParen)?;
            break;
        }

        Ok(args)
    }

    fn peek_comparison_operator(&self) -> Option<BinaryOperator> {
        match self.peek() {
            Some(Token::Symbol(Symbol::EqualEqual)) => Some(BinaryOperator::Equal),
            Some(Token::Symbol(Symbol::NotEqual)) => Some(BinaryOperator::NotEqual),
            Some(Token::Symbol(Symbol::Greater)) => Some(BinaryOperator::Greater),
            Some(Token::Symbol(Symbol::GreaterEqual)) => Some(BinaryOperator::GreaterEqual),
            Some(Token::Symbol(Symbol::Less)) => Some(BinaryOperator::Less),
            Some(Token::Symbol(Symbol::LessEqual)) => Some(BinaryOperator::LessEqual),
            _ => None,
        }
    }

    fn peek_decl_keyword(&self) -> Option<(apl_core::VariableKind, ValueType)> {
        match self.peek() {
            Some(Token::Keyword(value)) => ValueType::parse_decl_keyword(value),
            _ => None,
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        match self.next() {
            Some(Token::Ident(value)) => Ok(value),
            Some(_) => Err(self.error_at_previous("expected identifier")),
            None => Err(self.error("expected identifier")),
        }
    }

    fn expect_int_literal(&mut self) -> Result<i64, ParseError> {
        if self.eat_symbol(Symbol::Minus) {
            return match self.next() {
                Some(Token::Int(value)) => Ok(-value),
                Some(_) => Err(self.error_at_previous("expected int literal after `-`")),
                None => Err(self.error("expected int literal after `-`")),
            };
        }

        match self.next() {
            Some(Token::Int(value)) => Ok(value),
            Some(_) => Err(self.error_at_previous("expected int literal")),
            None => Err(self.error("expected int literal")),
        }
    }

    fn expect_protection_level(&mut self) -> Result<ProtectionLevel, ParseError> {
        match self.next() {
            Some(Token::Keyword(value)) if value == "AV" => Ok(ProtectionLevel::Av),
            Some(Token::Keyword(value)) if value == "ASV" => Ok(ProtectionLevel::Asv),
            Some(Token::Keyword(value)) if value == "SASV" => Ok(ProtectionLevel::Sasv),
            Some(_) => Err(self.error_at_previous("expected protection level AV, ASV, or SASV")),
            None => Err(self.error("expected protection level AV, ASV, or SASV")),
        }
    }

    fn expect_keyword(&mut self, expected: &'static str) -> Result<(), ParseError> {
        if self.eat_keyword(expected) {
            Ok(())
        } else {
            Err(self.error(format!("expected `{expected}`")))
        }
    }

    fn eat_keyword(&mut self, expected: &str) -> bool {
        match self.peek() {
            Some(Token::Keyword(value)) if value == expected => {
                self.cursor += 1;
                true
            }
            _ => false,
        }
    }

    fn check_keyword(&self, expected: &str) -> bool {
        matches!(self.peek(), Some(Token::Keyword(value)) if value == expected)
    }

    fn check_ident(&self, expected: &str) -> bool {
        matches!(self.peek(), Some(Token::Ident(value)) if value == expected)
    }

    fn check_symbol(&self, expected: Symbol) -> bool {
        matches!(self.peek(), Some(Token::Symbol(value)) if *value == expected)
    }

    fn next_starts_statement_or_block_end(&self) -> bool {
        match self.peek() {
            None | Some(Token::Symbol(Symbol::RightBrace)) => true,
            Some(Token::Keyword(value)) => {
                matches!(
                    value.as_str(),
                    "if" | "func"
                        | "while"
                        | "pick"
                        | "break"
                        | "continue"
                        | "return"
                        | "secretup"
                        | "stop"
                        | "fail"
                        | "out"
                        | "VTime"
                        | "List"
                ) || ValueType::parse_decl_keyword(value).is_some()
            }
            _ => false,
        }
    }

    fn expect_symbol(&mut self, expected: Symbol) -> Result<(), ParseError> {
        if self.eat_symbol(expected) {
            Ok(())
        } else {
            Err(self.error(format!("expected `{}`", expected.display())))
        }
    }

    fn eat_symbol(&mut self, expected: Symbol) -> bool {
        match self.peek() {
            Some(Token::Symbol(value)) if *value == expected => {
                self.cursor += 1;
                true
            }
            _ => false,
        }
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.cursor).cloned();
        self.cursor += usize::from(token.is_some());
        token
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.cursor)
    }

    fn is_done(&self) -> bool {
        self.cursor >= self.tokens.len()
    }

    fn error(&self, message: impl Into<String>) -> ParseError {
        let offset = self
            .offsets
            .get(self.cursor)
            .copied()
            .unwrap_or(self.source.len());
        parse_error_at(self.source, offset, message)
    }

    fn error_at_previous(&self, message: impl Into<String>) -> ParseError {
        let offset = self
            .cursor
            .checked_sub(1)
            .and_then(|index| self.offsets.get(index))
            .copied()
            .unwrap_or(self.source.len());
        parse_error_at(self.source, offset, message)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

struct Tokenized {
    tokens: Vec<Token>,
    offsets: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Keyword(String),
    Ident(String),
    Int(i64),
    Float(f64),
    Str(String),
    Symbol(Symbol),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Symbol {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    EqualEqual,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    SelfCheck,
    Plus,
    Minus,
    Star,
    Slash,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
}

impl Symbol {
    fn display(self) -> &'static str {
        match self {
            Self::Assign => "=",
            Self::AddAssign => "+=",
            Self::SubAssign => "-=",
            Self::MulAssign => "*=",
            Self::DivAssign => "/=",
            Self::EqualEqual => "==",
            Self::NotEqual => "!=",
            Self::Greater => ">",
            Self::GreaterEqual => ">=",
            Self::Less => "<",
            Self::LessEqual => "<=",
            Self::SelfCheck => "=self=",
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Star => "*",
            Self::Slash => "/",
            Self::LeftParen => "(",
            Self::RightParen => ")",
            Self::LeftBrace => "{",
            Self::RightBrace => "}",
            Self::LeftBracket => "[",
            Self::RightBracket => "]",
            Self::Comma => ",",
            Self::Colon => ":",
        }
    }
}

fn tokenize(source: &str) -> Result<Tokenized, ParseError> {
    let mut tokens = Vec::new();
    let mut offsets = Vec::new();
    let mut cursor = 0;

    while cursor < source.len() {
        let ch = source[cursor..].chars().next().unwrap();

        if ch.is_whitespace() {
            cursor += ch.len_utf8();
            continue;
        }

        if ch == '#' {
            while cursor < source.len() && !source[cursor..].starts_with('\n') {
                cursor += source[cursor..].chars().next().unwrap().len_utf8();
            }
            continue;
        }

        if ch == '"' {
            let (value, next_cursor) = read_string(source, cursor)?;
            offsets.push(cursor);
            tokens.push(Token::Str(value));
            cursor = next_cursor;
            continue;
        }

        if ch.is_ascii_digit() {
            let (token, next_cursor) = read_number(source, cursor)?;
            offsets.push(cursor);
            tokens.push(token);
            cursor = next_cursor;
            continue;
        }

        if is_ident_start(ch) {
            let start = cursor;
            cursor += ch.len_utf8();
            while cursor < source.len() {
                let next = source[cursor..].chars().next().unwrap();
                if !is_ident_continue(next) {
                    break;
                }
                cursor += next.len_utf8();
            }
            let value = &source[start..cursor];
            offsets.push(start);
            if is_keyword(value) || ValueType::parse_decl_keyword(value).is_some() {
                tokens.push(Token::Keyword(value.to_owned()));
            } else {
                tokens.push(Token::Ident(value.to_owned()));
            }
            continue;
        }

        let (symbol, next_cursor) = read_symbol(source, cursor)?;
        offsets.push(cursor);
        tokens.push(Token::Symbol(symbol));
        cursor = next_cursor;
    }

    Ok(Tokenized { tokens, offsets })
}

fn read_string(source: &str, start: usize) -> Result<(String, usize), ParseError> {
    let mut cursor = start + 1;
    let mut value = String::new();

    while cursor < source.len() {
        let ch = source[cursor..].chars().next().unwrap();
        if ch == '"' {
            return Ok((value, cursor + 1));
        }
        value.push(ch);
        cursor += ch.len_utf8();
    }

    Err(parse_error_at(source, start, "unterminated string literal"))
}

fn read_number(source: &str, start: usize) -> Result<(Token, usize), ParseError> {
    let mut cursor = start;
    let mut has_dot = false;

    while cursor < source.len() {
        let ch = source[cursor..].chars().next().unwrap();
        if ch == '.' && !has_dot {
            has_dot = true;
            cursor += 1;
        } else if ch.is_ascii_digit() {
            cursor += ch.len_utf8();
        } else {
            break;
        }
    }

    let raw = &source[start..cursor];
    if has_dot {
        let value = raw
            .parse::<f64>()
            .map_err(|_| parse_error_at(source, start, format!("invalid float literal `{raw}`")))?;
        Ok((Token::Float(value), cursor))
    } else {
        let value = raw
            .parse::<i64>()
            .map_err(|_| parse_error_at(source, start, format!("invalid int literal `{raw}`")))?;
        Ok((Token::Int(value), cursor))
    }
}

fn read_symbol(source: &str, cursor: usize) -> Result<(Symbol, usize), ParseError> {
    let rest = &source[cursor..];
    let pairs = [
        ("=self=", Symbol::SelfCheck),
        ("+=", Symbol::AddAssign),
        ("-=", Symbol::SubAssign),
        ("*=", Symbol::MulAssign),
        ("/=", Symbol::DivAssign),
        ("==", Symbol::EqualEqual),
        ("!=", Symbol::NotEqual),
        (">=", Symbol::GreaterEqual),
        ("<=", Symbol::LessEqual),
    ];

    for (raw, symbol) in pairs {
        if rest.starts_with(raw) {
            return Ok((symbol, cursor + raw.len()));
        }
    }

    let ch = rest.chars().next().unwrap();
    let symbol = match ch {
        '=' => Symbol::Assign,
        '>' => Symbol::Greater,
        '<' => Symbol::Less,
        '+' => Symbol::Plus,
        '-' => Symbol::Minus,
        '*' => Symbol::Star,
        '/' => Symbol::Slash,
        '(' => Symbol::LeftParen,
        ')' => Symbol::RightParen,
        '{' => Symbol::LeftBrace,
        '}' => Symbol::RightBrace,
        '[' => Symbol::LeftBracket,
        ']' => Symbol::RightBracket,
        ',' => Symbol::Comma,
        ':' => Symbol::Colon,
        _ => {
            return Err(parse_error_at(
                source,
                cursor,
                format!("unexpected character `{ch}`"),
            ))
        }
    };

    Ok((symbol, cursor + ch.len_utf8()))
}

fn parse_error_at(source: &str, offset: usize, message: impl Into<String>) -> ParseError {
    let prefix = &source[..offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = prefix
        .rsplit_once('\n')
        .map_or(prefix, |(_, current_line)| current_line)
        .chars()
        .count()
        + 1;
    ParseError {
        message: message.into(),
        offset,
        line,
        column,
    }
}

fn is_keyword(value: &str) -> bool {
    matches!(
        value,
        "if" | "else"
            | "while"
            | "pick"
            | "break"
            | "continue"
            | "func"
            | "return"
            | "true"
            | "false"
            | "and"
            | "or"
            | "not"
            | "out"
            | "input"
            | "secret"
            | "secretup"
            | "info"
            | "stop"
            | "fail"
            | "NONE"
            | "VTime"
            | "List"
            | "AV"
            | "ASV"
            | "SASV"
    )
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    ch == '_' || ch == '.' || ch.is_ascii_alphanumeric()
}

#[cfg(test)]
mod tests {
    use super::*;
    use apl_core::{validate_program, VariableKind};

    #[test]
    fn reports_tokenizer_line_and_unicode_column() {
        let error = parse_program("AVStr text = \"Привет\"\n  @").unwrap_err();

        assert_eq!(error.message, "unexpected character `@`");
        assert_eq!(error.line, 2);
        assert_eq!(error.column, 3);
        assert_eq!(error.offset, 30);
    }

    #[test]
    fn reports_parser_location_at_end_of_source() {
        let error = parse_program("AVInt value =\n").unwrap_err();

        assert_eq!(error.message, "expected expression");
        assert_eq!(error.line, 2);
        assert_eq!(error.column, 1);
        assert_eq!(error.offset, 14);
    }

    #[test]
    fn parses_variable_declarations_and_arithmetic() {
        let program = parse_program(
            r#"
            AVInt x = 4
            AVInt y = (x + 2) * 3
            x += 1
            "#,
        )
        .unwrap();

        assert_eq!(program.statements.len(), 3);
        assert_eq!(validate_program(&program), Ok(()));
    }

    #[test]
    fn parses_secret_families() {
        let program = parse_program(
            r#"
            ASVStr token = "hidden"
            SASVStr master_key = "raw"
            "#,
        )
        .unwrap();

        let Statement::VariableDecl(first) = &program.statements[0] else {
            panic!("expected declaration");
        };
        let Statement::VariableDecl(second) = &program.statements[1] else {
            panic!("expected declaration");
        };

        assert_eq!(first.kind, VariableKind::Asv);
        assert_eq!(second.kind, VariableKind::Sasv);
    }

    #[test]
    fn parses_self_check() {
        let program = parse_program(
            r#"
            AVInt x = 4
            AVBool unchanged = x =self=
            "#,
        )
        .unwrap();

        assert_eq!(validate_program(&program), Ok(()));
    }

    #[test]
    fn rejects_ungrouped_logical_conditions() {
        let error = parse_program(
            r#"
            AVBool a = true
            AVBool b = false
            if a == true and b == false {
              a = false
            }
            "#,
        )
        .unwrap_err();

        assert!(error.message.contains("logical operands"));
    }

    #[test]
    fn parses_grouped_if_else_chain() {
        let program = parse_program(
            r#"
            AVBool a = true
            AVBool b = false
            if (a == true) and (b == false) {
              b = true
            } else if a =self= {
              b = false
            } else {
              a = false
            }
            "#,
        )
        .unwrap();

        assert_eq!(validate_program(&program), Ok(()));
    }

    #[test]
    fn parses_none_input_secret_input_out_and_info() {
        let program = parse_program(
            r#"
            AVInt age = input
            AVStr raw = NONE
            ASVStr token = secret input
            SASVStr master_key = "raw"
            AVStr typ = ""
            AVStr level = ""
            typ, level = info(master_key)
            if age != NONE {
              out age
            } else {
              out raw
            }
            "#,
        )
        .unwrap();

        assert_eq!(validate_program(&program), Ok(()));
    }

    #[test]
    fn parses_while_with_limit_break_and_continue() {
        let program = parse_program(
            r#"
            AVInt x = 0
            while (x < 10) (-1) {
              x += 1
              if x == 3 {
                continue
              }
              if x == 5 {
                break
              }
            }
            "#,
        )
        .unwrap();

        assert_eq!(validate_program(&program), Ok(()));
    }

    #[test]
    fn parses_vtime_and_pick() {
        let program = parse_program(
            r#"
            AVStr text = "a b"
            pick(text): ch {
              VTime tmp = ch
              tmp = 1
              out tmp
            }
            "#,
        )
        .unwrap();

        assert_eq!(validate_program(&program), Ok(()));
    }

    #[test]
    fn parses_functions_calls_conversions_secretup_stop_and_fail() {
        let program = parse_program(
            r#"
            func parse_age(raw) {
              VTime age = int(raw)

              if age == NONE {
                return NONE
              }

              return age
            }

            AVStr raw = input
            AVInt age = parse_age(raw)
            AVStr token = input
            secretup(token)

            if age == NONE {
              fail "bad age"
            } else {
              stop
            }
            "#,
        )
        .unwrap();

        assert_eq!(validate_program(&program), Ok(()));
    }

    #[test]
    fn parses_lists_with_tagged_elements_and_stack_ops() {
        let program = parse_program(
            r#"
            List public_items = ["a", "b"]
            List secret_items = [1:SASV, "abc":ASV, true]
            List matrix = [[1, 2], [3, 4]]
            add(public_items, "c")
            add(secret_items, "hidden":ASV)
            VTime first = get(public_items, 0)
            VTime last = pop(public_items)
            pick(public_items): item {
              out item
            }
            "#,
        )
        .unwrap();

        assert_eq!(validate_program(&program), Ok(()));
    }

    #[test]
    fn parses_index_and_slice_access() {
        let program = parse_program(
            r#"
            List values = [1, 2, 3, 4]
            List matrix = [[1, 2], [3, 4]]
            VTime first = values[0]
            VTime cell = matrix[0][1]
            VTime head = values[:3]
            VTime tail = values[1:]
            VTime every_second = values[::2]
            VTime middle = values[1:3:1]
            "#,
        )
        .unwrap();

        assert_eq!(validate_program(&program), Ok(()));
    }
}
