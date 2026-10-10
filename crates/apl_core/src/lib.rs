use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

impl Program {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    VariableDecl(VariableDecl),
    ListDecl(ListDecl),
    VTimeDecl(VTimeDecl),
    FunctionDecl(FunctionDecl),
    Assignment(Assignment),
    InfoAssignment(InfoAssignment),
    AddToList {
        list_name: String,
        element: ListElement,
    },
    SecretUp(String),
    Out(Expression),
    Stop(Option<Expression>),
    Fail(Expression),
    Return(Expression),
    If(IfStatement),
    While(WhileStatement),
    Pick(PickStatement),
    Break,
    Continue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDecl {
    pub kind: VariableKind,
    pub value_type: ValueType,
    pub name: String,
    pub initial_value: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListDecl {
    pub name: String,
    pub elements: Vec<ListElement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListElement {
    pub value: Expression,
    pub protection: Option<ProtectionLevel>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VTimeDecl {
    pub name: String,
    pub initial_value: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableKind {
    Av,
    Asv,
    Sasv,
}

impl VariableKind {
    fn protection(self) -> ProtectionLevel {
        match self {
            Self::Av => ProtectionLevel::Av,
            Self::Asv => ProtectionLevel::Asv,
            Self::Sasv => ProtectionLevel::Sasv,
        }
    }
}

impl fmt::Display for VariableKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Av => "AV",
            Self::Asv => "ASV",
            Self::Sasv => "SASV",
        };
        f.write_str(name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProtectionLevel {
    Av,
    Asv,
    Sasv,
}

impl fmt::Display for ProtectionLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Av => "AV",
            Self::Asv => "ASV",
            Self::Sasv => "SASV",
        };
        f.write_str(name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Int,
    Float,
    Bool,
    Str,
    Bytes,
    Json,
    List,
}

impl ValueType {
    pub fn parse_decl_keyword(input: &str) -> Option<(VariableKind, Self)> {
        let (kind, value_type) = if let Some(value_type) = input.strip_prefix("SASV") {
            (VariableKind::Sasv, value_type)
        } else if let Some(value_type) = input.strip_prefix("ASV") {
            (VariableKind::Asv, value_type)
        } else if let Some(value_type) = input.strip_prefix("AV") {
            (VariableKind::Av, value_type)
        } else {
            return None;
        };

        let value_type = match value_type {
            "Int" => Self::Int,
            "Float" => Self::Float,
            "Bool" => Self::Bool,
            "Str" => Self::Str,
            "Bytes" => Self::Bytes,
            "Json" => Self::Json,
            _ => return None,
        };

        Some((kind, value_type))
    }
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Int => "Int",
            Self::Float => "Float",
            Self::Bool => "Bool",
            Self::Str => "Str",
            Self::Bytes => "Bytes",
            Self::Json => "Json",
            Self::List => "List",
        };
        f.write_str(name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub name: String,
    pub operator: AssignmentOperator,
    pub value: Expression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignmentOperator {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InfoAssignment {
    pub type_target: String,
    pub protection_target: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStatement {
    pub condition: Expression,
    pub body: Vec<Statement>,
    pub else_if_branches: Vec<ElseIfBranch>,
    pub else_body: Option<Vec<Statement>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElseIfBranch {
    pub condition: Expression,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileStatement {
    pub condition: Expression,
    pub limit: i64,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PickStatement {
    pub value: Expression,
    pub item_name: String,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Value),
    ListLiteral(Vec<ListElement>),
    None,
    Input,
    SecretInput,
    Variable(String),
    Call {
        name: String,
        args: Vec<Expression>,
    },
    Index {
        target: Box<Expression>,
        index: Box<Expression>,
    },
    Slice {
        target: Box<Expression>,
        start: Option<Box<Expression>>,
        end: Option<Box<Expression>>,
        step: Option<Box<Expression>>,
    },
    Unary {
        operator: UnaryOperator,
        expression: Box<Expression>,
    },
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    SelfCheck(String),
    Grouped(Box<Expression>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Negate,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Bytes(Vec<u8>),
    Json(String),
}

impl Value {
    pub fn value_type(&self) -> ValueType {
        match self {
            Self::Int(_) => ValueType::Int,
            Self::Float(_) => ValueType::Float,
            Self::Bool(_) => ValueType::Bool,
            Self::Str(_) => ValueType::Str,
            Self::Bytes(_) => ValueType::Bytes,
            Self::Json(_) => ValueType::Json,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExpressionInfo {
    value_type: DynamicType,
    protection: ProtectionLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DynamicType {
    Value(ValueType),
    VTime,
    None,
    Input,
    SecretInput,
}

impl ExpressionInfo {
    fn value(value_type: ValueType, protection: ProtectionLevel) -> Self {
        Self {
            value_type: DynamicType::Value(value_type),
            protection,
        }
    }

    fn is_assignable_to(self, value_type: ValueType, target_protection: ProtectionLevel) -> bool {
        self.protection <= target_protection
            && match self.value_type {
                DynamicType::Value(actual) => actual == value_type,
                DynamicType::VTime
                | DynamicType::None
                | DynamicType::Input
                | DynamicType::SecretInput => true,
            }
    }

    fn reported_type(self) -> ValueType {
        match self.value_type {
            DynamicType::Value(value_type) => value_type,
            DynamicType::VTime
            | DynamicType::None
            | DynamicType::Input
            | DynamicType::SecretInput => ValueType::Str,
        }
    }
}

impl fmt::Display for ExpressionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value_type {
            DynamicType::Value(value_type) => write!(f, "{value_type}:{}", self.protection),
            DynamicType::VTime => write!(f, "VTime:{}", self.protection),
            DynamicType::None => f.write_str("NONE"),
            DynamicType::Input => f.write_str("input"),
            DynamicType::SecretInput => f.write_str("secret input"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckError {
    DuplicateVariable(String),
    DuplicateFunction(String),
    InvalidName(String),
    UnknownVariable(String),
    UnknownFunction(String),
    WrongArgumentCount {
        name: String,
        expected: usize,
        actual: usize,
    },
    TypeMismatch {
        name: String,
        expected: ValueType,
        actual: ValueType,
    },
    ProtectionDowngrade {
        name: String,
        source: ProtectionLevel,
        target: ProtectionLevel,
    },
    InvalidAssignmentTarget(String),
    InvalidOperation(String),
    InvalidLoopLimit(i64),
    BreakOutsideLoop,
    ContinueOutsideLoop,
    ReturnOutsideFunction,
    SecretExpressionDenied(String),
}

pub fn validate_program(program: &Program) -> Result<(), CheckError> {
    let mut checker = Checker::default();
    checker.collect_functions(&program.statements)?;
    checker.check_statements(&program.statements)
}

#[derive(Default)]
struct Checker {
    symbols: HashMap<String, Symbol>,
    functions: HashMap<String, usize>,
    vtime_scopes: Vec<HashMap<String, ProtectionLevel>>,
    loop_depth: usize,
    function_depth: usize,
}

#[derive(Debug, Clone, Copy)]
enum Symbol {
    Absolute {
        kind: VariableKind,
        value_type: ValueType,
    },
    List {
        protection: ProtectionLevel,
    },
}

impl Checker {
    fn collect_functions(&mut self, statements: &[Statement]) -> Result<(), CheckError> {
        for statement in statements {
            if let Statement::FunctionDecl(function) = statement {
                validate_variable_name(&function.name)?;
                if self.functions.contains_key(&function.name) {
                    return Err(CheckError::DuplicateFunction(function.name.clone()));
                }
                self.functions
                    .insert(function.name.clone(), function.params.len());
            }
        }
        Ok(())
    }

    fn check_statements(&mut self, statements: &[Statement]) -> Result<(), CheckError> {
        for statement in statements {
            self.check_statement(statement)?;
        }
        Ok(())
    }

    fn check_statement(&mut self, statement: &Statement) -> Result<(), CheckError> {
        match statement {
            Statement::VariableDecl(decl) => self.check_variable_decl(decl),
            Statement::ListDecl(decl) => self.check_list_decl(decl),
            Statement::VTimeDecl(decl) => self.check_vtime_decl(decl),
            Statement::FunctionDecl(function) => self.check_function_decl(function),
            Statement::Assignment(assignment) => self.check_assignment(assignment),
            Statement::InfoAssignment(assignment) => self.check_info_assignment(assignment),
            Statement::AddToList { list_name, element } => {
                self.check_add_to_list(list_name, element)
            }
            Statement::SecretUp(name) => self.check_secretup(name),
            Statement::Out(expression) => self.check_out(expression),
            Statement::Stop(reason) => {
                if let Some(reason) = reason {
                    self.check_public_external(reason)?;
                }
                Ok(())
            }
            Statement::Fail(reason) => {
                self.check_public_external(reason)?;
                Ok(())
            }
            Statement::Return(expression) => {
                if self.function_depth == 0 {
                    return Err(CheckError::ReturnOutsideFunction);
                }
                self.check_expression(expression)?;
                Ok(())
            }
            Statement::If(if_statement) => self.check_if(if_statement),
            Statement::While(while_statement) => self.check_while(while_statement),
            Statement::Pick(pick_statement) => self.check_pick(pick_statement),
            Statement::Break => {
                if self.loop_depth == 0 {
                    Err(CheckError::BreakOutsideLoop)
                } else {
                    Ok(())
                }
            }
            Statement::Continue => {
                if self.loop_depth == 0 {
                    Err(CheckError::ContinueOutsideLoop)
                } else {
                    Ok(())
                }
            }
        }
    }

    fn check_variable_decl(&mut self, decl: &VariableDecl) -> Result<(), CheckError> {
        validate_variable_name(&decl.name)?;
        if self.name_exists(&decl.name) || self.functions.contains_key(&decl.name) {
            return Err(CheckError::DuplicateVariable(decl.name.clone()));
        }

        let actual = self.check_expression(&decl.initial_value)?;
        let target_protection = decl.kind.protection();
        if matches!(actual.value_type, DynamicType::SecretInput) && decl.kind != VariableKind::Asv {
            return Err(CheckError::SecretExpressionDenied(decl.name.clone()));
        }
        if decl.kind == VariableKind::Sasv
            && matches!(
                actual.value_type,
                DynamicType::Input | DynamicType::SecretInput
            )
        {
            return Err(CheckError::SecretExpressionDenied(decl.name.clone()));
        }
        self.ensure_assignable(&decl.name, actual, decl.value_type, target_protection)?;

        self.symbols.insert(
            decl.name.clone(),
            Symbol::Absolute {
                kind: decl.kind,
                value_type: decl.value_type,
            },
        );
        Ok(())
    }

    fn check_list_decl(&mut self, decl: &ListDecl) -> Result<(), CheckError> {
        validate_variable_name(&decl.name)?;
        if self.name_exists(&decl.name) || self.functions.contains_key(&decl.name) {
            return Err(CheckError::DuplicateVariable(decl.name.clone()));
        }
        let mut protection = ProtectionLevel::Av;
        for element in &decl.elements {
            let info = self.check_list_element(element)?;
            protection = protection.max(info.protection);
        }
        self.symbols
            .insert(decl.name.clone(), Symbol::List { protection });
        Ok(())
    }

    fn check_list_element(&mut self, element: &ListElement) -> Result<ExpressionInfo, CheckError> {
        let mut info = self.check_expression(&element.value)?;
        if let Some(protection) = element.protection {
            if info.protection > protection {
                return Err(CheckError::ProtectionDowngrade {
                    name: "list element".to_owned(),
                    source: info.protection,
                    target: protection,
                });
            }
            info.protection = protection;
        }
        Ok(info)
    }

    fn check_vtime_decl(&mut self, decl: &VTimeDecl) -> Result<(), CheckError> {
        validate_variable_name(&decl.name)?;
        if self.name_exists(&decl.name) || self.functions.contains_key(&decl.name) {
            return Err(CheckError::DuplicateVariable(decl.name.clone()));
        }
        let info = self.check_expression(&decl.initial_value)?;
        self.declare_vtime(decl.name.clone(), info.protection);
        Ok(())
    }

    fn check_function_decl(&mut self, function: &FunctionDecl) -> Result<(), CheckError> {
        let mut seen = HashSet::new();
        for param in &function.params {
            validate_variable_name(param)?;
            if !seen.insert(param.clone()) {
                return Err(CheckError::DuplicateVariable(param.clone()));
            }
        }

        let enclosing_loop_depth = self.loop_depth;
        self.loop_depth = 0;
        self.function_depth += 1;
        let result = self.with_scope(|checker| {
            for param in &function.params {
                checker.declare_vtime(param.clone(), ProtectionLevel::Av);
            }
            checker.check_statements(&function.body)
        });
        self.function_depth -= 1;
        self.loop_depth = enclosing_loop_depth;
        result
    }

    fn check_assignment(&mut self, assignment: &Assignment) -> Result<(), CheckError> {
        if self.is_vtime(&assignment.name) {
            let info = self.check_expression(&assignment.value)?;
            self.update_vtime(&assignment.name, info.protection);
            return Ok(());
        }

        let Some(symbol) = self.symbols.get(&assignment.name).copied() else {
            return Err(CheckError::UnknownVariable(assignment.name.clone()));
        };

        let Symbol::Absolute { kind, value_type } = symbol else {
            return Err(CheckError::InvalidAssignmentTarget(assignment.name.clone()));
        };

        let target_protection = kind.protection();
        let actual = self.check_expression(&assignment.value)?;
        self.ensure_assignable(&assignment.name, actual, value_type, target_protection)?;

        match assignment.operator {
            AssignmentOperator::Assign => Ok(()),
            AssignmentOperator::AddAssign
            | AssignmentOperator::SubAssign
            | AssignmentOperator::MulAssign
            | AssignmentOperator::DivAssign => {
                if kind != VariableKind::Av {
                    return Err(CheckError::InvalidAssignmentTarget(assignment.name.clone()));
                }
                if matches!(value_type, ValueType::Int | ValueType::Float) {
                    Ok(())
                } else {
                    Err(CheckError::InvalidOperation(format!(
                        "compound arithmetic assignment requires Int or Float, got {value_type}"
                    )))
                }
            }
        }
    }

    fn check_info_assignment(&mut self, assignment: &InfoAssignment) -> Result<(), CheckError> {
        self.require_av_str_target(&assignment.type_target)?;
        self.require_av_str_target(&assignment.protection_target)?;
        if self.is_vtime(&assignment.source)
            || !matches!(
                self.symbols.get(&assignment.source),
                Some(Symbol::Absolute { .. })
            )
        {
            return Err(CheckError::UnknownVariable(assignment.source.clone()));
        }
        Ok(())
    }

    fn check_add_to_list(
        &mut self,
        list_name: &str,
        element: &ListElement,
    ) -> Result<(), CheckError> {
        if self.is_vtime(list_name) {
            let info = self.check_list_element(element)?;
            let protection = self
                .vtime_protection(list_name)
                .unwrap_or(ProtectionLevel::Av)
                .max(info.protection);
            self.update_vtime(list_name, protection);
            return Ok(());
        }

        if !matches!(self.symbols.get(list_name), Some(Symbol::List { .. })) {
            return Err(CheckError::InvalidAssignmentTarget(list_name.to_owned()));
        }
        let info = self.check_list_element(element)?;
        if let Some(Symbol::List { protection }) = self.symbols.get_mut(list_name) {
            *protection = (*protection).max(info.protection);
        }
        Ok(())
    }

    fn check_secretup(&mut self, name: &str) -> Result<(), CheckError> {
        if self.is_vtime(name) {
            return Err(CheckError::InvalidAssignmentTarget(name.to_owned()));
        }

        let Some(symbol) = self.symbols.get_mut(name) else {
            return Err(CheckError::UnknownVariable(name.to_owned()));
        };

        let Symbol::Absolute { kind, .. } = symbol else {
            return Err(CheckError::InvalidAssignmentTarget(name.to_owned()));
        };

        *kind = match *kind {
            VariableKind::Av => VariableKind::Asv,
            VariableKind::Asv => VariableKind::Sasv,
            VariableKind::Sasv => return Err(CheckError::InvalidAssignmentTarget(name.to_owned())),
        };
        Ok(())
    }

    fn check_out(&mut self, expression: &Expression) -> Result<(), CheckError> {
        self.check_public_external(expression)
    }

    fn check_public_external(&mut self, expression: &Expression) -> Result<(), CheckError> {
        let info = self.check_expression(expression)?;
        if info.protection != ProtectionLevel::Av {
            return Err(CheckError::SecretExpressionDenied(format!(
                "{expression:?}"
            )));
        }
        Ok(())
    }

    fn require_av_str_target(&self, name: &str) -> Result<(), CheckError> {
        if self.is_vtime(name) {
            return Err(CheckError::InvalidAssignmentTarget(name.to_owned()));
        }

        match self.symbols.get(name) {
            Some(Symbol::Absolute {
                kind: VariableKind::Av,
                value_type: ValueType::Str,
            }) => Ok(()),
            Some(_) => Err(CheckError::InvalidAssignmentTarget(name.to_owned())),
            None => Err(CheckError::UnknownVariable(name.to_owned())),
        }
    }

    fn check_if(&mut self, if_statement: &IfStatement) -> Result<(), CheckError> {
        self.require_bool(&if_statement.condition)?;
        self.with_scope(|checker| checker.check_statements(&if_statement.body))?;
        for branch in &if_statement.else_if_branches {
            self.require_bool(&branch.condition)?;
            self.with_scope(|checker| checker.check_statements(&branch.body))?;
        }
        if let Some(body) = &if_statement.else_body {
            self.with_scope(|checker| checker.check_statements(body))?;
        }
        Ok(())
    }

    fn check_while(&mut self, while_statement: &WhileStatement) -> Result<(), CheckError> {
        if while_statement.limit < -1 {
            return Err(CheckError::InvalidLoopLimit(while_statement.limit));
        }
        self.require_bool(&while_statement.condition)?;
        self.loop_depth += 1;
        let result = self.with_scope(|checker| checker.check_statements(&while_statement.body));
        self.loop_depth -= 1;
        result
    }

    fn check_pick(&mut self, pick_statement: &PickStatement) -> Result<(), CheckError> {
        validate_variable_name(&pick_statement.item_name)?;
        if self.name_exists(&pick_statement.item_name)
            || self.functions.contains_key(&pick_statement.item_name)
        {
            return Err(CheckError::DuplicateVariable(
                pick_statement.item_name.clone(),
            ));
        }
        let value_info = self.check_expression(&pick_statement.value)?;
        if !matches!(
            value_info.value_type,
            DynamicType::Value(ValueType::Str | ValueType::Bytes | ValueType::List)
                | DynamicType::VTime
        ) {
            return Err(CheckError::InvalidOperation(format!(
                "pick requires Str, Bytes, List, or VTime, got {value_info}"
            )));
        }
        self.loop_depth += 1;
        let result = self.with_scope(|checker| {
            checker.declare_vtime(pick_statement.item_name.clone(), value_info.protection);
            checker.check_statements(&pick_statement.body)
        });
        self.loop_depth -= 1;
        result
    }

    fn require_bool(&mut self, expression: &Expression) -> Result<(), CheckError> {
        let actual = self.check_expression(expression)?;
        if matches!(
            actual.value_type,
            DynamicType::Value(ValueType::Bool) | DynamicType::VTime
        ) {
            Ok(())
        } else {
            Err(CheckError::TypeMismatch {
                name: "condition".to_owned(),
                expected: ValueType::Bool,
                actual: actual.reported_type(),
            })
        }
    }

    fn check_expression(&mut self, expression: &Expression) -> Result<ExpressionInfo, CheckError> {
        match expression {
            Expression::Literal(value) => Ok(ExpressionInfo::value(
                value.value_type(),
                ProtectionLevel::Av,
            )),
            Expression::ListLiteral(elements) => {
                let mut protection = ProtectionLevel::Av;
                for element in elements {
                    let info = self.check_list_element(element)?;
                    protection = protection.max(info.protection);
                }
                Ok(ExpressionInfo::value(ValueType::List, protection))
            }
            Expression::None => Ok(ExpressionInfo {
                value_type: DynamicType::None,
                protection: ProtectionLevel::Av,
            }),
            Expression::Input => Ok(ExpressionInfo {
                value_type: DynamicType::Input,
                protection: ProtectionLevel::Av,
            }),
            Expression::SecretInput => Ok(ExpressionInfo {
                value_type: DynamicType::SecretInput,
                protection: ProtectionLevel::Asv,
            }),
            Expression::Variable(name) => self.check_variable_reference(name),
            Expression::Call { name, args } => self.check_call(name, args),
            Expression::Index { target, index } => self.check_index(target, index),
            Expression::Slice {
                target,
                start,
                end,
                step,
            } => self.check_slice(target, start.as_deref(), end.as_deref(), step.as_deref()),
            Expression::Grouped(inner) => self.check_expression(inner),
            Expression::SelfCheck(name) => {
                if self.is_vtime(name) {
                    return Err(CheckError::InvalidOperation(
                        "`=self=` is only available for absolute variables".to_owned(),
                    ));
                }

                let Some(symbol) = self.symbols.get(name) else {
                    return Err(CheckError::UnknownVariable(name.clone()));
                };
                let Symbol::Absolute { kind, .. } = symbol else {
                    return Err(CheckError::InvalidOperation(
                        "`=self=` is only available for absolute variables".to_owned(),
                    ));
                };
                if *kind != VariableKind::Av {
                    return Err(CheckError::SecretExpressionDenied(name.clone()));
                }
                Ok(ExpressionInfo::value(ValueType::Bool, ProtectionLevel::Av))
            }
            Expression::Unary {
                operator,
                expression,
            } => {
                let inner = self.check_expression(expression)?;
                match operator {
                    UnaryOperator::Negate
                        if matches!(
                            inner.value_type,
                            DynamicType::Value(ValueType::Int | ValueType::Float)
                                | DynamicType::VTime
                        ) =>
                    {
                        Ok(inner)
                    }
                    UnaryOperator::Not
                        if matches!(
                            inner.value_type,
                            DynamicType::Value(ValueType::Bool) | DynamicType::VTime
                        ) =>
                    {
                        Ok(ExpressionInfo::value(ValueType::Bool, inner.protection))
                    }
                    _ => Err(CheckError::InvalidOperation(format!(
                        "invalid unary operation for {inner}"
                    ))),
                }
            }
            Expression::Binary {
                left,
                operator,
                right,
            } => self.check_binary_expression(left, *operator, right),
        }
    }

    fn check_call(
        &mut self,
        name: &str,
        args: &[Expression],
    ) -> Result<ExpressionInfo, CheckError> {
        match name {
            "add" => {
                return Err(CheckError::InvalidOperation(
                    "add(list, value) is only valid as a statement".to_owned(),
                ));
            }
            "int" | "float" | "bool" | "str" | "bytes" | "json" => {
                if args.len() != 1 {
                    return Err(CheckError::WrongArgumentCount {
                        name: name.to_owned(),
                        expected: 1,
                        actual: args.len(),
                    });
                }
                let arg = self.check_expression(&args[0])?;
                return Ok(ExpressionInfo::value(
                    conversion_type(name).unwrap(),
                    arg.protection,
                ));
            }
            "get" => {
                if args.len() != 2 {
                    return Err(CheckError::WrongArgumentCount {
                        name: name.to_owned(),
                        expected: 2,
                        actual: args.len(),
                    });
                }
                let list = self.check_expression(&args[0])?;
                let index = self.check_expression(&args[1])?;
                if !matches!(
                    list.value_type,
                    DynamicType::Value(ValueType::List) | DynamicType::VTime
                ) || !matches!(
                    index.value_type,
                    DynamicType::Value(ValueType::Int) | DynamicType::VTime
                ) {
                    return Err(CheckError::InvalidOperation(
                        "get(list, index) requires List and Int".to_owned(),
                    ));
                }
                return Ok(ExpressionInfo {
                    value_type: DynamicType::VTime,
                    protection: list.protection,
                });
            }
            "pop" => {
                if args.len() != 1 {
                    return Err(CheckError::WrongArgumentCount {
                        name: name.to_owned(),
                        expected: 1,
                        actual: args.len(),
                    });
                }
                let list = self.check_expression(&args[0])?;
                if !matches!(
                    list.value_type,
                    DynamicType::Value(ValueType::List) | DynamicType::VTime
                ) {
                    return Err(CheckError::InvalidOperation(
                        "pop(list) requires List".to_owned(),
                    ));
                }
                if expression_variable_name(&args[0]).is_none() {
                    return Err(CheckError::InvalidOperation(
                        "pop(list) requires a named List or VTime target".to_owned(),
                    ));
                }
                return Ok(ExpressionInfo {
                    value_type: DynamicType::VTime,
                    protection: list.protection,
                });
            }
            "split" => {
                if args.len() != 2 {
                    return Err(CheckError::WrongArgumentCount {
                        name: name.to_owned(),
                        expected: 2,
                        actual: args.len(),
                    });
                }
                let value = self.check_expression(&args[0])?;
                let separator = self.check_expression(&args[1])?;
                if !matches!(
                    value.value_type,
                    DynamicType::Value(ValueType::Str) | DynamicType::VTime
                ) || !matches!(
                    separator.value_type,
                    DynamicType::Value(ValueType::Str) | DynamicType::VTime
                ) {
                    return Err(CheckError::InvalidOperation(
                        "split(value, separator) requires Str and Str".to_owned(),
                    ));
                }
                return Ok(ExpressionInfo::value(
                    ValueType::List,
                    value.protection.max(separator.protection),
                ));
            }
            "len" => {
                if args.len() != 1 {
                    return Err(CheckError::WrongArgumentCount {
                        name: name.to_owned(),
                        expected: 1,
                        actual: args.len(),
                    });
                }
                let value = self.check_expression(&args[0])?;
                if !matches!(
                    value.value_type,
                    DynamicType::Value(ValueType::Str | ValueType::Bytes | ValueType::List)
                        | DynamicType::VTime
                ) {
                    return Err(CheckError::InvalidOperation(
                        "len(value) requires Str, Bytes, List, or VTime".to_owned(),
                    ));
                }
                return Ok(ExpressionInfo::value(ValueType::Int, value.protection));
            }
            "contains" => {
                if args.len() != 2 {
                    return Err(CheckError::WrongArgumentCount {
                        name: name.to_owned(),
                        expected: 2,
                        actual: args.len(),
                    });
                }
                let value = self.check_expression(&args[0])?;
                let needle = self.check_expression(&args[1])?;
                if !matches!(
                    value.value_type,
                    DynamicType::Value(ValueType::Str) | DynamicType::VTime
                ) || !matches!(
                    needle.value_type,
                    DynamicType::Value(ValueType::Str) | DynamicType::VTime
                ) {
                    return Err(CheckError::InvalidOperation(
                        "contains(value, needle) requires Str and Str".to_owned(),
                    ));
                }
                return Ok(ExpressionInfo::value(
                    ValueType::Bool,
                    value.protection.max(needle.protection),
                ));
            }
            "join" => {
                if args.len() != 2 {
                    return Err(CheckError::WrongArgumentCount {
                        name: name.to_owned(),
                        expected: 2,
                        actual: args.len(),
                    });
                }
                let values = self.check_expression(&args[0])?;
                let separator = self.check_expression(&args[1])?;
                if !matches!(
                    values.value_type,
                    DynamicType::Value(ValueType::List) | DynamicType::VTime
                ) || !matches!(
                    separator.value_type,
                    DynamicType::Value(ValueType::Str) | DynamicType::VTime
                ) {
                    return Err(CheckError::InvalidOperation(
                        "join(values, separator) requires List and Str".to_owned(),
                    ));
                }
                return Ok(ExpressionInfo::value(
                    ValueType::Str,
                    values.protection.max(separator.protection),
                ));
            }
            "ord" => {
                if args.len() != 1 {
                    return Err(CheckError::WrongArgumentCount {
                        name: name.to_owned(),
                        expected: 1,
                        actual: args.len(),
                    });
                }
                let value = self.check_expression(&args[0])?;
                if !matches!(
                    value.value_type,
                    DynamicType::Value(ValueType::Str) | DynamicType::VTime
                ) {
                    return Err(CheckError::InvalidOperation(
                        "ord(value) requires Str".to_owned(),
                    ));
                }
                return Ok(ExpressionInfo::value(ValueType::Int, value.protection));
            }
            "char" => {
                if args.len() != 1 {
                    return Err(CheckError::WrongArgumentCount {
                        name: name.to_owned(),
                        expected: 1,
                        actual: args.len(),
                    });
                }
                let value = self.check_expression(&args[0])?;
                if !matches!(
                    value.value_type,
                    DynamicType::Value(ValueType::Int) | DynamicType::VTime
                ) {
                    return Err(CheckError::InvalidOperation(
                        "char(value) requires Int".to_owned(),
                    ));
                }
                return Ok(ExpressionInfo::value(ValueType::Str, value.protection));
            }
            "pow" => {
                if args.len() != 2 {
                    return Err(CheckError::WrongArgumentCount {
                        name: name.to_owned(),
                        expected: 2,
                        actual: args.len(),
                    });
                }
                let base = self.check_expression(&args[0])?;
                let exponent = self.check_expression(&args[1])?;
                if !matches!(
                    base.value_type,
                    DynamicType::Value(ValueType::Int | ValueType::Float) | DynamicType::VTime
                ) || !matches!(
                    exponent.value_type,
                    DynamicType::Value(ValueType::Int) | DynamicType::VTime
                ) {
                    return Err(CheckError::InvalidOperation(
                        "pow(base, exponent) requires Int/Float and Int".to_owned(),
                    ));
                }
                let value_type = match base.value_type {
                    DynamicType::Value(value_type) => DynamicType::Value(value_type),
                    _ => DynamicType::VTime,
                };
                return Ok(ExpressionInfo {
                    value_type,
                    protection: base.protection.max(exponent.protection),
                });
            }
            _ => {}
        }

        let Some(expected) = self.functions.get(name).copied() else {
            return Err(CheckError::UnknownFunction(name.to_owned()));
        };
        if expected != args.len() {
            return Err(CheckError::WrongArgumentCount {
                name: name.to_owned(),
                expected,
                actual: args.len(),
            });
        }
        let mut protection = ProtectionLevel::Av;
        for arg in args {
            protection = protection.max(self.check_expression(arg)?.protection);
        }
        Ok(ExpressionInfo {
            value_type: DynamicType::VTime,
            protection,
        })
    }

    fn check_index(
        &mut self,
        target: &Expression,
        index: &Expression,
    ) -> Result<ExpressionInfo, CheckError> {
        let target = self.check_expression(target)?;
        let index = self.check_expression(index)?;

        if !matches!(
            target.value_type,
            DynamicType::Value(ValueType::Str | ValueType::Bytes | ValueType::List)
                | DynamicType::VTime
        ) {
            return Err(CheckError::InvalidOperation(format!(
                "index access requires Str, Bytes, List, or VTime, got {target}"
            )));
        }

        if !matches!(
            index.value_type,
            DynamicType::Value(ValueType::Int) | DynamicType::VTime
        ) {
            return Err(CheckError::InvalidOperation(format!(
                "index must be Int or VTime, got {index}"
            )));
        }

        Ok(ExpressionInfo {
            value_type: DynamicType::VTime,
            protection: target.protection.max(index.protection),
        })
    }

    fn check_slice(
        &mut self,
        target: &Expression,
        start: Option<&Expression>,
        end: Option<&Expression>,
        step: Option<&Expression>,
    ) -> Result<ExpressionInfo, CheckError> {
        let target = self.check_expression(target)?;
        if !matches!(
            target.value_type,
            DynamicType::Value(ValueType::Str | ValueType::Bytes | ValueType::List)
                | DynamicType::VTime
        ) {
            return Err(CheckError::InvalidOperation(format!(
                "slice requires Str, Bytes, List, or VTime, got {target}"
            )));
        }

        let mut protection = target.protection;
        for bound in [start, end, step].into_iter().flatten() {
            let info = self.check_expression(bound)?;
            if !matches!(
                info.value_type,
                DynamicType::Value(ValueType::Int) | DynamicType::VTime
            ) {
                return Err(CheckError::InvalidOperation(format!(
                    "slice bounds must be Int or VTime, got {info}"
                )));
            }
            protection = protection.max(info.protection);
        }

        let value_type = match target.value_type {
            DynamicType::Value(ValueType::Str) => DynamicType::Value(ValueType::Str),
            DynamicType::Value(ValueType::Bytes) => DynamicType::Value(ValueType::Bytes),
            DynamicType::Value(ValueType::List) => DynamicType::Value(ValueType::List),
            DynamicType::VTime => DynamicType::VTime,
            _ => unreachable!("validated slice target type"),
        };

        Ok(ExpressionInfo {
            value_type,
            protection,
        })
    }

    fn check_variable_reference(&self, name: &str) -> Result<ExpressionInfo, CheckError> {
        if let Some(protection) = self.vtime_protection(name) {
            return Ok(ExpressionInfo {
                value_type: DynamicType::VTime,
                protection,
            });
        }
        match self.symbols.get(name) {
            Some(Symbol::Absolute { kind, value_type }) => {
                Ok(ExpressionInfo::value(*value_type, kind.protection()))
            }
            Some(Symbol::List { protection }) => {
                Ok(ExpressionInfo::value(ValueType::List, *protection))
            }
            None => Err(CheckError::UnknownVariable(name.to_owned())),
        }
    }

    fn check_binary_expression(
        &mut self,
        left: &Expression,
        operator: BinaryOperator,
        right: &Expression,
    ) -> Result<ExpressionInfo, CheckError> {
        let left_info = self.check_expression(left)?;
        let right_info = self.check_expression(right)?;
        let protection = left_info.protection.max(right_info.protection);

        if matches!(left_info.value_type, DynamicType::VTime)
            || matches!(right_info.value_type, DynamicType::VTime)
        {
            return match operator {
                BinaryOperator::Equal
                | BinaryOperator::NotEqual
                | BinaryOperator::Greater
                | BinaryOperator::GreaterEqual
                | BinaryOperator::Less
                | BinaryOperator::LessEqual
                | BinaryOperator::And
                | BinaryOperator::Or => Ok(ExpressionInfo::value(ValueType::Bool, protection)),
                BinaryOperator::Add
                | BinaryOperator::Sub
                | BinaryOperator::Mul
                | BinaryOperator::Div => Ok(ExpressionInfo {
                    value_type: DynamicType::VTime,
                    protection,
                }),
            };
        }

        match operator {
            BinaryOperator::Add
            | BinaryOperator::Sub
            | BinaryOperator::Mul
            | BinaryOperator::Div => {
                if left_info.value_type == right_info.value_type
                    && matches!(
                        left_info.value_type,
                        DynamicType::Value(ValueType::Int | ValueType::Float)
                    )
                {
                    Ok(ExpressionInfo {
                        value_type: left_info.value_type,
                        protection,
                    })
                } else {
                    Err(CheckError::InvalidOperation(format!(
                        "arithmetic requires matching Int or Float values, got {left_info} and {right_info}"
                    )))
                }
            }
            BinaryOperator::Equal | BinaryOperator::NotEqual => {
                if matches!(left_info.value_type, DynamicType::None)
                    || matches!(right_info.value_type, DynamicType::None)
                    || left_info.value_type == right_info.value_type
                {
                    Ok(ExpressionInfo::value(ValueType::Bool, protection))
                } else {
                    Err(CheckError::InvalidOperation(format!(
                        "comparison requires matching types, got {left_info} and {right_info}"
                    )))
                }
            }
            BinaryOperator::Greater
            | BinaryOperator::GreaterEqual
            | BinaryOperator::Less
            | BinaryOperator::LessEqual => {
                if left_info.value_type == right_info.value_type
                    && matches!(
                        left_info.value_type,
                        DynamicType::Value(ValueType::Int | ValueType::Float)
                    )
                {
                    Ok(ExpressionInfo::value(ValueType::Bool, protection))
                } else {
                    Err(CheckError::InvalidOperation(format!(
                        "ordered comparison requires matching Int or Float values, got {left_info} and {right_info}"
                    )))
                }
            }
            BinaryOperator::And | BinaryOperator::Or => {
                if left_info.value_type == DynamicType::Value(ValueType::Bool)
                    && right_info.value_type == DynamicType::Value(ValueType::Bool)
                {
                    Ok(ExpressionInfo::value(ValueType::Bool, protection))
                } else {
                    Err(CheckError::InvalidOperation(format!(
                        "logical operation requires Bool values, got {left_info} and {right_info}"
                    )))
                }
            }
        }
    }

    fn ensure_assignable(
        &self,
        name: &str,
        actual: ExpressionInfo,
        expected_type: ValueType,
        target_protection: ProtectionLevel,
    ) -> Result<(), CheckError> {
        if actual.protection > target_protection {
            return Err(CheckError::ProtectionDowngrade {
                name: name.to_owned(),
                source: actual.protection,
                target: target_protection,
            });
        }
        if !actual.is_assignable_to(expected_type, target_protection) {
            return Err(CheckError::TypeMismatch {
                name: name.to_owned(),
                expected: expected_type,
                actual: actual.reported_type(),
            });
        }
        Ok(())
    }

    fn with_scope<T>(
        &mut self,
        body: impl FnOnce(&mut Self) -> Result<T, CheckError>,
    ) -> Result<T, CheckError> {
        self.vtime_scopes.push(HashMap::new());
        let result = body(self);
        self.vtime_scopes.pop();
        result
    }

    fn declare_vtime(&mut self, name: String, protection: ProtectionLevel) {
        if self.vtime_scopes.is_empty() {
            self.vtime_scopes.push(HashMap::new());
        }
        self.vtime_scopes
            .last_mut()
            .unwrap()
            .insert(name, protection);
    }

    fn update_vtime(&mut self, name: &str, protection: ProtectionLevel) {
        for scope in self.vtime_scopes.iter_mut().rev() {
            if let Some(value) = scope.get_mut(name) {
                *value = protection;
                return;
            }
        }
    }

    fn vtime_protection(&self, name: &str) -> Option<ProtectionLevel> {
        self.vtime_scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    fn is_vtime(&self, name: &str) -> bool {
        self.vtime_protection(name).is_some()
    }

    fn name_exists(&self, name: &str) -> bool {
        self.symbols.contains_key(name) || self.is_vtime(name)
    }
}

fn conversion_type(name: &str) -> Option<ValueType> {
    match name {
        "int" => Some(ValueType::Int),
        "float" => Some(ValueType::Float),
        "bool" => Some(ValueType::Bool),
        "str" => Some(ValueType::Str),
        "bytes" => Some(ValueType::Bytes),
        "json" => Some(ValueType::Json),
        _ => None,
    }
}

fn expression_variable_name(expression: &Expression) -> Option<&str> {
    match expression {
        Expression::Variable(name) => Some(name),
        Expression::Grouped(inner) => expression_variable_name(inner),
        _ => None,
    }
}

pub fn validate_variable_name(name: &str) -> Result<(), CheckError> {
    if name.is_empty()
        || is_reserved_word(name)
        || name.starts_with('.')
        || name.ends_with('.')
        || name.contains("..")
    {
        return Err(CheckError::InvalidName(name.to_owned()));
    }

    for part in name.split('.') {
        let mut chars = part.chars();
        let Some(first) = chars.next() else {
            return Err(CheckError::InvalidName(name.to_owned()));
        };
        if !is_ident_start(first) || !chars.all(is_ident_continue) {
            return Err(CheckError::InvalidName(name.to_owned()));
        }
    }

    Ok(())
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

fn is_reserved_word(name: &str) -> bool {
    matches!(
        name,
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
            | "add"
            | "get"
            | "pop"
            | "int"
            | "float"
            | "bool"
            | "str"
            | "bytes"
            | "json"
            | "split"
            | "len"
            | "contains"
            | "join"
            | "ord"
            | "char"
            | "pow"
            | "AVInt"
            | "AVFloat"
            | "AVBool"
            | "AVStr"
            | "AVBytes"
            | "AVJson"
            | "ASVInt"
            | "ASVFloat"
            | "ASVBool"
            | "ASVStr"
            | "ASVBytes"
            | "ASVJson"
            | "SASVInt"
            | "SASVFloat"
            | "SASVBool"
            | "SASVStr"
            | "SASVBytes"
            | "SASVJson"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decl(name: &str, kind: VariableKind, value_type: ValueType, value: Expression) -> Statement {
        Statement::VariableDecl(VariableDecl {
            kind,
            value_type,
            name: name.to_owned(),
            initial_value: value,
        })
    }

    fn av_decl(name: &str, value_type: ValueType, value: Expression) -> Statement {
        decl(name, VariableKind::Av, value_type, value)
    }

    #[test]
    fn rejects_duplicate_names_even_with_different_kinds() {
        let program = Program::new(vec![
            av_decl("x", ValueType::Int, Expression::Literal(Value::Int(4))),
            decl(
                "x",
                VariableKind::Asv,
                ValueType::Str,
                Expression::Literal(Value::Str("secret".to_owned())),
            ),
        ]);
        assert_eq!(
            validate_program(&program),
            Err(CheckError::DuplicateVariable("x".to_owned()))
        );
    }

    #[test]
    fn rejects_string_concatenation_with_plus() {
        let program = Program::new(vec![av_decl(
            "x",
            ValueType::Str,
            Expression::Binary {
                left: Box::new(Expression::Literal(Value::Str("a".to_owned()))),
                operator: BinaryOperator::Add,
                right: Box::new(Expression::Literal(Value::Str("b".to_owned()))),
            },
        )]);
        assert!(matches!(
            validate_program(&program),
            Err(CheckError::InvalidOperation(_))
        ));
    }

    #[test]
    fn allows_integer_division_to_stay_integer() {
        let program = Program::new(vec![av_decl(
            "x",
            ValueType::Int,
            Expression::Binary {
                left: Box::new(Expression::Literal(Value::Int(5))),
                operator: BinaryOperator::Div,
                right: Box::new(Expression::Literal(Value::Int(2))),
            },
        )]);
        assert_eq!(validate_program(&program), Ok(()));
    }

    #[test]
    fn allows_none_for_any_declared_type() {
        let program = Program::new(vec![
            av_decl("x", ValueType::Int, Expression::None),
            av_decl("s", ValueType::Str, Expression::None),
        ]);
        assert_eq!(validate_program(&program), Ok(()));
    }

    #[test]
    fn rejects_secret_input_for_av() {
        let program = Program::new(vec![av_decl(
            "token",
            ValueType::Str,
            Expression::SecretInput,
        )]);
        assert_eq!(
            validate_program(&program),
            Err(CheckError::SecretExpressionDenied("token".to_owned()))
        );
    }

    #[test]
    fn allows_info_about_sasv() {
        let program = Program::new(vec![
            av_decl(
                "value_type",
                ValueType::Str,
                Expression::Literal(Value::Str(String::new())),
            ),
            av_decl(
                "protection",
                ValueType::Str,
                Expression::Literal(Value::Str(String::new())),
            ),
            decl(
                "master_key",
                VariableKind::Sasv,
                ValueType::Str,
                Expression::Literal(Value::Str("raw".to_owned())),
            ),
            Statement::InfoAssignment(InfoAssignment {
                type_target: "value_type".to_owned(),
                protection_target: "protection".to_owned(),
                source: "master_key".to_owned(),
            }),
        ]);
        assert_eq!(validate_program(&program), Ok(()));
    }

    #[test]
    fn rejects_info_about_vtime() {
        let program = Program::new(vec![
            av_decl(
                "value_type",
                ValueType::Str,
                Expression::Literal(Value::Str(String::new())),
            ),
            av_decl(
                "protection",
                ValueType::Str,
                Expression::Literal(Value::Str(String::new())),
            ),
            Statement::VTimeDecl(VTimeDecl {
                name: "tmp".to_owned(),
                initial_value: Expression::Literal(Value::Int(1)),
            }),
            Statement::InfoAssignment(InfoAssignment {
                type_target: "value_type".to_owned(),
                protection_target: "protection".to_owned(),
                source: "tmp".to_owned(),
            }),
        ]);
        assert_eq!(
            validate_program(&program),
            Err(CheckError::UnknownVariable("tmp".to_owned()))
        );
    }

    #[test]
    fn rejects_out_for_secrets() {
        let program = Program::new(vec![
            decl(
                "token",
                VariableKind::Asv,
                ValueType::Str,
                Expression::Literal(Value::Str("raw".to_owned())),
            ),
            Statement::Out(Expression::Variable("token".to_owned())),
        ]);
        assert_eq!(
            validate_program(&program),
            Err(CheckError::SecretExpressionDenied(
                "Variable(\"token\")".to_owned()
            ))
        );
    }

    #[test]
    fn accepts_while_limit_minus_one_as_unlimited() {
        let program = Program::new(vec![
            av_decl("x", ValueType::Int, Expression::Literal(Value::Int(0))),
            Statement::While(WhileStatement {
                condition: Expression::Binary {
                    left: Box::new(Expression::Variable("x".to_owned())),
                    operator: BinaryOperator::Less,
                    right: Box::new(Expression::Literal(Value::Int(10))),
                },
                limit: -1,
                body: vec![Statement::Break],
            }),
        ]);
        assert_eq!(validate_program(&program), Ok(()));
    }

    #[test]
    fn rejects_loop_limits_below_minus_one() {
        let program = Program::new(vec![
            av_decl("x", ValueType::Int, Expression::Literal(Value::Int(0))),
            Statement::While(WhileStatement {
                condition: Expression::Binary {
                    left: Box::new(Expression::Variable("x".to_owned())),
                    operator: BinaryOperator::Less,
                    right: Box::new(Expression::Literal(Value::Int(10))),
                },
                limit: -2,
                body: vec![],
            }),
        ]);
        assert_eq!(
            validate_program(&program),
            Err(CheckError::InvalidLoopLimit(-2))
        );
    }

    #[test]
    fn rejects_break_outside_loop() {
        assert_eq!(
            validate_program(&Program::new(vec![Statement::Break])),
            Err(CheckError::BreakOutsideLoop)
        );
    }

    #[test]
    fn function_declared_in_loop_cannot_break_the_enclosing_loop() {
        let program = Program::new(vec![Statement::While(WhileStatement {
            condition: Expression::Literal(Value::Bool(true)),
            limit: 1,
            body: vec![Statement::FunctionDecl(FunctionDecl {
                name: "nested".to_owned(),
                params: vec![],
                body: vec![Statement::Break],
            })],
        })]);

        assert_eq!(
            validate_program(&program),
            Err(CheckError::BreakOutsideLoop)
        );
    }

    #[test]
    fn vtime_is_scoped_and_flexible() {
        let program = Program::new(vec![
            av_decl("x", ValueType::Int, Expression::Literal(Value::Int(0))),
            Statement::While(WhileStatement {
                condition: Expression::Binary {
                    left: Box::new(Expression::Variable("x".to_owned())),
                    operator: BinaryOperator::Less,
                    right: Box::new(Expression::Literal(Value::Int(10))),
                },
                limit: 3,
                body: vec![
                    Statement::VTimeDecl(VTimeDecl {
                        name: "tmp".to_owned(),
                        initial_value: Expression::Literal(Value::Str("a".to_owned())),
                    }),
                    Statement::Assignment(Assignment {
                        name: "tmp".to_owned(),
                        operator: AssignmentOperator::Assign,
                        value: Expression::Literal(Value::Int(1)),
                    }),
                ],
            }),
        ]);
        assert_eq!(validate_program(&program), Ok(()));
    }

    #[test]
    fn pick_creates_scoped_vtime_item() {
        let program = Program::new(vec![
            av_decl(
                "text",
                ValueType::Str,
                Expression::Literal(Value::Str("abc".to_owned())),
            ),
            Statement::Pick(PickStatement {
                value: Expression::Variable("text".to_owned()),
                item_name: "ch".to_owned(),
                body: vec![Statement::Out(Expression::Variable("ch".to_owned()))],
            }),
        ]);
        assert_eq!(validate_program(&program), Ok(()));
    }

    #[test]
    fn checks_functions_and_returns() {
        let program = Program::new(vec![
            Statement::FunctionDecl(FunctionDecl {
                name: "normalize".to_owned(),
                params: vec!["raw".to_owned()],
                body: vec![Statement::Return(Expression::Variable("raw".to_owned()))],
            }),
            av_decl(
                "result",
                ValueType::Str,
                Expression::Call {
                    name: "normalize".to_owned(),
                    args: vec![Expression::Literal(Value::Str("ok".to_owned()))],
                },
            ),
        ]);
        assert_eq!(validate_program(&program), Ok(()));
    }

    #[test]
    fn function_parameters_shadow_values_but_not_function_calls() {
        let program = Program::new(vec![
            av_decl(
                "value",
                ValueType::Str,
                Expression::Literal(Value::Str("global".to_owned())),
            ),
            Statement::FunctionDecl(FunctionDecl {
                name: "increment".to_owned(),
                params: vec!["value".to_owned()],
                body: vec![Statement::Return(Expression::Binary {
                    left: Box::new(Expression::Variable("value".to_owned())),
                    operator: BinaryOperator::Add,
                    right: Box::new(Expression::Literal(Value::Int(1))),
                })],
            }),
            Statement::FunctionDecl(FunctionDecl {
                name: "apply".to_owned(),
                params: vec!["increment".to_owned()],
                body: vec![Statement::Return(Expression::Call {
                    name: "increment".to_owned(),
                    args: vec![Expression::Variable("increment".to_owned())],
                })],
            }),
        ]);

        assert_eq!(validate_program(&program), Ok(()));
    }

    #[test]
    fn shadowed_absolutes_are_not_valid_absolute_operation_targets() {
        let shadowed_secretup = Program::new(vec![
            av_decl("value", ValueType::Int, Expression::Literal(Value::Int(1))),
            Statement::FunctionDecl(FunctionDecl {
                name: "promote".to_owned(),
                params: vec!["value".to_owned()],
                body: vec![Statement::SecretUp("value".to_owned())],
            }),
        ]);
        assert_eq!(
            validate_program(&shadowed_secretup),
            Err(CheckError::InvalidAssignmentTarget("value".to_owned()))
        );

        let shadowed_info_target = Program::new(vec![
            av_decl(
                "metadata",
                ValueType::Str,
                Expression::Literal(Value::Str(String::new())),
            ),
            decl(
                "master_key",
                VariableKind::Sasv,
                ValueType::Int,
                Expression::Literal(Value::Int(1)),
            ),
            Statement::FunctionDecl(FunctionDecl {
                name: "inspect".to_owned(),
                params: vec!["metadata".to_owned()],
                body: vec![Statement::InfoAssignment(InfoAssignment {
                    type_target: "metadata".to_owned(),
                    protection_target: "metadata".to_owned(),
                    source: "master_key".to_owned(),
                })],
            }),
        ]);
        assert_eq!(
            validate_program(&shadowed_info_target),
            Err(CheckError::InvalidAssignmentTarget("metadata".to_owned()))
        );

        let shadowed_self = Program::new(vec![
            av_decl("value", ValueType::Int, Expression::Literal(Value::Int(1))),
            Statement::FunctionDecl(FunctionDecl {
                name: "same".to_owned(),
                params: vec!["value".to_owned()],
                body: vec![Statement::Return(Expression::SelfCheck("value".to_owned()))],
            }),
        ]);
        assert_eq!(
            validate_program(&shadowed_self),
            Err(CheckError::InvalidOperation(
                "`=self=` is only available for absolute variables".to_owned()
            ))
        );
    }

    #[test]
    fn rejects_return_outside_function() {
        assert_eq!(
            validate_program(&Program::new(vec![Statement::Return(Expression::None)])),
            Err(CheckError::ReturnOutsideFunction)
        );
    }

    #[test]
    fn checks_conversions_as_typed_values() {
        let program = Program::new(vec![av_decl(
            "x",
            ValueType::Int,
            Expression::Call {
                name: "int".to_owned(),
                args: vec![Expression::Literal(Value::Str("123".to_owned()))],
            },
        )]);
        assert_eq!(validate_program(&program), Ok(()));
    }

    #[test]
    fn secretup_only_moves_up() {
        let program = Program::new(vec![
            av_decl(
                "token",
                ValueType::Str,
                Expression::Literal(Value::Str("raw".to_owned())),
            ),
            Statement::SecretUp("token".to_owned()),
            Statement::SecretUp("token".to_owned()),
        ]);
        assert_eq!(validate_program(&program), Ok(()));
    }

    #[test]
    fn supports_list_literals_and_tagged_elements() {
        let program = Program::new(vec![Statement::ListDecl(ListDecl {
            name: "items".to_owned(),
            elements: vec![
                ListElement {
                    value: Expression::Literal(Value::Int(1)),
                    protection: Some(ProtectionLevel::Sasv),
                },
                ListElement {
                    value: Expression::Literal(Value::Str("abc".to_owned())),
                    protection: Some(ProtectionLevel::Asv),
                },
            ],
        })]);
        assert_eq!(validate_program(&program), Ok(()));
    }

    #[test]
    fn list_mutations_require_named_targets_and_track_vtime_protection() {
        let protected_vtime = Program::new(vec![
            Statement::VTimeDecl(VTimeDecl {
                name: "items".to_owned(),
                initial_value: Expression::ListLiteral(vec![ListElement {
                    value: Expression::Literal(Value::Int(1)),
                    protection: None,
                }]),
            }),
            decl(
                "hidden",
                VariableKind::Asv,
                ValueType::Int,
                Expression::Literal(Value::Int(2)),
            ),
            Statement::AddToList {
                list_name: "items".to_owned(),
                element: ListElement {
                    value: Expression::Variable("hidden".to_owned()),
                    protection: None,
                },
            },
            Statement::Out(Expression::Variable("items".to_owned())),
        ]);
        assert!(matches!(
            validate_program(&protected_vtime),
            Err(CheckError::SecretExpressionDenied(_))
        ));

        let temporary_pop = Program::new(vec![Statement::VTimeDecl(VTimeDecl {
            name: "item".to_owned(),
            initial_value: Expression::Call {
                name: "pop".to_owned(),
                args: vec![Expression::ListLiteral(vec![ListElement {
                    value: Expression::Literal(Value::Int(1)),
                    protection: None,
                }])],
            },
        })]);
        assert_eq!(
            validate_program(&temporary_pop),
            Err(CheckError::InvalidOperation(
                "pop(list) requires a named List or VTime target".to_owned()
            ))
        );

        let expression_add = Program::new(vec![Statement::VTimeDecl(VTimeDecl {
            name: "result".to_owned(),
            initial_value: Expression::Call {
                name: "add".to_owned(),
                args: vec![
                    Expression::ListLiteral(vec![]),
                    Expression::Literal(Value::Int(1)),
                ],
            },
        })]);
        assert_eq!(
            validate_program(&expression_add),
            Err(CheckError::InvalidOperation(
                "add(list, value) is only valid as a statement".to_owned()
            ))
        );
    }

    #[test]
    fn blocks_secret_vtime_to_av_assignment() {
        let program = Program::new(vec![
            decl(
                "secret_value",
                VariableKind::Asv,
                ValueType::Str,
                Expression::Literal(Value::Str("s".to_owned())),
            ),
            Statement::VTimeDecl(VTimeDecl {
                name: "item".to_owned(),
                initial_value: Expression::Variable("secret_value".to_owned()),
            }),
            av_decl(
                "public",
                ValueType::Str,
                Expression::Variable("item".to_owned()),
            ),
        ]);
        assert_eq!(
            validate_program(&program),
            Err(CheckError::ProtectionDowngrade {
                name: "public".to_owned(),
                source: ProtectionLevel::Asv,
                target: ProtectionLevel::Av,
            })
        );
    }

    #[test]
    fn allows_secret_vtime_to_sasv_assignment() {
        let program = Program::new(vec![
            decl(
                "secret_value",
                VariableKind::Asv,
                ValueType::Str,
                Expression::Literal(Value::Str("s".to_owned())),
            ),
            Statement::VTimeDecl(VTimeDecl {
                name: "item".to_owned(),
                initial_value: Expression::Variable("secret_value".to_owned()),
            }),
            decl(
                "sealed",
                VariableKind::Sasv,
                ValueType::Str,
                Expression::Variable("item".to_owned()),
            ),
        ]);
        assert_eq!(validate_program(&program), Ok(()));
    }
}
