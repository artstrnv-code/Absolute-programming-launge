use apl_core::{
    Assignment, AssignmentOperator, BinaryOperator, ElseIfBranch, Expression, FunctionDecl,
    IfStatement, ListDecl, ListElement, PickStatement, Program, ProtectionLevel, Statement,
    UnaryOperator, VTimeDecl, Value, ValueType, VariableDecl, VariableKind, WhileStatement,
};

const MAGIC: &[u8; 8] = b"APLIR001";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrError {
    InvalidMagic,
    UnexpectedEof,
    InvalidUtf8,
    InvalidTag { kind: &'static str, tag: u8 },
    TrailingBytes,
    LengthOverflow,
}

pub fn encode_program(program: &Program) -> Result<Vec<u8>, IrError> {
    let mut writer = Writer { bytes: Vec::new() };
    writer.bytes.extend_from_slice(MAGIC);
    writer.statements(&program.statements)?;
    Ok(writer.bytes)
}

pub fn decode_program(bytes: &[u8]) -> Result<Program, IrError> {
    let mut reader = Reader { bytes, cursor: 0 };
    if reader.take(MAGIC.len())? != MAGIC {
        return Err(IrError::InvalidMagic);
    }
    let statements = reader.statements()?;
    if reader.cursor != bytes.len() {
        return Err(IrError::TrailingBytes);
    }
    Ok(Program::new(statements))
}

struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    fn u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    fn i64(&mut self, value: i64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn f64(&mut self, value: f64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn bool(&mut self, value: bool) {
        self.u8(u8::from(value));
    }

    fn len(&mut self, value: usize) -> Result<(), IrError> {
        let value = u32::try_from(value).map_err(|_| IrError::LengthOverflow)?;
        self.bytes.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }

    fn string(&mut self, value: &str) -> Result<(), IrError> {
        self.bytes(value.as_bytes())
    }

    fn bytes(&mut self, value: &[u8]) -> Result<(), IrError> {
        self.len(value.len())?;
        self.bytes.extend_from_slice(value);
        Ok(())
    }

    fn statements(&mut self, statements: &[Statement]) -> Result<(), IrError> {
        self.len(statements.len())?;
        for statement in statements {
            self.statement(statement)?;
        }
        Ok(())
    }

    fn statement(&mut self, statement: &Statement) -> Result<(), IrError> {
        match statement {
            Statement::VariableDecl(decl) => {
                self.u8(0);
                self.variable_decl(decl)
            }
            Statement::ListDecl(decl) => {
                self.u8(1);
                self.string(&decl.name)?;
                self.list_elements(&decl.elements)
            }
            Statement::VTimeDecl(decl) => {
                self.u8(2);
                self.string(&decl.name)?;
                self.expression(&decl.initial_value)
            }
            Statement::FunctionDecl(function) => {
                self.u8(3);
                self.string(&function.name)?;
                self.len(function.params.len())?;
                for param in &function.params {
                    self.string(param)?;
                }
                self.statements(&function.body)
            }
            Statement::Assignment(assignment) => {
                self.u8(4);
                self.string(&assignment.name)?;
                self.assignment_operator(assignment.operator);
                self.expression(&assignment.value)
            }
            Statement::InfoAssignment(assignment) => {
                self.u8(5);
                self.string(&assignment.type_target)?;
                self.string(&assignment.protection_target)?;
                self.string(&assignment.source)
            }
            Statement::AddToList { list_name, element } => {
                self.u8(6);
                self.string(list_name)?;
                self.list_element(element)
            }
            Statement::SecretUp(name) => {
                self.u8(7);
                self.string(name)
            }
            Statement::Out(expression) => {
                self.u8(8);
                self.expression(expression)
            }
            Statement::Stop(reason) => {
                self.u8(9);
                self.option_expression(reason.as_ref())
            }
            Statement::Fail(reason) => {
                self.u8(10);
                self.expression(reason)
            }
            Statement::Return(expression) => {
                self.u8(11);
                self.expression(expression)
            }
            Statement::If(if_statement) => {
                self.u8(12);
                self.if_statement(if_statement)
            }
            Statement::While(while_statement) => {
                self.u8(13);
                self.expression(&while_statement.condition)?;
                self.i64(while_statement.limit);
                self.statements(&while_statement.body)
            }
            Statement::Pick(pick_statement) => {
                self.u8(14);
                self.expression(&pick_statement.value)?;
                self.string(&pick_statement.item_name)?;
                self.statements(&pick_statement.body)
            }
            Statement::Break => {
                self.u8(15);
                Ok(())
            }
            Statement::Continue => {
                self.u8(16);
                Ok(())
            }
        }
    }

    fn variable_decl(&mut self, decl: &VariableDecl) -> Result<(), IrError> {
        self.variable_kind(decl.kind);
        self.value_type(decl.value_type);
        self.string(&decl.name)?;
        self.expression(&decl.initial_value)
    }

    fn if_statement(&mut self, if_statement: &IfStatement) -> Result<(), IrError> {
        self.expression(&if_statement.condition)?;
        self.statements(&if_statement.body)?;
        self.len(if_statement.else_if_branches.len())?;
        for branch in &if_statement.else_if_branches {
            self.expression(&branch.condition)?;
            self.statements(&branch.body)?;
        }
        self.option_statements(if_statement.else_body.as_deref())
    }

    fn list_elements(&mut self, elements: &[ListElement]) -> Result<(), IrError> {
        self.len(elements.len())?;
        for element in elements {
            self.list_element(element)?;
        }
        Ok(())
    }

    fn list_element(&mut self, element: &ListElement) -> Result<(), IrError> {
        self.expression(&element.value)?;
        self.option_protection(element.protection);
        Ok(())
    }

    fn expression(&mut self, expression: &Expression) -> Result<(), IrError> {
        match expression {
            Expression::Literal(value) => {
                self.u8(0);
                self.value(value)
            }
            Expression::ListLiteral(elements) => {
                self.u8(1);
                self.list_elements(elements)
            }
            Expression::None => {
                self.u8(2);
                Ok(())
            }
            Expression::Input => {
                self.u8(3);
                Ok(())
            }
            Expression::SecretInput => {
                self.u8(4);
                Ok(())
            }
            Expression::Variable(name) => {
                self.u8(5);
                self.string(name)
            }
            Expression::Call { name, args } => {
                self.u8(6);
                self.string(name)?;
                self.len(args.len())?;
                for arg in args {
                    self.expression(arg)?;
                }
                Ok(())
            }
            Expression::Index { target, index } => {
                self.u8(7);
                self.expression(target)?;
                self.expression(index)
            }
            Expression::Slice {
                target,
                start,
                end,
                step,
            } => {
                self.u8(8);
                self.expression(target)?;
                self.option_boxed_expression(start.as_deref())?;
                self.option_boxed_expression(end.as_deref())?;
                self.option_boxed_expression(step.as_deref())
            }
            Expression::Unary {
                operator,
                expression,
            } => {
                self.u8(9);
                self.unary_operator(*operator);
                self.expression(expression)
            }
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                self.u8(10);
                self.expression(left)?;
                self.binary_operator(*operator);
                self.expression(right)
            }
            Expression::SelfCheck(name) => {
                self.u8(11);
                self.string(name)
            }
            Expression::Grouped(expression) => {
                self.u8(12);
                self.expression(expression)
            }
        }
    }

    fn value(&mut self, value: &Value) -> Result<(), IrError> {
        match value {
            Value::Int(value) => {
                self.u8(0);
                self.i64(*value);
                Ok(())
            }
            Value::Float(value) => {
                self.u8(1);
                self.f64(*value);
                Ok(())
            }
            Value::Bool(value) => {
                self.u8(2);
                self.bool(*value);
                Ok(())
            }
            Value::Str(value) => {
                self.u8(3);
                self.string(value)
            }
            Value::Bytes(value) => {
                self.u8(4);
                self.bytes(value)
            }
            Value::Json(value) => {
                self.u8(5);
                self.string(value)
            }
        }
    }

    fn option_expression(&mut self, expression: Option<&Expression>) -> Result<(), IrError> {
        match expression {
            Some(expression) => {
                self.u8(1);
                self.expression(expression)
            }
            None => {
                self.u8(0);
                Ok(())
            }
        }
    }

    fn option_boxed_expression(&mut self, expression: Option<&Expression>) -> Result<(), IrError> {
        self.option_expression(expression)
    }

    fn option_statements(&mut self, statements: Option<&[Statement]>) -> Result<(), IrError> {
        match statements {
            Some(statements) => {
                self.u8(1);
                self.statements(statements)
            }
            None => {
                self.u8(0);
                Ok(())
            }
        }
    }

    fn option_protection(&mut self, protection: Option<ProtectionLevel>) {
        match protection {
            Some(protection) => {
                self.u8(1);
                self.protection(protection);
            }
            None => self.u8(0),
        }
    }

    fn variable_kind(&mut self, value: VariableKind) {
        self.u8(match value {
            VariableKind::Av => 0,
            VariableKind::Asv => 1,
            VariableKind::Sasv => 2,
        });
    }

    fn protection(&mut self, value: ProtectionLevel) {
        self.u8(match value {
            ProtectionLevel::Av => 0,
            ProtectionLevel::Asv => 1,
            ProtectionLevel::Sasv => 2,
        });
    }

    fn value_type(&mut self, value: ValueType) {
        self.u8(match value {
            ValueType::Int => 0,
            ValueType::Float => 1,
            ValueType::Bool => 2,
            ValueType::Str => 3,
            ValueType::Bytes => 4,
            ValueType::Json => 5,
            ValueType::List => 6,
        });
    }

    fn assignment_operator(&mut self, value: AssignmentOperator) {
        self.u8(match value {
            AssignmentOperator::Assign => 0,
            AssignmentOperator::AddAssign => 1,
            AssignmentOperator::SubAssign => 2,
            AssignmentOperator::MulAssign => 3,
            AssignmentOperator::DivAssign => 4,
        });
    }

    fn unary_operator(&mut self, value: UnaryOperator) {
        self.u8(match value {
            UnaryOperator::Negate => 0,
            UnaryOperator::Not => 1,
        });
    }

    fn binary_operator(&mut self, value: BinaryOperator) {
        self.u8(match value {
            BinaryOperator::Add => 0,
            BinaryOperator::Sub => 1,
            BinaryOperator::Mul => 2,
            BinaryOperator::Div => 3,
            BinaryOperator::Equal => 4,
            BinaryOperator::NotEqual => 5,
            BinaryOperator::Greater => 6,
            BinaryOperator::GreaterEqual => 7,
            BinaryOperator::Less => 8,
            BinaryOperator::LessEqual => 9,
            BinaryOperator::And => 10,
            BinaryOperator::Or => 11,
        });
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8], IrError> {
        let end = self
            .cursor
            .checked_add(count)
            .ok_or(IrError::UnexpectedEof)?;
        let Some(slice) = self.bytes.get(self.cursor..end) else {
            return Err(IrError::UnexpectedEof);
        };
        self.cursor = end;
        Ok(slice)
    }

    fn u8(&mut self) -> Result<u8, IrError> {
        Ok(self.take(1)?[0])
    }

    fn i64(&mut self) -> Result<i64, IrError> {
        let mut bytes = [0; 8];
        bytes.copy_from_slice(self.take(8)?);
        Ok(i64::from_le_bytes(bytes))
    }

    fn f64(&mut self) -> Result<f64, IrError> {
        let mut bytes = [0; 8];
        bytes.copy_from_slice(self.take(8)?);
        Ok(f64::from_le_bytes(bytes))
    }

    fn bool(&mut self) -> Result<bool, IrError> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            tag => Err(IrError::InvalidTag { kind: "Bool", tag }),
        }
    }

    fn len(&mut self) -> Result<usize, IrError> {
        let mut bytes = [0; 4];
        bytes.copy_from_slice(self.take(4)?);
        Ok(u32::from_le_bytes(bytes) as usize)
    }

    fn string(&mut self) -> Result<String, IrError> {
        String::from_utf8(self.bytes()?.to_vec()).map_err(|_| IrError::InvalidUtf8)
    }

    fn bytes(&mut self) -> Result<&'a [u8], IrError> {
        let len = self.len()?;
        self.take(len)
    }

    fn statements(&mut self) -> Result<Vec<Statement>, IrError> {
        let len = self.len()?;
        let mut statements = Vec::with_capacity(len);
        for _ in 0..len {
            statements.push(self.statement()?);
        }
        Ok(statements)
    }

    fn statement(&mut self) -> Result<Statement, IrError> {
        match self.u8()? {
            0 => Ok(Statement::VariableDecl(self.variable_decl()?)),
            1 => Ok(Statement::ListDecl(ListDecl {
                name: self.string()?,
                elements: self.list_elements()?,
            })),
            2 => Ok(Statement::VTimeDecl(VTimeDecl {
                name: self.string()?,
                initial_value: self.expression()?,
            })),
            3 => Ok(Statement::FunctionDecl(self.function_decl()?)),
            4 => Ok(Statement::Assignment(Assignment {
                name: self.string()?,
                operator: self.assignment_operator()?,
                value: self.expression()?,
            })),
            5 => Ok(Statement::InfoAssignment(apl_core::InfoAssignment {
                type_target: self.string()?,
                protection_target: self.string()?,
                source: self.string()?,
            })),
            6 => Ok(Statement::AddToList {
                list_name: self.string()?,
                element: self.list_element()?,
            }),
            7 => Ok(Statement::SecretUp(self.string()?)),
            8 => Ok(Statement::Out(self.expression()?)),
            9 => Ok(Statement::Stop(self.option_expression()?)),
            10 => Ok(Statement::Fail(self.expression()?)),
            11 => Ok(Statement::Return(self.expression()?)),
            12 => Ok(Statement::If(self.if_statement()?)),
            13 => Ok(Statement::While(WhileStatement {
                condition: self.expression()?,
                limit: self.i64()?,
                body: self.statements()?,
            })),
            14 => Ok(Statement::Pick(PickStatement {
                value: self.expression()?,
                item_name: self.string()?,
                body: self.statements()?,
            })),
            15 => Ok(Statement::Break),
            16 => Ok(Statement::Continue),
            tag => Err(IrError::InvalidTag {
                kind: "Statement",
                tag,
            }),
        }
    }

    fn variable_decl(&mut self) -> Result<VariableDecl, IrError> {
        Ok(VariableDecl {
            kind: self.variable_kind()?,
            value_type: self.value_type()?,
            name: self.string()?,
            initial_value: self.expression()?,
        })
    }

    fn function_decl(&mut self) -> Result<FunctionDecl, IrError> {
        let name = self.string()?;
        let param_count = self.len()?;
        let mut params = Vec::with_capacity(param_count);
        for _ in 0..param_count {
            params.push(self.string()?);
        }
        Ok(FunctionDecl {
            name,
            params,
            body: self.statements()?,
        })
    }

    fn if_statement(&mut self) -> Result<IfStatement, IrError> {
        let condition = self.expression()?;
        let body = self.statements()?;
        let branch_count = self.len()?;
        let mut else_if_branches = Vec::with_capacity(branch_count);
        for _ in 0..branch_count {
            else_if_branches.push(ElseIfBranch {
                condition: self.expression()?,
                body: self.statements()?,
            });
        }
        Ok(IfStatement {
            condition,
            body,
            else_if_branches,
            else_body: self.option_statements()?,
        })
    }

    fn list_elements(&mut self) -> Result<Vec<ListElement>, IrError> {
        let len = self.len()?;
        let mut elements = Vec::with_capacity(len);
        for _ in 0..len {
            elements.push(self.list_element()?);
        }
        Ok(elements)
    }

    fn list_element(&mut self) -> Result<ListElement, IrError> {
        Ok(ListElement {
            value: self.expression()?,
            protection: self.option_protection()?,
        })
    }

    fn expression(&mut self) -> Result<Expression, IrError> {
        match self.u8()? {
            0 => Ok(Expression::Literal(self.value()?)),
            1 => Ok(Expression::ListLiteral(self.list_elements()?)),
            2 => Ok(Expression::None),
            3 => Ok(Expression::Input),
            4 => Ok(Expression::SecretInput),
            5 => Ok(Expression::Variable(self.string()?)),
            6 => {
                let name = self.string()?;
                let arg_count = self.len()?;
                let mut args = Vec::with_capacity(arg_count);
                for _ in 0..arg_count {
                    args.push(self.expression()?);
                }
                Ok(Expression::Call { name, args })
            }
            7 => Ok(Expression::Index {
                target: Box::new(self.expression()?),
                index: Box::new(self.expression()?),
            }),
            8 => Ok(Expression::Slice {
                target: Box::new(self.expression()?),
                start: self.option_boxed_expression()?,
                end: self.option_boxed_expression()?,
                step: self.option_boxed_expression()?,
            }),
            9 => Ok(Expression::Unary {
                operator: self.unary_operator()?,
                expression: Box::new(self.expression()?),
            }),
            10 => Ok(Expression::Binary {
                left: Box::new(self.expression()?),
                operator: self.binary_operator()?,
                right: Box::new(self.expression()?),
            }),
            11 => Ok(Expression::SelfCheck(self.string()?)),
            12 => Ok(Expression::Grouped(Box::new(self.expression()?))),
            tag => Err(IrError::InvalidTag {
                kind: "Expression",
                tag,
            }),
        }
    }

    fn value(&mut self) -> Result<Value, IrError> {
        match self.u8()? {
            0 => Ok(Value::Int(self.i64()?)),
            1 => Ok(Value::Float(self.f64()?)),
            2 => Ok(Value::Bool(self.bool()?)),
            3 => Ok(Value::Str(self.string()?)),
            4 => Ok(Value::Bytes(self.bytes()?.to_vec())),
            5 => Ok(Value::Json(self.string()?)),
            tag => Err(IrError::InvalidTag { kind: "Value", tag }),
        }
    }

    fn option_expression(&mut self) -> Result<Option<Expression>, IrError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.expression()?)),
            tag => Err(IrError::InvalidTag {
                kind: "Option",
                tag,
            }),
        }
    }

    fn option_boxed_expression(&mut self) -> Result<Option<Box<Expression>>, IrError> {
        Ok(self.option_expression()?.map(Box::new))
    }

    fn option_statements(&mut self) -> Result<Option<Vec<Statement>>, IrError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.statements()?)),
            tag => Err(IrError::InvalidTag {
                kind: "Option",
                tag,
            }),
        }
    }

    fn option_protection(&mut self) -> Result<Option<ProtectionLevel>, IrError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.protection()?)),
            tag => Err(IrError::InvalidTag {
                kind: "Option",
                tag,
            }),
        }
    }

    fn variable_kind(&mut self) -> Result<VariableKind, IrError> {
        match self.u8()? {
            0 => Ok(VariableKind::Av),
            1 => Ok(VariableKind::Asv),
            2 => Ok(VariableKind::Sasv),
            tag => Err(IrError::InvalidTag {
                kind: "VariableKind",
                tag,
            }),
        }
    }

    fn protection(&mut self) -> Result<ProtectionLevel, IrError> {
        match self.u8()? {
            0 => Ok(ProtectionLevel::Av),
            1 => Ok(ProtectionLevel::Asv),
            2 => Ok(ProtectionLevel::Sasv),
            tag => Err(IrError::InvalidTag {
                kind: "ProtectionLevel",
                tag,
            }),
        }
    }

    fn value_type(&mut self) -> Result<ValueType, IrError> {
        match self.u8()? {
            0 => Ok(ValueType::Int),
            1 => Ok(ValueType::Float),
            2 => Ok(ValueType::Bool),
            3 => Ok(ValueType::Str),
            4 => Ok(ValueType::Bytes),
            5 => Ok(ValueType::Json),
            6 => Ok(ValueType::List),
            tag => Err(IrError::InvalidTag {
                kind: "ValueType",
                tag,
            }),
        }
    }

    fn assignment_operator(&mut self) -> Result<AssignmentOperator, IrError> {
        match self.u8()? {
            0 => Ok(AssignmentOperator::Assign),
            1 => Ok(AssignmentOperator::AddAssign),
            2 => Ok(AssignmentOperator::SubAssign),
            3 => Ok(AssignmentOperator::MulAssign),
            4 => Ok(AssignmentOperator::DivAssign),
            tag => Err(IrError::InvalidTag {
                kind: "AssignmentOperator",
                tag,
            }),
        }
    }

    fn unary_operator(&mut self) -> Result<UnaryOperator, IrError> {
        match self.u8()? {
            0 => Ok(UnaryOperator::Negate),
            1 => Ok(UnaryOperator::Not),
            tag => Err(IrError::InvalidTag {
                kind: "UnaryOperator",
                tag,
            }),
        }
    }

    fn binary_operator(&mut self) -> Result<BinaryOperator, IrError> {
        match self.u8()? {
            0 => Ok(BinaryOperator::Add),
            1 => Ok(BinaryOperator::Sub),
            2 => Ok(BinaryOperator::Mul),
            3 => Ok(BinaryOperator::Div),
            4 => Ok(BinaryOperator::Equal),
            5 => Ok(BinaryOperator::NotEqual),
            6 => Ok(BinaryOperator::Greater),
            7 => Ok(BinaryOperator::GreaterEqual),
            8 => Ok(BinaryOperator::Less),
            9 => Ok(BinaryOperator::LessEqual),
            10 => Ok(BinaryOperator::And),
            11 => Ok(BinaryOperator::Or),
            tag => Err(IrError::InvalidTag {
                kind: "BinaryOperator",
                tag,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_program() {
        let program = Program::new(vec![
            Statement::FunctionDecl(FunctionDecl {
                name: "apl.id".to_owned(),
                params: vec!["value".to_owned()],
                body: vec![Statement::Return(Expression::Variable("value".to_owned()))],
            }),
            Statement::VariableDecl(VariableDecl {
                kind: VariableKind::Av,
                value_type: ValueType::Int,
                name: "x".to_owned(),
                initial_value: Expression::Call {
                    name: "apl.id".to_owned(),
                    args: vec![Expression::Literal(Value::Int(7))],
                },
            }),
            Statement::Out(Expression::Variable("x".to_owned())),
        ]);

        let encoded = encode_program(&program).unwrap();
        let decoded = decode_program(&encoded).unwrap();

        assert_eq!(decoded, program);
    }

    #[test]
    fn rejects_bad_magic() {
        assert_eq!(decode_program(b"nope"), Err(IrError::UnexpectedEof));
    }
}
