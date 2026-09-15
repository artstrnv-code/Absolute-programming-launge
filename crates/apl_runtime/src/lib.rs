use apl_core::{
    AssignmentOperator, BinaryOperator, Expression, FunctionDecl, InfoAssignment, Program,
    ProtectionLevel, Statement, UnaryOperator, Value as AstValue, ValueType, VariableKind,
};
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, PartialEq)]
pub struct RunOutput {
    pub stdout: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeError {
    Message(String),
    Failed(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompiledProgram {
    entry: BlockId,
    functions: HashMap<String, CompiledFunction>,
    blocks: Vec<CompiledBlock>,
    code: Vec<StatementOp>,
}

#[derive(Debug, Clone, PartialEq)]
struct CompiledFunction {
    params: Vec<String>,
    body: BlockId,
}

#[derive(Debug, Clone, PartialEq)]
struct CompiledBlock {
    start: usize,
    end: usize,
}

type BlockId = usize;

#[derive(Debug, Clone, PartialEq)]
enum StatementOp {
    VariableDecl(CompiledVariableDecl),
    ListDecl(CompiledListDecl),
    VTimeDecl(CompiledVTimeDecl),
    Assignment(CompiledAssignment),
    InfoAssignment(InfoAssignment),
    AddToList {
        list_name: String,
        element: CompiledListElement,
    },
    SecretUp(String),
    Out(CompiledExpression),
    Stop(Option<CompiledExpression>),
    Fail(CompiledExpression),
    Return(CompiledExpression),
    Jump {
        target_pc: usize,
    },
    JumpIfFalse {
        condition: CompiledExpression,
        target_pc: usize,
    },
    ExecScopedBlock(BlockId),
    LoopCheck {
        condition: CompiledExpression,
        limit: i64,
        exit_pc: usize,
    },
    ExecLoopBody {
        body: BlockId,
        continue_pc: usize,
        break_pc: usize,
    },
    Pick {
        value: CompiledExpression,
        item_name: String,
        body: BlockId,
    },
    Break,
    Continue,
}

#[derive(Debug, Clone, PartialEq)]
struct CompiledVariableDecl {
    kind: VariableKind,
    value_type: ValueType,
    name: String,
    initial_value: CompiledExpression,
}

#[derive(Debug, Clone, PartialEq)]
struct CompiledListDecl {
    name: String,
    elements: Vec<CompiledListElement>,
}

#[derive(Debug, Clone, PartialEq)]
struct CompiledVTimeDecl {
    name: String,
    initial_value: CompiledExpression,
}

#[derive(Debug, Clone, PartialEq)]
struct CompiledAssignment {
    name: String,
    operator: AssignmentOperator,
    value: CompiledExpression,
}

#[derive(Debug, Clone, PartialEq)]
struct CompiledListElement {
    value: CompiledExpression,
    protection: Option<ProtectionLevel>,
}

#[derive(Debug, Clone, PartialEq)]
struct CompiledExpression {
    ops: Vec<ExpressionOp>,
    form: ExpressionForm,
}

#[derive(Debug, Clone, PartialEq)]
enum ExpressionForm {
    Other,
    Input,
    SecretInput,
    Variable(String),
}

#[derive(Debug, Clone, PartialEq)]
enum ExpressionOp {
    PushLiteral(AstValue),
    PushNone,
    ReadInput,
    ReadSecretInput,
    LoadVariable(String),
    Call {
        name: String,
        arg_count: usize,
        arg_forms: Vec<ExpressionForm>,
    },
    Index,
    Slice {
        has_start: bool,
        has_end: bool,
        has_step: bool,
    },
    Unary(UnaryOperator),
    Binary(BinaryOperator),
    SelfCheck(String),
    BuildList(Vec<Option<ProtectionLevel>>),
}

impl From<apl_ir::IrError> for RuntimeError {
    fn from(error: apl_ir::IrError) -> Self {
        Self::Message(format!("IR decode error: {error:?}"))
    }
}

pub fn run_program(program: &Program, input: Vec<String>) -> Result<RunOutput, RuntimeError> {
    let compiled = compile_program(program)?;
    run_compiled_program(&compiled, input)
}

pub fn compile_program(program: &Program) -> Result<CompiledProgram, RuntimeError> {
    let mut compiler = StatementCompiler::new();
    let mut functions = HashMap::new();
    let mut entry_statements = Vec::new();

    for statement in &program.statements {
        if let Statement::FunctionDecl(function) = statement {
            if functions
                .insert(function.name.clone(), compiler.compile_function(function)?)
                .is_some()
            {
                return Err(RuntimeError::Message(format!(
                    "duplicate function `{}`",
                    function.name
                )));
            }
        } else {
            entry_statements.push(statement.clone());
        }
    }

    let entry = compiler.compile_block(&entry_statements)?;
    Ok(CompiledProgram {
        entry,
        functions,
        blocks: compiler.blocks,
        code: compiler.code,
    })
}

struct StatementCompiler {
    blocks: Vec<CompiledBlock>,
    code: Vec<StatementOp>,
}

impl StatementCompiler {
    fn new() -> Self {
        Self {
            blocks: Vec::new(),
            code: Vec::new(),
        }
    }

    fn compile_function(
        &mut self,
        function: &FunctionDecl,
    ) -> Result<CompiledFunction, RuntimeError> {
        Ok(CompiledFunction {
            params: function.params.clone(),
            body: self.compile_block(&function.body)?,
        })
    }

    fn compile_block(&mut self, statements: &[Statement]) -> Result<BlockId, RuntimeError> {
        let mut ops = Vec::new();
        for statement in statements {
            ops.extend(self.compile_statement(statement, ops.len())?);
        }
        let start = self.code.len();
        for op in &mut ops {
            absolutize_statement_op_pcs(op, start);
        }
        let end = start + ops.len();
        let id = self.blocks.len();
        self.code.extend(ops);
        self.blocks.push(CompiledBlock { start, end });
        Ok(id)
    }

    fn compile_statement(
        &mut self,
        statement: &Statement,
        base_pc: usize,
    ) -> Result<Vec<StatementOp>, RuntimeError> {
        let op = match statement {
            Statement::VariableDecl(decl) => StatementOp::VariableDecl(CompiledVariableDecl {
                kind: decl.kind,
                value_type: decl.value_type,
                name: decl.name.clone(),
                initial_value: compile_expression(&decl.initial_value),
            }),
            Statement::ListDecl(decl) => StatementOp::ListDecl(CompiledListDecl {
                name: decl.name.clone(),
                elements: decl.elements.iter().map(compile_list_element).collect(),
            }),
            Statement::VTimeDecl(decl) => StatementOp::VTimeDecl(CompiledVTimeDecl {
                name: decl.name.clone(),
                initial_value: compile_expression(&decl.initial_value),
            }),
            Statement::FunctionDecl(_) => return Ok(Vec::new()),
            Statement::Assignment(assignment) => StatementOp::Assignment(CompiledAssignment {
                name: assignment.name.clone(),
                operator: assignment.operator,
                value: compile_expression(&assignment.value),
            }),
            Statement::InfoAssignment(assignment) => {
                StatementOp::InfoAssignment(assignment.clone())
            }
            Statement::AddToList { list_name, element } => StatementOp::AddToList {
                list_name: list_name.clone(),
                element: compile_list_element(element),
            },
            Statement::SecretUp(name) => StatementOp::SecretUp(name.clone()),
            Statement::Out(expression) => StatementOp::Out(compile_expression(expression)),
            Statement::Stop(reason) => StatementOp::Stop(reason.as_ref().map(compile_expression)),
            Statement::Fail(reason) => StatementOp::Fail(compile_expression(reason)),
            Statement::Return(expression) => StatementOp::Return(compile_expression(expression)),
            Statement::If(if_statement) => {
                return self.compile_if_statement(if_statement, base_pc);
            }
            Statement::While(while_statement) => {
                return Ok(vec![
                    StatementOp::LoopCheck {
                        condition: compile_expression(&while_statement.condition),
                        limit: while_statement.limit,
                        exit_pc: base_pc + 3,
                    },
                    StatementOp::ExecLoopBody {
                        body: self.compile_block(&while_statement.body)?,
                        continue_pc: base_pc,
                        break_pc: base_pc + 3,
                    },
                    StatementOp::Jump { target_pc: base_pc },
                ]);
            }
            Statement::Pick(pick_statement) => StatementOp::Pick {
                value: compile_expression(&pick_statement.value),
                item_name: pick_statement.item_name.clone(),
                body: self.compile_block(&pick_statement.body)?,
            },
            Statement::Break => StatementOp::Break,
            Statement::Continue => StatementOp::Continue,
        };
        Ok(vec![op])
    }

    fn compile_if_statement(
        &mut self,
        if_statement: &apl_core::IfStatement,
        base_pc: usize,
    ) -> Result<Vec<StatementOp>, RuntimeError> {
        let mut branches = Vec::new();
        branches.push((
            compile_expression(&if_statement.condition),
            self.compile_block(&if_statement.body)?,
        ));
        for branch in &if_statement.else_if_branches {
            branches.push((
                compile_expression(&branch.condition),
                self.compile_block(&branch.body)?,
            ));
        }
        let else_body = if_statement
            .else_body
            .as_deref()
            .map(|body| self.compile_block(body))
            .transpose()?;

        let branch_count = branches.len();
        let local_end = branch_count * 3 + usize::from(else_body.is_some());
        let mut ops = Vec::new();
        for (index, (condition, body)) in branches.into_iter().enumerate() {
            let next_branch = (index + 1) * 3;
            let false_target = if index + 1 == branch_count {
                if else_body.is_some() {
                    local_end - 1
                } else {
                    local_end
                }
            } else {
                next_branch
            };
            ops.push(StatementOp::JumpIfFalse {
                condition,
                target_pc: base_pc + false_target,
            });
            ops.push(StatementOp::ExecScopedBlock(body));
            ops.push(StatementOp::Jump {
                target_pc: base_pc + local_end,
            });
        }
        if let Some(body) = else_body {
            ops.push(StatementOp::ExecScopedBlock(body));
        }
        Ok(ops)
    }
}

fn absolutize_statement_op_pcs(op: &mut StatementOp, block_start: usize) {
    match op {
        StatementOp::Jump { target_pc }
        | StatementOp::JumpIfFalse { target_pc, .. }
        | StatementOp::LoopCheck {
            exit_pc: target_pc, ..
        } => {
            *target_pc += block_start;
        }
        StatementOp::ExecLoopBody {
            continue_pc,
            break_pc,
            ..
        } => {
            *continue_pc += block_start;
            *break_pc += block_start;
        }
        _ => {}
    }
}

fn compile_list_element(element: &apl_core::ListElement) -> CompiledListElement {
    CompiledListElement {
        value: compile_expression(&element.value),
        protection: element.protection,
    }
}

fn compile_expression(expression: &Expression) -> CompiledExpression {
    let mut ops = Vec::new();
    compile_expression_ops(expression, &mut ops);
    CompiledExpression {
        ops,
        form: expression_form(expression),
    }
}

fn compile_expression_ops(expression: &Expression, ops: &mut Vec<ExpressionOp>) {
    match expression {
        Expression::Literal(value) => ops.push(ExpressionOp::PushLiteral(value.clone())),
        Expression::ListLiteral(elements) => {
            let protections = elements
                .iter()
                .map(|element| {
                    compile_expression_ops(&element.value, ops);
                    element.protection
                })
                .collect();
            ops.push(ExpressionOp::BuildList(protections));
        }
        Expression::None => ops.push(ExpressionOp::PushNone),
        Expression::Input => ops.push(ExpressionOp::ReadInput),
        Expression::SecretInput => ops.push(ExpressionOp::ReadSecretInput),
        Expression::Variable(name) => ops.push(ExpressionOp::LoadVariable(name.clone())),
        Expression::Call { name, args } => {
            for arg in args {
                compile_expression_ops(arg, ops);
            }
            ops.push(ExpressionOp::Call {
                name: name.clone(),
                arg_count: args.len(),
                arg_forms: args.iter().map(expression_form).collect(),
            });
        }
        Expression::Index { target, index } => {
            compile_expression_ops(target, ops);
            compile_expression_ops(index, ops);
            ops.push(ExpressionOp::Index);
        }
        Expression::Slice {
            target,
            start,
            end,
            step,
        } => {
            compile_expression_ops(target, ops);
            if let Some(start) = start {
                compile_expression_ops(start, ops);
            }
            if let Some(end) = end {
                compile_expression_ops(end, ops);
            }
            if let Some(step) = step {
                compile_expression_ops(step, ops);
            }
            ops.push(ExpressionOp::Slice {
                has_start: start.is_some(),
                has_end: end.is_some(),
                has_step: step.is_some(),
            });
        }
        Expression::Unary {
            operator,
            expression,
        } => {
            compile_expression_ops(expression, ops);
            ops.push(ExpressionOp::Unary(*operator));
        }
        Expression::Binary {
            left,
            operator,
            right,
        } => {
            compile_expression_ops(left, ops);
            compile_expression_ops(right, ops);
            ops.push(ExpressionOp::Binary(*operator));
        }
        Expression::SelfCheck(name) => ops.push(ExpressionOp::SelfCheck(name.clone())),
        Expression::Grouped(inner) => compile_expression_ops(inner, ops),
    }
}

fn expression_form(expression: &Expression) -> ExpressionForm {
    match expression {
        Expression::Input => ExpressionForm::Input,
        Expression::SecretInput => ExpressionForm::SecretInput,
        Expression::Variable(name) => ExpressionForm::Variable(name.clone()),
        Expression::Grouped(inner) => expression_form(inner),
        _ => ExpressionForm::Other,
    }
}

pub fn compile_ir_bytes(bytes: &[u8]) -> Result<CompiledProgram, RuntimeError> {
    let program = apl_ir::decode_program(bytes)?;
    compile_program(&program)
}

pub fn run_compiled_program(
    program: &CompiledProgram,
    input: Vec<String>,
) -> Result<RunOutput, RuntimeError> {
    let mut runtime = Runtime::new(input);
    runtime.run(program)
}

pub fn run_ir_bytes(bytes: &[u8], input: Vec<String>) -> Result<RunOutput, RuntimeError> {
    let program = compile_ir_bytes(bytes)?;
    run_compiled_program(&program, input)
}

struct Runtime<'a> {
    program: Option<&'a CompiledProgram>,
    absolute: HashMap<String, AbsoluteSlot>,
    lists: HashMap<String, RuntimeValue>,
    scopes: Vec<HashMap<String, RuntimeValue>>,
    input: VecDeque<String>,
    stdout: String,
    stopped: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct AbsoluteSlot {
    kind: VariableKind,
    value_type: ValueType,
    initial: RuntimeValue,
    current: RuntimeValue,
}

#[derive(Debug, Clone, PartialEq)]
struct RuntimeValue {
    data: RuntimeData,
    protection: ProtectionLevel,
}

#[derive(Debug, Clone, PartialEq)]
enum RuntimeData {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Bytes(Vec<u8>),
    Json(String),
    List(Vec<RuntimeValue>),
    None,
}

enum Flow {
    None,
    Break,
    Continue,
    Return(RuntimeValue),
    Stop,
}

#[derive(Debug, Clone, PartialEq)]
struct BlockFrame {
    block_id: BlockId,
    pc: usize,
    loop_iterations: HashMap<usize, i64>,
}

impl<'a> Runtime<'a> {
    fn new(input: Vec<String>) -> Self {
        Self {
            program: None,
            absolute: HashMap::new(),
            lists: HashMap::new(),
            scopes: Vec::new(),
            input: input.into(),
            stdout: String::new(),
            stopped: false,
        }
    }

    fn run(&mut self, program: &'a CompiledProgram) -> Result<RunOutput, RuntimeError> {
        self.program = Some(program);

        match self.exec_block(program.entry)? {
            Flow::None | Flow::Stop => Ok(RunOutput {
                stdout: self.stdout.clone(),
            }),
            Flow::Break => Err(RuntimeError::Message("break outside loop".to_owned())),
            Flow::Continue => Err(RuntimeError::Message("continue outside loop".to_owned())),
            Flow::Return(_) => Err(RuntimeError::Message("return outside function".to_owned())),
        }
    }

    fn exec_block(&mut self, block_id: BlockId) -> Result<Flow, RuntimeError> {
        let block = self
            .program
            .and_then(|program| program.blocks.get(block_id))
            .ok_or_else(|| RuntimeError::Message(format!("unknown block `{block_id}`")))?;
        let mut frame = BlockFrame {
            block_id,
            pc: block.start,
            loop_iterations: HashMap::new(),
        };

        while let Some(op) = self.next_statement_op(&mut frame)? {
            let flow = self.exec_statement_op(&mut frame, &op)?;
            if self.stopped {
                return Ok(Flow::Stop);
            }
            if !matches!(flow, Flow::None) {
                return Ok(flow);
            }
        }
        Ok(Flow::None)
    }

    fn next_statement_op(
        &self,
        frame: &mut BlockFrame,
    ) -> Result<Option<StatementOp>, RuntimeError> {
        let block = self
            .program
            .and_then(|program| program.blocks.get(frame.block_id))
            .ok_or_else(|| RuntimeError::Message(format!("unknown block `{}`", frame.block_id)))?;
        if frame.pc >= block.end {
            return Ok(None);
        }
        let op = self
            .program
            .and_then(|program| program.code.get(frame.pc))
            .cloned()
            .ok_or_else(|| RuntimeError::Message(format!("unknown pc `{}`", frame.pc)))?;
        frame.pc += 1;
        Ok(Some(op))
    }

    fn exec_scoped_block(&mut self, block_id: BlockId) -> Result<Flow, RuntimeError> {
        self.with_scope(|runtime| runtime.exec_block(block_id))
    }

    fn exec_statement_op(
        &mut self,
        frame: &mut BlockFrame,
        op: &StatementOp,
    ) -> Result<Flow, RuntimeError> {
        match op {
            StatementOp::VariableDecl(decl) => {
                let protection = variable_kind_protection(decl.kind);
                let value = self
                    .eval_for_type(&decl.initial_value, decl.value_type, protection)?
                    .with_protection(protection);
                self.absolute.insert(
                    decl.name.clone(),
                    AbsoluteSlot {
                        kind: decl.kind,
                        value_type: decl.value_type,
                        initial: value.clone(),
                        current: value,
                    },
                );
                Ok(Flow::None)
            }
            StatementOp::ListDecl(decl) => {
                let values = decl
                    .elements
                    .iter()
                    .map(|element| self.eval_list_element(element))
                    .collect::<Result<Vec<_>, _>>()?;
                self.lists
                    .insert(decl.name.clone(), RuntimeValue::list(values));
                Ok(Flow::None)
            }
            StatementOp::VTimeDecl(decl) => {
                let value = self.eval_expression(&decl.initial_value)?;
                self.declare_vtime(decl.name.clone(), value);
                Ok(Flow::None)
            }
            StatementOp::Assignment(assignment) => {
                self.exec_assignment(assignment)?;
                Ok(Flow::None)
            }
            StatementOp::InfoAssignment(assignment) => {
                let Some(slot) = self.absolute.get(&assignment.source) else {
                    return Err(RuntimeError::Message(format!(
                        "unknown absolute variable `{}`",
                        assignment.source
                    )));
                };
                let type_name = slot.value_type.to_string();
                let protection_name = slot.kind.to_string();
                self.assign_existing(&assignment.type_target, RuntimeValue::str(type_name))?;
                self.assign_existing(
                    &assignment.protection_target,
                    RuntimeValue::str(protection_name),
                )?;
                Ok(Flow::None)
            }
            StatementOp::AddToList { list_name, element } => {
                let value = self.eval_list_element(element)?;
                self.add_to_list(list_name, value)?;
                Ok(Flow::None)
            }
            StatementOp::SecretUp(name) => {
                let Some(slot) = self.absolute.get_mut(name) else {
                    return Err(RuntimeError::Message(format!(
                        "unknown absolute variable `{name}`"
                    )));
                };
                slot.kind = match slot.kind {
                    VariableKind::Av => VariableKind::Asv,
                    VariableKind::Asv => VariableKind::Sasv,
                    VariableKind::Sasv => {
                        return Err(RuntimeError::Message(format!("`{name}` is already SASV")))
                    }
                };
                let protection = variable_kind_protection(slot.kind);
                slot.current.protection = protection;
                slot.initial.protection = protection;
                Ok(Flow::None)
            }
            StatementOp::Out(expression) => {
                let value = self.eval_expression(expression)?;
                if value.effective_protection() != ProtectionLevel::Av {
                    return Err(RuntimeError::Message(
                        "refusing to output secret value".to_owned(),
                    ));
                }
                self.stdout.push_str(&value.render());
                self.stdout.push('\n');
                Ok(Flow::None)
            }
            StatementOp::Stop(reason) => {
                if let Some(reason) = reason {
                    let value = self.eval_expression(reason)?;
                    if value.effective_protection() != ProtectionLevel::Av {
                        return Err(RuntimeError::Message(
                            "refusing to stop with secret reason".to_owned(),
                        ));
                    }
                    self.stdout.push_str(&value.render());
                    self.stdout.push('\n');
                }
                self.stopped = true;
                Ok(Flow::Stop)
            }
            StatementOp::Fail(reason) => {
                let value = self.eval_expression(reason)?;
                if value.effective_protection() != ProtectionLevel::Av {
                    return Err(RuntimeError::Message(
                        "refusing to fail with secret reason".to_owned(),
                    ));
                }
                Err(RuntimeError::Failed(value.render()))
            }
            StatementOp::Return(expression) => Ok(Flow::Return(self.eval_expression(expression)?)),
            StatementOp::Jump { target_pc } => {
                frame.pc = *target_pc;
                Ok(Flow::None)
            }
            StatementOp::JumpIfFalse {
                condition,
                target_pc,
            } => {
                if !self.eval_bool(condition)? {
                    frame.pc = *target_pc;
                }
                Ok(Flow::None)
            }
            StatementOp::ExecScopedBlock(block_id) => self.exec_scoped_block(*block_id),
            StatementOp::LoopCheck {
                condition,
                limit,
                exit_pc,
            } => {
                let op_pc = frame.pc.saturating_sub(1);
                if !self.eval_bool(condition)? {
                    frame.loop_iterations.remove(&op_pc);
                    frame.pc = *exit_pc;
                    return Ok(Flow::None);
                }
                let iterations = frame.loop_iterations.entry(op_pc).or_insert(0);
                if *limit != -1 && *iterations >= *limit {
                    frame.loop_iterations.remove(&op_pc);
                    frame.pc = *exit_pc;
                    return Ok(Flow::None);
                }
                *iterations += 1;
                Ok(Flow::None)
            }
            StatementOp::ExecLoopBody {
                body,
                continue_pc,
                break_pc,
            } => match self.exec_scoped_block(*body)? {
                Flow::None => Ok(Flow::None),
                Flow::Continue => {
                    frame.pc = *continue_pc;
                    Ok(Flow::None)
                }
                Flow::Break => {
                    frame.pc = *break_pc;
                    Ok(Flow::None)
                }
                other => Ok(other),
            },
            StatementOp::Pick {
                value,
                item_name,
                body,
            } => {
                let value = self.eval_expression(value)?;
                let items = self.pick_items(value)?;
                for item in items {
                    match self.with_scope(|runtime| {
                        runtime.declare_vtime(item_name.clone(), item);
                        runtime.exec_block(*body)
                    })? {
                        Flow::None | Flow::Continue => {}
                        Flow::Break => break,
                        other => return Ok(other),
                    }
                }
                Ok(Flow::None)
            }
            StatementOp::Break => Ok(Flow::Break),
            StatementOp::Continue => Ok(Flow::Continue),
        }
    }

    fn exec_assignment(&mut self, assignment: &CompiledAssignment) -> Result<(), RuntimeError> {
        if self.vtime_exists(&assignment.name) {
            let current = self.lookup(&assignment.name)?;
            let value = match assignment.operator {
                AssignmentOperator::Assign => self.eval_expression(&assignment.value)?,
                operator => {
                    let right = self.eval_expression(&assignment.value)?;
                    self.apply_arithmetic(current, right, operator)?
                }
            };
            self.update_vtime(&assignment.name, value);
            return Ok(());
        }

        let current = self.lookup_absolute(&assignment.name)?.current.clone();
        let value = match assignment.operator {
            AssignmentOperator::Assign => self.eval_expression(&assignment.value)?,
            operator => {
                let right = self.eval_expression(&assignment.value)?;
                self.apply_arithmetic(current, right, operator)?
            }
        };
        self.assign_existing(&assignment.name, value)
    }

    fn eval_expression(
        &mut self,
        expression: &CompiledExpression,
    ) -> Result<RuntimeValue, RuntimeError> {
        let mut stack = Vec::new();

        for op in &expression.ops {
            match op {
                ExpressionOp::PushLiteral(value) => {
                    stack.push(RuntimeValue::from_ast(value.clone()));
                }
                ExpressionOp::PushNone => stack.push(RuntimeValue::none(ProtectionLevel::Av)),
                ExpressionOp::ReadInput => stack.push(RuntimeValue::str(self.read_input())),
                ExpressionOp::ReadSecretInput => stack.push(RuntimeValue::new(
                    RuntimeData::Str(self.read_input()),
                    ProtectionLevel::Asv,
                )),
                ExpressionOp::LoadVariable(name) => stack.push(self.lookup(name)?),
                ExpressionOp::Call {
                    name,
                    arg_count,
                    arg_forms,
                } => {
                    let args = pop_stack_values(&mut stack, *arg_count)?;
                    stack.push(self.eval_call(name, args, arg_forms)?);
                }
                ExpressionOp::Index => {
                    let index = pop_stack_value(&mut stack)?;
                    let target = pop_stack_value(&mut stack)?;
                    stack.push(self.eval_index(target, index)?);
                }
                ExpressionOp::Slice {
                    has_start,
                    has_end,
                    has_step,
                } => {
                    let step = pop_optional_stack_int(&mut stack, *has_step)?;
                    let end = pop_optional_stack_int(&mut stack, *has_end)?;
                    let start = pop_optional_stack_int(&mut stack, *has_start)?;
                    let target = pop_stack_value(&mut stack)?;
                    stack.push(self.eval_slice(target, start, end, step)?);
                }
                ExpressionOp::Unary(operator) => {
                    let value = pop_stack_value(&mut stack)?;
                    stack.push(self.eval_unary(*operator, value)?);
                }
                ExpressionOp::Binary(operator) => {
                    let right = pop_stack_value(&mut stack)?;
                    let left = pop_stack_value(&mut stack)?;
                    stack.push(self.eval_binary(left, *operator, right)?);
                }
                ExpressionOp::SelfCheck(name) => {
                    let Some(slot) = self.absolute.get(name) else {
                        return Err(RuntimeError::Message(format!(
                            "unknown absolute variable `{name}`"
                        )));
                    };
                    stack.push(RuntimeValue::new(
                        RuntimeData::Bool(slot.current.data == slot.initial.data),
                        ProtectionLevel::Av,
                    ));
                }
                ExpressionOp::BuildList(protections) => {
                    let mut values = pop_stack_values(&mut stack, protections.len())?;
                    for (value, protection) in values.iter_mut().zip(protections) {
                        if let Some(protection) = protection {
                            if value.effective_protection() > *protection {
                                return Err(RuntimeError::Message(
                                    "refusing to downgrade list element protection".to_owned(),
                                ));
                            }
                            value.protection = *protection;
                        }
                    }
                    stack.push(RuntimeValue::list(values));
                }
            }
        }

        if stack.len() != 1 {
            return Err(RuntimeError::Message(
                "invalid compiled expression stack".to_owned(),
            ));
        }

        Ok(stack.pop().unwrap())
    }

    fn eval_for_type(
        &mut self,
        expression: &CompiledExpression,
        value_type: ValueType,
        protection: ProtectionLevel,
    ) -> Result<RuntimeValue, RuntimeError> {
        let value = self.eval_expression(expression)?;
        if matches!(value.data, RuntimeData::None) {
            return Ok(value.with_protection(protection));
        }
        if matches!(
            expression.form,
            ExpressionForm::Input | ExpressionForm::SecretInput
        ) {
            return Ok(value.convert(value_type).with_protection(protection));
        }
        if value.matches_type(value_type) {
            Ok(value.with_protection(protection))
        } else {
            Ok(RuntimeValue::none(protection))
        }
    }
    fn eval_call(
        &mut self,
        name: &str,
        args: Vec<RuntimeValue>,
        arg_forms: &[ExpressionForm],
    ) -> Result<RuntimeValue, RuntimeError> {
        if let Some(value_type) = conversion_type(name) {
            let value = args[0].clone();
            return Ok(value.convert(value_type));
        }

        match name {
            "get" => {
                let target = args[0].clone();
                let index = args[1].clone();
                self.eval_index(target, index)
            }
            "pop" => {
                let ExpressionForm::Variable(name) = &arg_forms[0] else {
                    return Err(RuntimeError::Message("pop expects list name".to_owned()));
                };
                self.pop_list(name)
            }
            "split" => {
                let target = args[0].clone();
                let delim = args[1].clone();
                let protection = target
                    .effective_protection()
                    .max(delim.effective_protection());
                let RuntimeData::Str(text) = &target.data else {
                    return Ok(RuntimeValue::none(protection));
                };
                let RuntimeData::Str(separator) = &delim.data else {
                    return Ok(RuntimeValue::none(protection));
                };
                if separator.is_empty() {
                    return Ok(RuntimeValue::none(protection));
                }
                let parts: Vec<RuntimeValue> = text
                    .split(separator)
                    .map(|s| RuntimeValue::str(s.to_owned()).with_protection(protection))
                    .collect();
                Ok(RuntimeValue::list(parts).with_protection(protection))
            }
            "len" => {
                let target = args[0].clone();
                let protection = target.effective_protection();
                let length = match &target.data {
                    RuntimeData::Str(s) => s.chars().count() as i64,
                    RuntimeData::List(values) => values.len() as i64,
                    RuntimeData::Bytes(b) => b.len() as i64,
                    _ => return Ok(RuntimeValue::none(protection)),
                };
                Ok(RuntimeValue::new(RuntimeData::Int(length), protection))
            }
            "contains" => {
                let target = args[0].clone();
                let sub = args[1].clone();
                let protection = target
                    .effective_protection()
                    .max(sub.effective_protection());
                let RuntimeData::Str(text) = &target.data else {
                    return Ok(RuntimeValue::none(protection));
                };
                let RuntimeData::Str(substr) = &sub.data else {
                    return Ok(RuntimeValue::none(protection));
                };
                Ok(RuntimeValue::new(
                    RuntimeData::Bool(text.contains(substr)),
                    protection,
                ))
            }
            "join" => {
                let values = args[0].clone();
                let separator = args[1].clone();
                let protection = values
                    .effective_protection()
                    .max(separator.effective_protection());
                let RuntimeData::List(items) = &values.data else {
                    return Ok(RuntimeValue::none(protection));
                };
                let RuntimeData::Str(separator) = &separator.data else {
                    return Ok(RuntimeValue::none(protection));
                };
                let mut parts = Vec::with_capacity(items.len());
                for item in items {
                    let RuntimeData::Str(value) = &item.data else {
                        return Ok(RuntimeValue::none(protection));
                    };
                    parts.push(value.as_str());
                }
                Ok(RuntimeValue::str(parts.join(separator)).with_protection(protection))
            }
            "ord" => {
                let value = args[0].clone();
                let protection = value.effective_protection();
                let RuntimeData::Str(text) = &value.data else {
                    return Ok(RuntimeValue::none(protection));
                };
                let Some(ch) = text.chars().next() else {
                    return Ok(RuntimeValue::none(protection));
                };
                Ok(RuntimeValue::new(RuntimeData::Int(ch as i64), protection))
            }
            "char" => {
                let value = args[0].clone();
                let protection = value.effective_protection();
                let RuntimeData::Int(code) = value.data else {
                    return Ok(RuntimeValue::none(protection));
                };
                let Ok(code) = u32::try_from(code) else {
                    return Ok(RuntimeValue::none(protection));
                };
                let Some(ch) = char::from_u32(code) else {
                    return Ok(RuntimeValue::none(protection));
                };
                Ok(RuntimeValue::str(ch.to_string()).with_protection(protection))
            }
            "pow" => {
                let base = args[0].clone();
                let exponent = args[1].clone();
                let protection = base
                    .effective_protection()
                    .max(exponent.effective_protection());
                let RuntimeData::Int(exponent) = exponent.data else {
                    return Ok(RuntimeValue::none(protection));
                };
                if exponent < 0 {
                    return Ok(RuntimeValue::none(protection));
                }
                match base.data {
                    RuntimeData::Int(base) => {
                        let Ok(exponent) = u32::try_from(exponent) else {
                            return Ok(RuntimeValue::none(protection));
                        };
                        let Some(value) = base.checked_pow(exponent) else {
                            return Ok(RuntimeValue::none(protection));
                        };
                        Ok(RuntimeValue::new(RuntimeData::Int(value), protection))
                    }
                    RuntimeData::Float(base) => {
                        let Ok(exponent) = i32::try_from(exponent) else {
                            return Ok(RuntimeValue::none(protection));
                        };
                        Ok(RuntimeValue::new(
                            RuntimeData::Float(base.powi(exponent)),
                            protection,
                        ))
                    }
                    _ => Ok(RuntimeValue::none(protection)),
                }
            }
            _ => {
                let Some(function) = self
                    .program
                    .and_then(|program| program.functions.get(name))
                    .cloned()
                else {
                    return Err(RuntimeError::Message(format!("unknown function `{name}`")));
                };
                self.with_scope(|runtime| {
                    for (param, value) in function.params.iter().zip(args) {
                        runtime.declare_vtime(param.clone(), value);
                    }
                    match runtime.exec_block(function.body)? {
                        Flow::Return(value) => Ok(value),
                        Flow::None | Flow::Stop => Ok(RuntimeValue::none(ProtectionLevel::Av)),
                        Flow::Break => {
                            Err(RuntimeError::Message("break escaped function".to_owned()))
                        }
                        Flow::Continue => Err(RuntimeError::Message(
                            "continue escaped function".to_owned(),
                        )),
                    }
                })
            }
        }
    }

    fn eval_index(
        &mut self,
        target: RuntimeValue,
        index: RuntimeValue,
    ) -> Result<RuntimeValue, RuntimeError> {
        let RuntimeData::Int(index) = index.data else {
            return Ok(RuntimeValue::none(target.effective_protection()));
        };
        let protection = target.effective_protection();
        match target.data {
            RuntimeData::List(values) => Ok(resolve_index(&values, index)
                .cloned()
                .unwrap_or_else(|| RuntimeValue::none(protection))),
            RuntimeData::Str(value) => Ok(value
                .chars()
                .collect::<Vec<_>>()
                .get(resolve_raw_index(value.chars().count(), index).unwrap_or(usize::MAX))
                .map(|ch| RuntimeValue::str(ch.to_string()).with_protection(protection))
                .unwrap_or_else(|| RuntimeValue::none(protection))),
            RuntimeData::Bytes(value) => Ok(value
                .get(resolve_raw_index(value.len(), index).unwrap_or(usize::MAX))
                .map(|b| RuntimeValue::new(RuntimeData::Int(*b as i64), protection))
                .unwrap_or_else(|| RuntimeValue::none(protection))),
            _ => Ok(RuntimeValue::none(protection)),
        }
    }

    fn eval_slice(
        &self,
        target: RuntimeValue,
        start: Option<i64>,
        end: Option<i64>,
        step: Option<i64>,
    ) -> Result<RuntimeValue, RuntimeError> {
        let protection = target.effective_protection();
        let step = step.unwrap_or(1);
        if step == 0 {
            return Ok(RuntimeValue::none(protection));
        }
        match target.data {
            RuntimeData::Str(value) => {
                let chars: Vec<char> = value.chars().collect();
                let result: String = slice_indices(chars.len(), start, end, step)
                    .into_iter()
                    .map(|index| chars[index])
                    .collect();
                Ok(RuntimeValue::str(result).with_protection(protection))
            }
            RuntimeData::List(values) => {
                let result: Vec<RuntimeValue> = slice_indices(values.len(), start, end, step)
                    .into_iter()
                    .map(|index| values[index].clone())
                    .collect();
                Ok(RuntimeValue::list(result).with_protection(protection))
            }
            RuntimeData::Bytes(value) => {
                let result: Vec<u8> = slice_indices(value.len(), start, end, step)
                    .into_iter()
                    .map(|index| value[index])
                    .collect();
                Ok(RuntimeValue::new(RuntimeData::Bytes(result), protection))
            }
            _ => Ok(RuntimeValue::none(protection)),
        }
    }
    fn eval_list_element(
        &mut self,
        element: &CompiledListElement,
    ) -> Result<RuntimeValue, RuntimeError> {
        let value = self.eval_expression(&element.value)?;
        if let Some(protection) = element.protection {
            if value.effective_protection() > protection {
                return Err(RuntimeError::Message(
                    "refusing to downgrade list element protection".to_owned(),
                ));
            }
            Ok(value.with_protection(protection))
        } else {
            Ok(value)
        }
    }
    fn eval_unary(
        &self,
        operator: UnaryOperator,
        value: RuntimeValue,
    ) -> Result<RuntimeValue, RuntimeError> {
        let protection = value.protection;
        match (operator, value.data) {
            (_, RuntimeData::None) => Ok(RuntimeValue::none(protection)),
            (UnaryOperator::Negate, RuntimeData::Int(val)) => {
                Ok(RuntimeValue::new(RuntimeData::Int(-val), protection))
            }
            (UnaryOperator::Negate, RuntimeData::Float(val)) => {
                Ok(RuntimeValue::new(RuntimeData::Float(-val), protection))
            }
            (UnaryOperator::Not, RuntimeData::Bool(val)) => {
                Ok(RuntimeValue::new(RuntimeData::Bool(!val), protection))
            }
            _ => Err(RuntimeError::Message("invalid unary operands".to_owned())),
        }
    }

    fn eval_binary(
        &self,
        left: RuntimeValue,
        operator: BinaryOperator,
        right: RuntimeValue,
    ) -> Result<RuntimeValue, RuntimeError> {
        let protection = left.protection.max(right.protection);
        match operator {
            BinaryOperator::Add => match (left.data, right.data) {
                (RuntimeData::Int(a), RuntimeData::Int(b)) => {
                    Ok(RuntimeValue::new(RuntimeData::Int(a + b), protection))
                }
                (RuntimeData::Float(a), RuntimeData::Float(b)) => {
                    Ok(RuntimeValue::new(RuntimeData::Float(a + b), protection))
                }
                (RuntimeData::Str(a), RuntimeData::Str(b)) => Ok(RuntimeValue::new(
                    RuntimeData::Str(format!("{a}{b}")),
                    protection,
                )),
                (RuntimeData::Int(a), RuntimeData::Float(b)) => Ok(RuntimeValue::new(
                    RuntimeData::Float(a as f64 + b),
                    protection,
                )),
                (RuntimeData::Float(a), RuntimeData::Int(b)) => Ok(RuntimeValue::new(
                    RuntimeData::Float(a + b as f64),
                    protection,
                )),
                _ => Err(RuntimeError::Message("invalid add operands".to_owned())),
            },
            BinaryOperator::Sub => match (left.data, right.data) {
                (RuntimeData::Int(a), RuntimeData::Int(b)) => {
                    Ok(RuntimeValue::new(RuntimeData::Int(a - b), protection))
                }
                (RuntimeData::Float(a), RuntimeData::Float(b)) => {
                    Ok(RuntimeValue::new(RuntimeData::Float(a - b), protection))
                }
                (RuntimeData::Int(a), RuntimeData::Float(b)) => Ok(RuntimeValue::new(
                    RuntimeData::Float(a as f64 - b),
                    protection,
                )),
                (RuntimeData::Float(a), RuntimeData::Int(b)) => Ok(RuntimeValue::new(
                    RuntimeData::Float(a - b as f64),
                    protection,
                )),
                _ => Err(RuntimeError::Message("invalid sub operands".to_owned())),
            },
            BinaryOperator::Mul => match (left.data, right.data) {
                (RuntimeData::Int(a), RuntimeData::Int(b)) => {
                    Ok(RuntimeValue::new(RuntimeData::Int(a * b), protection))
                }
                (RuntimeData::Float(a), RuntimeData::Float(b)) => {
                    Ok(RuntimeValue::new(RuntimeData::Float(a * b), protection))
                }
                (RuntimeData::Int(a), RuntimeData::Float(b)) => Ok(RuntimeValue::new(
                    RuntimeData::Float(a as f64 * b),
                    protection,
                )),
                (RuntimeData::Float(a), RuntimeData::Int(b)) => Ok(RuntimeValue::new(
                    RuntimeData::Float(a * b as f64),
                    protection,
                )),
                _ => Err(RuntimeError::Message("invalid mul operands".to_owned())),
            },
            BinaryOperator::Div => match (left.data, right.data) {
                (RuntimeData::Int(a), RuntimeData::Int(b)) => {
                    if b == 0 {
                        return Err(RuntimeError::Message("division by zero".to_owned()));
                    }
                    Ok(RuntimeValue::new(RuntimeData::Int(a / b), protection))
                }
                (RuntimeData::Float(a), RuntimeData::Float(b)) => {
                    Ok(RuntimeValue::new(RuntimeData::Float(a / b), protection))
                }
                (RuntimeData::Int(a), RuntimeData::Float(b)) => Ok(RuntimeValue::new(
                    RuntimeData::Float(a as f64 / b),
                    protection,
                )),
                (RuntimeData::Float(a), RuntimeData::Int(b)) => Ok(RuntimeValue::new(
                    RuntimeData::Float(a / b as f64),
                    protection,
                )),
                _ => Err(RuntimeError::Message("invalid div operands".to_owned())),
            },
            BinaryOperator::Equal => Ok(RuntimeValue::new(
                RuntimeData::Bool(left.data == right.data),
                protection,
            )),
            BinaryOperator::NotEqual => Ok(RuntimeValue::new(
                RuntimeData::Bool(left.data != right.data),
                protection,
            )),
            BinaryOperator::Greater
            | BinaryOperator::GreaterEqual
            | BinaryOperator::Less
            | BinaryOperator::LessEqual => {
                self.eval_ordering(left.data, operator, right.data, protection)
            }
            BinaryOperator::And => match (left.data, right.data) {
                (RuntimeData::Bool(a), RuntimeData::Bool(b)) => {
                    Ok(RuntimeValue::new(RuntimeData::Bool(a && b), protection))
                }
                _ => Err(RuntimeError::Message("invalid and operands".to_owned())),
            },
            BinaryOperator::Or => match (left.data, right.data) {
                (RuntimeData::Bool(a), RuntimeData::Bool(b)) => {
                    Ok(RuntimeValue::new(RuntimeData::Bool(a || b), protection))
                }
                _ => Err(RuntimeError::Message("invalid or operands".to_owned())),
            },
        }
    }

    fn eval_ordering(
        &self,
        left: RuntimeData,
        operator: BinaryOperator,
        right: RuntimeData,
        protection: ProtectionLevel,
    ) -> Result<RuntimeValue, RuntimeError> {
        let result = match (left, right) {
            (RuntimeData::Int(a), RuntimeData::Int(b)) => compare_values(a, b, operator),
            (RuntimeData::Float(a), RuntimeData::Float(b)) => compare_values(a, b, operator),
            _ => {
                return Err(RuntimeError::Message(
                    "invalid ordering operands".to_owned(),
                ));
            }
        };
        Ok(RuntimeValue::new(RuntimeData::Bool(result), protection))
    }

    fn eval_bool(&mut self, expression: &CompiledExpression) -> Result<bool, RuntimeError> {
        match self.eval_expression(expression)?.data {
            RuntimeData::Bool(value) => Ok(value),
            RuntimeData::None => Ok(false),
            _ => Err(RuntimeError::Message("condition is not Bool".to_owned())),
        }
    }

    fn apply_arithmetic(
        &self,
        current: RuntimeValue,
        right: RuntimeValue,
        operator: AssignmentOperator,
    ) -> Result<RuntimeValue, RuntimeError> {
        let binary = match operator {
            AssignmentOperator::Assign => unreachable!(),
            AssignmentOperator::AddAssign => BinaryOperator::Add,
            AssignmentOperator::SubAssign => BinaryOperator::Sub,
            AssignmentOperator::MulAssign => BinaryOperator::Mul,
            AssignmentOperator::DivAssign => BinaryOperator::Div,
        };
        self.eval_binary(current, binary, right)
    }
    fn pop_list(&mut self, name: &str) -> Result<RuntimeValue, RuntimeError> {
        if let Some(list) = self.vtime_mut(name) {
            return pop_list_value(list);
        }
        let Some(list) = self.lists.get_mut(name) else {
            return Ok(RuntimeValue::none(ProtectionLevel::Av));
        };
        pop_list_value(list)
    }

    fn add_to_list(&mut self, name: &str, value: RuntimeValue) -> Result<(), RuntimeError> {
        if let Some(list) = self.vtime_mut(name) {
            return push_list_value(name, list, value);
        }
        let Some(list) = self.lists.get_mut(name) else {
            return Err(RuntimeError::Message(format!("unknown list `{name}`")));
        };
        push_list_value(name, list, value)
    }

    fn vtime_mut(&mut self, name: &str) -> Option<&mut RuntimeValue> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(value) = scope.get_mut(name) {
                return Some(value);
            }
        }
        None
    }

    fn pick_items(&self, value: RuntimeValue) -> Result<Vec<RuntimeValue>, RuntimeError> {
        let protection = value.effective_protection();
        match value.data {
            RuntimeData::List(values) => Ok(values),
            RuntimeData::Str(value) => Ok(value
                .chars()
                .map(|ch| RuntimeValue::str(ch.to_string()).with_protection(protection))
                .collect()),
            RuntimeData::Bytes(value) => Ok(value
                .into_iter()
                .map(|byte| RuntimeValue::new(RuntimeData::Int(i64::from(byte)), protection))
                .collect()),
            _ => Err(RuntimeError::Message("value is not pickable".to_owned())),
        }
    }

    fn assign_existing(&mut self, name: &str, value: RuntimeValue) -> Result<(), RuntimeError> {
        if self.vtime_exists(name) {
            self.update_vtime(name, value);
            return Ok(());
        }
        let Some(slot) = self.absolute.get_mut(name) else {
            return Err(RuntimeError::Message(format!("unknown variable `{name}`")));
        };
        let target_protection = variable_kind_protection(slot.kind);
        if value.effective_protection() > target_protection {
            return Err(RuntimeError::Message(format!(
                "refusing to downgrade protection of `{name}`"
            )));
        }
        let value =
            if value.matches_type(slot.value_type) || matches!(value.data, RuntimeData::None) {
                value.with_protection(target_protection)
            } else {
                RuntimeValue::none(target_protection)
            };
        slot.current = value;
        Ok(())
    }

    fn lookup(&self, name: &str) -> Result<RuntimeValue, RuntimeError> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value.clone());
            }
        }
        if let Some(slot) = self.absolute.get(name) {
            return Ok(slot.current.clone());
        }
        if let Some(value) = self.lists.get(name) {
            return Ok(value.clone());
        }
        Err(RuntimeError::Message(format!("unknown variable `{name}`")))
    }

    fn lookup_absolute(&self, name: &str) -> Result<&AbsoluteSlot, RuntimeError> {
        self.absolute
            .get(name)
            .ok_or_else(|| RuntimeError::Message(format!("unknown absolute variable `{name}`")))
    }

    fn with_scope<T>(
        &mut self,
        body: impl FnOnce(&mut Self) -> Result<T, RuntimeError>,
    ) -> Result<T, RuntimeError> {
        self.scopes.push(HashMap::new());
        let result = body(self);
        self.scopes.pop();
        result
    }

    fn declare_vtime(&mut self, name: String, value: RuntimeValue) {
        if self.scopes.is_empty() {
            self.scopes.push(HashMap::new());
        }
        self.scopes.last_mut().unwrap().insert(name, value);
    }

    fn update_vtime(&mut self, name: &str, value: RuntimeValue) {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_owned(), value);
                return;
            }
        }
    }

    fn vtime_exists(&self, name: &str) -> bool {
        self.scopes
            .iter()
            .rev()
            .any(|scope| scope.contains_key(name))
    }

    fn read_input(&mut self) -> String {
        self.input.pop_front().unwrap_or_default()
    }
}

fn pop_stack_value(stack: &mut Vec<RuntimeValue>) -> Result<RuntimeValue, RuntimeError> {
    stack
        .pop()
        .ok_or_else(|| RuntimeError::Message("compiled expression stack underflow".to_owned()))
}

fn pop_stack_values(
    stack: &mut Vec<RuntimeValue>,
    count: usize,
) -> Result<Vec<RuntimeValue>, RuntimeError> {
    if stack.len() < count {
        return Err(RuntimeError::Message(
            "compiled expression stack underflow".to_owned(),
        ));
    }
    let start = stack.len() - count;
    Ok(stack.split_off(start))
}

fn pop_optional_stack_int(
    stack: &mut Vec<RuntimeValue>,
    has_value: bool,
) -> Result<Option<i64>, RuntimeError> {
    if !has_value {
        return Ok(None);
    }
    match pop_stack_value(stack)?.data {
        RuntimeData::Int(value) => Ok(Some(value)),
        _ => Ok(None),
    }
}

fn push_list_value(
    name: &str,
    list: &mut RuntimeValue,
    value: RuntimeValue,
) -> Result<(), RuntimeError> {
    let RuntimeData::List(values) = &mut list.data else {
        return Err(RuntimeError::Message(format!("`{name}` is not a list")));
    };
    values.push(value);
    list.refresh_list_protection();
    Ok(())
}

fn pop_list_value(list: &mut RuntimeValue) -> Result<RuntimeValue, RuntimeError> {
    let protection = list.effective_protection();
    let RuntimeData::List(values) = &mut list.data else {
        return Ok(RuntimeValue::none(protection));
    };
    let popped = values
        .pop()
        .unwrap_or_else(|| RuntimeValue::none(protection));
    list.refresh_list_protection();
    Ok(popped)
}

impl RuntimeValue {
    fn new(data: RuntimeData, protection: ProtectionLevel) -> Self {
        Self { data, protection }
    }

    fn none(protection: ProtectionLevel) -> Self {
        Self::new(RuntimeData::None, protection)
    }

    fn list(values: Vec<RuntimeValue>) -> Self {
        let protection = values
            .iter()
            .map(RuntimeValue::effective_protection)
            .max()
            .unwrap_or(ProtectionLevel::Av);
        Self::new(RuntimeData::List(values), protection)
    }

    fn str(value: String) -> Self {
        Self::new(RuntimeData::Str(value), ProtectionLevel::Av)
    }

    fn from_ast(value: AstValue) -> Self {
        let data = match value {
            AstValue::Int(value) => RuntimeData::Int(value),
            AstValue::Float(value) => RuntimeData::Float(value),
            AstValue::Bool(value) => RuntimeData::Bool(value),
            AstValue::Str(value) => RuntimeData::Str(value),
            AstValue::Bytes(value) => RuntimeData::Bytes(value),
            AstValue::Json(value) => RuntimeData::Json(value),
        };
        Self::new(data, ProtectionLevel::Av)
    }

    fn with_protection(mut self, protection: ProtectionLevel) -> Self {
        self.protection = protection;
        self
    }

    fn effective_protection(&self) -> ProtectionLevel {
        match &self.data {
            RuntimeData::List(values) => values
                .iter()
                .map(RuntimeValue::effective_protection)
                .fold(self.protection, ProtectionLevel::max),
            _ => self.protection,
        }
    }

    fn refresh_list_protection(&mut self) {
        if let RuntimeData::List(values) = &self.data {
            self.protection = values
                .iter()
                .map(RuntimeValue::effective_protection)
                .max()
                .unwrap_or(ProtectionLevel::Av);
        }
    }

    fn matches_type(&self, value_type: ValueType) -> bool {
        matches!(
            (&self.data, value_type),
            (RuntimeData::Int(_), ValueType::Int)
                | (RuntimeData::Float(_), ValueType::Float)
                | (RuntimeData::Bool(_), ValueType::Bool)
                | (RuntimeData::Str(_), ValueType::Str)
                | (RuntimeData::Bytes(_), ValueType::Bytes)
                | (RuntimeData::Json(_), ValueType::Json)
                | (RuntimeData::List(_), ValueType::List)
        )
    }

    fn convert(self, value_type: ValueType) -> Self {
        let protection = self.protection;
        if matches!(self.data, RuntimeData::None) {
            return Self::none(protection);
        }
        let converted = match value_type {
            ValueType::Int => self.render().parse::<i64>().ok().map(RuntimeData::Int),
            ValueType::Float => self.render().parse::<f64>().ok().map(RuntimeData::Float),
            ValueType::Bool => match self.render().as_str() {
                "true" => Some(RuntimeData::Bool(true)),
                "false" => Some(RuntimeData::Bool(false)),
                _ => None,
            },
            ValueType::Str => Some(RuntimeData::Str(self.render())),
            ValueType::Bytes => Some(RuntimeData::Bytes(self.render().into_bytes())),
            ValueType::Json => Some(RuntimeData::Json(self.render())),
            ValueType::List => match self.data {
                RuntimeData::List(values) => Some(RuntimeData::List(values)),
                _ => None,
            },
        };
        converted
            .map(|data| Self::new(data, protection))
            .unwrap_or_else(|| Self::none(protection))
    }

    fn render(&self) -> String {
        match &self.data {
            RuntimeData::Int(value) => value.to_string(),
            RuntimeData::Float(value) => value.to_string(),
            RuntimeData::Bool(value) => value.to_string(),
            RuntimeData::Str(value) => value.clone(),
            RuntimeData::Bytes(value) => format!("{value:?}"),
            RuntimeData::Json(value) => value.clone(),
            RuntimeData::List(values) => {
                let inner = values
                    .iter()
                    .map(RuntimeValue::render)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{inner}]")
            }
            RuntimeData::None => "NONE".to_owned(),
        }
    }
}

fn variable_kind_protection(kind: VariableKind) -> ProtectionLevel {
    match kind {
        VariableKind::Av => ProtectionLevel::Av,
        VariableKind::Asv => ProtectionLevel::Asv,
        VariableKind::Sasv => ProtectionLevel::Sasv,
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

fn compare_values<T: PartialOrd>(a: T, b: T, operator: BinaryOperator) -> bool {
    match operator {
        BinaryOperator::Greater => a > b,
        BinaryOperator::GreaterEqual => a >= b,
        BinaryOperator::Less => a < b,
        BinaryOperator::LessEqual => a <= b,
        _ => unreachable!(),
    }
}

fn resolve_index<T>(values: &[T], index: i64) -> Option<&T> {
    let index = resolve_raw_index(values.len(), index)?;
    values.get(index)
}

fn resolve_raw_index(len: usize, index: i64) -> Option<usize> {
    if index < 0 {
        len.checked_sub(index.unsigned_abs() as usize)
    } else {
        Some(index as usize)
    }
}

fn slice_indices(len: usize, start: Option<i64>, end: Option<i64>, step: i64) -> Vec<usize> {
    debug_assert_ne!(step, 0);

    if step > 0 {
        let mut index = normalize_forward_slice_bound(len, start.unwrap_or(0));
        let end = normalize_forward_slice_bound(len, end.unwrap_or(len as i64));
        let mut indices = Vec::new();

        while index < end {
            indices.push(index as usize);
            index += step as isize;
        }

        return indices;
    }

    let mut index = normalize_reverse_slice_start(len, start);
    let end = normalize_reverse_slice_end(len, end);
    let mut indices = Vec::new();

    while index > end {
        if (0..len as isize).contains(&index) {
            indices.push(index as usize);
        }
        index += step as isize;
    }

    indices
}

fn normalize_forward_slice_bound(len: usize, bound: i64) -> isize {
    let len = len as isize;
    let index = if bound < 0 {
        len + bound as isize
    } else {
        bound as isize
    };
    index.clamp(0, len)
}

fn normalize_reverse_slice_start(len: usize, start: Option<i64>) -> isize {
    let len = len as isize;
    let Some(start) = start else {
        return len - 1;
    };
    let index = if start < 0 {
        len + start as isize
    } else {
        start as isize
    };
    index.clamp(-1, len - 1)
}

fn normalize_reverse_slice_end(len: usize, end: Option<i64>) -> isize {
    let len = len as isize;
    let Some(end) = end else {
        return -1;
    };
    let index = if end < 0 {
        len + end as isize
    } else {
        end as isize
    };
    index.clamp(-1, len - 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use apl_core::{
        validate_program, Assignment, BinaryOperator, Expression, ListDecl, ListElement, Statement,
        Value, VariableDecl, WhileStatement,
    };

    fn run(source: Vec<Statement>) -> Result<String, RuntimeError> {
        run_program(&Program::new(source), vec![]).map(|output| output.stdout)
    }

    fn run_source(source: &str) -> Result<String, RuntimeError> {
        let program = apl_parser::parse_program(source).unwrap();
        validate_program(&program).unwrap();
        run_program(&program, vec![]).map(|output| output.stdout)
    }

    fn run_source_with_prelude(source: &str) -> Result<String, RuntimeError> {
        let prelude = concat!(
            include_str!("../../../std/runtime.apl"),
            "\n\n",
            include_str!("../../../std/lexer.apl"),
            "\n\n",
            include_str!("../../../std/parser.apl"),
            "\n\n",
            include_str!("../../../std/checker.apl"),
            "\n\n",
            include_str!("../../../std/ir.apl"),
            "\n\n",
            include_str!("../../../std/vm.apl"),
            "\n\n",
            include_str!("../../../std/bootstrap.apl")
        );
        let source = format!("{}\n\n{}\n", prelude.trim(), source.trim());
        let program = apl_parser::parse_program(&source).unwrap();
        validate_program(&program).unwrap();
        run_program(&program, vec![]).map(|output| output.stdout)
    }

    fn checked_program(source: &str) -> Program {
        let program = apl_parser::parse_program(source).unwrap();
        validate_program(&program).unwrap();
        program
    }

    fn block_ops(program: &CompiledProgram, block_id: BlockId) -> &[StatementOp] {
        let block = &program.blocks[block_id];
        &program.code[block.start..block.end]
    }

    #[test]
    fn runs_arithmetic_and_out() {
        let output = run(vec![
            Statement::VariableDecl(VariableDecl {
                kind: VariableKind::Av,
                value_type: ValueType::Int,
                name: "x".to_owned(),
                initial_value: Expression::Literal(Value::Int(1)),
            }),
            Statement::Assignment(Assignment {
                name: "x".to_owned(),
                operator: AssignmentOperator::AddAssign,
                value: Expression::Literal(Value::Int(2)),
            }),
            Statement::Out(Expression::Variable("x".to_owned())),
        ])
        .unwrap();

        assert_eq!(output, "3\n");
    }

    #[test]
    fn source_runtime_uses_pow_builtin() {
        let output = run_source(
            r#"
            AVInt x = pow(2, 3)
            AVFloat y = pow(2.0, 3)
            AVInt bad = pow(2, -1)
            ASVInt hidden = 2
            ASVInt hidden_pow = pow(hidden, 2)
            out x
            out y == 8.0
            out bad
            "#,
        )
        .unwrap();

        assert_eq!(output, "8\ntrue\nNONE\n");
    }

    #[test]
    fn runs_while_with_limit() {
        let output = run(vec![
            Statement::VariableDecl(VariableDecl {
                kind: VariableKind::Av,
                value_type: ValueType::Int,
                name: "x".to_owned(),
                initial_value: Expression::Literal(Value::Int(0)),
            }),
            Statement::While(WhileStatement {
                condition: Expression::Binary {
                    left: Box::new(Expression::Variable("x".to_owned())),
                    operator: BinaryOperator::Less,
                    right: Box::new(Expression::Literal(Value::Int(10))),
                },
                limit: 3,
                body: vec![Statement::Assignment(Assignment {
                    name: "x".to_owned(),
                    operator: AssignmentOperator::AddAssign,
                    value: Expression::Literal(Value::Int(1)),
                })],
            }),
            Statement::Out(Expression::Variable("x".to_owned())),
        ])
        .unwrap();

        assert_eq!(output, "3\n");
    }

    #[test]
    fn runs_nested_list_indexing() {
        let output = run(vec![
            Statement::ListDecl(ListDecl {
                name: "matrix".to_owned(),
                elements: vec![ListElement {
                    value: Expression::ListLiteral(vec![
                        ListElement {
                            value: Expression::Literal(Value::Int(1)),
                            protection: None,
                        },
                        ListElement {
                            value: Expression::Literal(Value::Int(2)),
                            protection: None,
                        },
                    ]),
                    protection: None,
                }],
            }),
            Statement::VTimeDecl(apl_core::VTimeDecl {
                name: "cell".to_owned(),
                initial_value: Expression::Index {
                    target: Box::new(Expression::Index {
                        target: Box::new(Expression::Variable("matrix".to_owned())),
                        index: Box::new(Expression::Literal(Value::Int(0))),
                    }),
                    index: Box::new(Expression::Literal(Value::Int(1))),
                },
            }),
            Statement::Out(Expression::Variable("cell".to_owned())),
        ])
        .unwrap();

        assert_eq!(output, "2\n");
    }

    #[test]
    fn source_runtime_handles_functions_lists_loops_and_slices() {
        let output = run_source(
            r#"
            List nums = [1, 2, 3, 4]
            AVInt total = 0

            func inc(value) {
              return value + 1
            }

            pick(nums[:3]): item {
              VTime next = inc(item)
              total += next
            }

            out total
            "#,
        )
        .unwrap();

        assert_eq!(output, "9\n");
    }

    #[test]
    fn source_runtime_handles_negative_step_slices() {
        let output = run_source(
            r#"
            List nums = [1, 2, 3]
            VTime reversed_nums = nums[::-1]
            AVStr text = "APL"
            VTime reversed_text = text[::-1]
            List empty = []
            VTime reversed_empty = empty[::-1]

            out get(reversed_nums, 0)
            out get(reversed_nums, 2)
            out reversed_text
            out len(reversed_empty)
            "#,
        )
        .unwrap();

        assert_eq!(output, "3\n1\nLPA\n0\n");
    }

    #[test]
    fn source_runtime_uses_apl_lexer_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = join(["AVInt x += 42 out ", char(34), "ok", char(34), " # ignored"], "")
            VTime tokens = lexer.tokenize(source)
            VTime first = get(tokens, 0)
            VTime op = get(tokens, 2)
            VTime string = get(tokens, 5)
            out len(tokens)
            out lexer.token_value(first)
            out lexer.token_kind(first)
            out lexer.token_value(op)
            out lexer.token_kind(op)
            out lexer.token_value(string)
            out lexer.token_kind(string)
            "#,
        )
        .unwrap();

        assert_eq!(output, "6\nAVInt\nKeyword\n+=\nSymbol\nok\nStr\n");
    }

    #[test]
    fn source_runtime_supports_vtime_list_add_and_join() {
        let output = run_source(
            r#"
            VTime chars = []
            add(chars, "A")
            add(chars, "P")
            add(chars, "L")
            out join(chars, "")
            out ord("A")
            out char(65)
            "#,
        )
        .unwrap();

        assert_eq!(output, "APL\n65\nA\n");
    }

    #[test]
    fn source_runtime_uses_apl_parser_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = "AVInt x = 42 out x x += 1"
            VTime statements = parser.parse_source(source)
            VTime decl = get(statements, 0)
            VTime out_node = get(statements, 1)
            VTime assign = get(statements, 2)
            VTime literal = get(decl, 3)

            out len(statements)
            out parser.node_kind(decl)
            out get(decl, 1)
            out get(decl, 2)
            out parser.expr_kind(literal)
            out parser.expr_value(literal)
            out parser.node_kind(out_node)
            out parser.node_kind(assign)
            out get(assign, 2)
            "#,
        )
        .unwrap();

        assert_eq!(output, "3\nDecl\nAVInt\nx\nInt\n42\nOut\nAssign\n+=\n");
    }

    #[test]
    fn source_runtime_uses_apl_ir_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = "AVInt x = 42 out x x += 1"
            VTime program = ir.compile_source(source)
            VTime decl = get(program, 0)
            VTime out_instruction = get(program, 1)
            VTime assign = get(program, 2)
            VTime literal = get(decl, 3)

            out len(program)
            out ir.opcode(decl)
            out get(decl, 1)
            out get(decl, 2)
            out ir.expr_opcode(literal)
            out ir.expr_value(literal)
            out ir.opcode(out_instruction)
            out ir.opcode(assign)
            out get(assign, 2)
            "#,
        )
        .unwrap();

        assert_eq!(output, "3\nDECL\nAVInt\nx\nLITERAL\n42\nOUT\nASSIGN\n+=\n");
    }

    #[test]
    fn source_runtime_uses_apl_vm_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = "AVInt x = 41 x += 1 out x"
            VTime vm_output = vm.run_source(source)

            out len(vm_output)
            out get(vm_output, 0)
            "#,
        )
        .unwrap();

        assert_eq!(output, "1\n42\n");
    }

    #[test]
    fn source_runtime_uses_apl_vm_if_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = "AVInt x = 1 if x == 1 { x += 41 out x } if x != 42 { out 0 }"
            VTime vm_output = vm.run_source(source)

            out len(vm_output)
            out get(vm_output, 0)
            "#,
        )
        .unwrap();

        assert_eq!(output, "1\n42\n");
    }

    #[test]
    fn source_runtime_uses_apl_vm_else_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = "AVInt x = 1 if x == 2 { out 0 } else { x += 41 out x }"
            VTime vm_output = vm.run_source(source)

            out len(vm_output)
            out get(vm_output, 0)
            "#,
        )
        .unwrap();

        assert_eq!(output, "1\n42\n");
    }

    #[test]
    fn source_runtime_uses_apl_vm_else_if_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = "AVInt x = 2 if x == 1 { out 1 } else if x == 2 { out 42 } else { out 0 }"
            VTime vm_output = vm.run_source(source)

            out len(vm_output)
            out get(vm_output, 0)
            "#,
        )
        .unwrap();

        assert_eq!(output, "1\n42\n");
    }

    #[test]
    fn source_runtime_uses_apl_vm_while_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = "AVInt x = 0 while ( x < 3 ) ( 10 ) { x += 1 out x }"
            VTime vm_output = vm.run_source(source)

            out len(vm_output)
            out get(vm_output, 0)
            out get(vm_output, 1)
            out get(vm_output, 2)
            "#,
        )
        .unwrap();

        assert_eq!(output, "3\n1\n2\n3\n");
    }

    #[test]
    fn source_runtime_uses_apl_vm_loop_flow_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = "AVInt x = 0 while ( x < 10 ) ( 20 ) { x += 1 if x == 2 { continue } if x == 5 { break } out x } out x"
            VTime vm_output = vm.run_source(source)

            out len(vm_output)
            out get(vm_output, 0)
            out get(vm_output, 1)
            out get(vm_output, 2)
            out get(vm_output, 3)
            "#,
        )
        .unwrap();

        assert_eq!(output, "4\n1\n3\n4\n5\n");
    }

    #[test]
    fn source_runtime_uses_apl_vm_function_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = "func sum_two(left, right) { return left + right } AVInt x = sum_two(20, 22) out x"
            VTime vm_output = vm.run_source(source)

            out len(vm_output)
            out get(vm_output, 0)
            "#,
        )
        .unwrap();

        assert_eq!(output, "1\n42\n");
    }

    #[test]
    fn source_runtime_uses_apl_vm_list_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = "List items = [1, 2] add(items, 3) VTime first = get(items, 0) VTime last = pop(items) out first out last out len(items)"
            VTime vm_output = vm.run_source(source)

            out len(vm_output)
            out get(vm_output, 0)
            out get(vm_output, 1)
            out get(vm_output, 2)
            "#,
        )
        .unwrap();

        assert_eq!(output, "3\n1\n3\n2\n");
    }

    #[test]
    fn source_runtime_uses_apl_vm_index_slice_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = join(["List items = [1, 2, 3, 4] VTime first = items[0] VTime tail = items[1:] AVStr text = ", char(34), "APL", char(34), " out first out get(tail, 0) out get(tail, 2) out text[::-1]"], "")
            VTime vm_output = vm.run_source(source)

            out len(vm_output)
            out get(vm_output, 0)
            out get(vm_output, 1)
            out get(vm_output, 2)
            out get(vm_output, 3)
            "#,
        )
        .unwrap();

        assert_eq!(output, "4\n1\n2\n4\nLPA\n");
    }

    #[test]
    fn source_runtime_uses_apl_vm_pick_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = "List items = [1, 2, 3, 4] AVInt total = 0 pick(items): item { if item == 2 { continue } if item == 4 { break } total += item } out total out item"
            VTime vm_output = vm.run_source(source)

            out len(vm_output)
            out get(vm_output, 0)
            out get(vm_output, 1)
            "#,
        )
        .unwrap();

        assert_eq!(output, "2\n4\nNONE\n");
    }

    #[test]
    fn source_runtime_uses_apl_vm_info_and_secret_out_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = join(["AVStr typ = ", char(34), char(34), " AVStr level = ", char(34), char(34), " SASVStr master = ", char(34), "root", char(34), " typ, level = info(master) out typ out level out master"], "")
            VTime vm_output = vm.run_source(source)

            out len(vm_output)
            out get(vm_output, 0)
            out get(vm_output, 1)
            out get(vm_output, 2)
            "#,
        )
        .unwrap();

        assert_eq!(output, "3\nStr\nSASV\nDENIED\n");
    }

    #[test]
    fn source_runtime_uses_apl_vm_derived_secret_out_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = join(["ASVStr token = ", char(34), "root", char(34), " VTime tmp = token List items = [token] out tmp out token == token out items out get(items, 0)"], "")
            VTime vm_output = vm.run_source(source)

            out len(vm_output)
            out get(vm_output, 0)
            out get(vm_output, 1)
            out get(vm_output, 2)
            out get(vm_output, 3)
            "#,
        )
        .unwrap();

        assert_eq!(output, "4\nDENIED\nDENIED\nDENIED\nDENIED\n");
    }

    #[test]
    fn source_runtime_rejects_apl_vm_secret_downgrade_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = join(["out ", char(34), "before", char(34), " ASVStr token = ", char(34), "root", char(34), " VTime tmp = token AVStr public = tmp out public out ", char(34), "after", char(34)], "")
            VTime vm_output = vm.run_source(source)

            out len(vm_output)
            out get(vm_output, 0)
            out get(vm_output, 1)
            "#,
        )
        .unwrap();

        assert_eq!(output, "1\nbefore\nNONE\n");
    }

    #[test]
    fn source_runtime_rejects_apl_vm_duplicate_names_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr duplicate_var = join(["AVInt x = 1 out x AVStr x = ", char(34), "bad", char(34), " out x"], "")
            AVStr duplicate_func = "func same() { return 1 } func same() { return 2 } out same()"
            VTime var_output = vm.run_source(duplicate_var)
            VTime func_output = vm.run_source(duplicate_func)

            out len(var_output)
            out get(var_output, 0)
            out get(var_output, 1)
            out len(func_output)
            out get(func_output, 0)
            "#,
        )
        .unwrap();

        assert_eq!(output, "1\n1\nNONE\n0\nNONE\n");
    }

    #[test]
    fn source_runtime_rejects_apl_vm_unknown_mutation_targets_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr assign_source = join(["out ", char(34), "before", char(34), " missing = 4 out ", char(34), "after", char(34)], "")
            AVStr secretup_source = join(["out ", char(34), "start", char(34), " secretup(missing) out ", char(34), "after", char(34)], "")
            VTime assign_output = vm.run_source(assign_source)
            VTime secretup_output = vm.run_source(secretup_source)

            out len(assign_output)
            out get(assign_output, 0)
            out get(assign_output, 1)
            out len(secretup_output)
            out get(secretup_output, 0)
            out get(secretup_output, 1)
            "#,
        )
        .unwrap();

        assert_eq!(output, "1\nbefore\nNONE\n1\nstart\nNONE\n");
    }

    #[test]
    fn source_runtime_rejects_apl_vm_bad_compound_assignments_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr public_source = "AVInt x = 1 x += 2 out x"
            AVStr secret_source = "ASVInt x = 1 x += 2 out x"
            AVStr string_source = join(["AVStr text = ", char(34), "a", char(34), " text += ", char(34), "b", char(34), " out text"], "")
            AVStr list_source = "List items = [1] items = [2] out items"
            VTime public_output = vm.run_source(public_source)
            VTime secret_output = vm.run_source(secret_source)
            VTime string_output = vm.run_source(string_source)
            VTime list_output = vm.run_source(list_source)

            out get(public_output, 0)
            out len(secret_output)
            out len(string_output)
            out len(list_output)
            "#,
        )
        .unwrap();

        assert_eq!(output, "3\n0\n0\n0\n");
    }

    #[test]
    fn source_runtime_rejects_apl_vm_bad_info_targets_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr ok_source = join(["AVStr typ = ", char(34), char(34), " AVStr level = ", char(34), char(34), " SASVStr master = ", char(34), "root", char(34), " typ, level = info(master) out typ out level"], "")
            AVStr missing_target_source = join(["out ", char(34), "before", char(34), " SASVStr master = ", char(34), "root", char(34), " typ, level = info(master) out typ"], "")
            AVStr secret_target_source = join(["ASVStr typ = ", char(34), char(34), " AVStr level = ", char(34), char(34), " AVStr public = ", char(34), "x", char(34), " typ, level = info(public) out level"], "")
            AVStr vtime_source = join(["AVStr typ = ", char(34), char(34), " AVStr level = ", char(34), char(34), " VTime tmp = 1 typ, level = info(tmp) out typ"], "")
            VTime ok_output = vm.run_source(ok_source)
            VTime missing_target_output = vm.run_source(missing_target_source)
            VTime secret_target_output = vm.run_source(secret_target_source)
            VTime vtime_output = vm.run_source(vtime_source)

            out get(ok_output, 0)
            out get(ok_output, 1)
            out len(missing_target_output)
            out get(missing_target_output, 0)
            out len(secret_target_output)
            out len(vtime_output)
            "#,
        )
        .unwrap();

        assert_eq!(output, "Str\nSASV\n1\nbefore\n0\n0\n");
    }

    #[test]
    fn source_runtime_guards_apl_vm_list_mutation_builtins_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = "List items = [1] add(items, 2) AVInt x = 7 VTime bad_add = add(x, 9) VTime missing_add = add(missing, 1) VTime missing_pop = pop(missing) out len(items) out pop(items) out len(items) out bad_add out missing_add out missing_pop"
            VTime vm_output = vm.run_source(source)

            out len(vm_output)
            out get(vm_output, 0)
            out get(vm_output, 1)
            out get(vm_output, 2)
            out get(vm_output, 3)
            out get(vm_output, 4)
            out get(vm_output, 5)
            "#,
        )
        .unwrap();

        assert_eq!(output, "6\n2\n2\n1\nNONE\nNONE\nNONE\n");
    }

    #[test]
    fn source_runtime_uses_apl_vm_secret_aware_stop_fail_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr public_stop = join(["stop ", char(34), "done", char(34), " out ", char(34), "after", char(34)], "")
            AVStr secret_stop = join(["ASVStr token = ", char(34), "root", char(34), " stop token out ", char(34), "after", char(34)], "")
            AVStr public_fail = join(["fail ", char(34), "bad", char(34), " out ", char(34), "after", char(34)], "")
            VTime stop_output = vm.run_source(public_stop)
            VTime secret_output = vm.run_source(secret_stop)
            VTime fail_output = vm.run_source(public_fail)

            out len(stop_output)
            out get(stop_output, 0)
            out len(secret_output)
            out get(secret_output, 0)
            out len(fail_output)
            out get(fail_output, 0)
            "#,
        )
        .unwrap();

        assert_eq!(output, "1\ndone\n1\nDENIED\n1\nbad\n");
    }

    #[test]
    fn source_runtime_uses_apl_vm_grouped_logical_conditions_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = join(["AVBool a = true AVBool b = false ASVBool secret = true if (a == true) and (b != true) { out ", char(34), "ok", char(34), " } if (secret == true) or (b == true) { out secret == true }"], "")
            VTime vm_output = vm.run_source(source)

            out len(vm_output)
            out get(vm_output, 0)
            out get(vm_output, 1)
            "#,
        )
        .unwrap();

        assert_eq!(output, "2\nok\nDENIED\n");
    }

    #[test]
    fn source_runtime_uses_apl_vm_expression_precedence_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = "AVInt x = 1 + 2 * 3 AVInt y = (1 + 2) * 3 AVBool ok = (x == 7) and (y == 9) or false out x out y out ok"
            VTime vm_output = vm.run_source(source)

            out len(vm_output)
            out get(vm_output, 0)
            out get(vm_output, 1)
            out get(vm_output, 2)
            "#,
        )
        .unwrap();

        assert_eq!(output, "3\n7\n9\ntrue\n");
    }

    #[test]
    fn source_runtime_uses_apl_vm_float_literals_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = "AVFloat x = 1.5 AVFloat y = -2.5 AVFloat z = x + y out z out z < 0.0"
            VTime vm_output = vm.run_source(source)

            out len(vm_output)
            out get(vm_output, 0)
            out get(vm_output, 1)
            "#,
        )
        .unwrap();

        assert_eq!(output, "2\n-1\ntrue\n");
    }

    #[test]
    fn source_runtime_uses_apl_vm_unary_expressions_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = "AVInt x = 4 AVInt y = -x AVFloat z = -1.5 AVBool flag = false out y out z out not flag out -(x + 1)"
            VTime vm_output = vm.run_source(source)

            out len(vm_output)
            out get(vm_output, 0)
            out get(vm_output, 1)
            out get(vm_output, 2)
            out get(vm_output, 3)
            "#,
        )
        .unwrap();

        assert_eq!(output, "4\n-4\n-1.5\ntrue\n-5\n");
    }

    #[test]
    fn source_runtime_uses_apl_vm_pow_builtin_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = "AVInt x = pow(2, 3) AVFloat y = pow(2.0, 3) AVInt bad = pow(2, -1) ASVInt hidden = 2 out x out y == 8.0 out bad out pow(hidden, 2)"
            VTime vm_output = vm.run_source(source)

            out len(vm_output)
            out get(vm_output, 0)
            out get(vm_output, 1)
            out get(vm_output, 2)
            out get(vm_output, 3)
            "#,
        )
        .unwrap();

        assert_eq!(output, "4\n8\ntrue\nNONE\nDENIED\n");
    }

    #[test]
    fn source_runtime_uses_apl_vm_self_check_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = join(["AVInt x = 4 out x =self= x += 1 out x =self= ASVStr token = ", char(34), "root", char(34), " out token =self="], "")
            VTime vm_output = vm.run_source(source)

            out len(vm_output)
            out get(vm_output, 0)
            out get(vm_output, 1)
            out get(vm_output, 2)
            "#,
        )
        .unwrap();

        assert_eq!(output, "3\ntrue\nfalse\nDENIED\n");
    }

    #[test]
    fn source_runtime_uses_apl_vm_conversion_builtins_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = join(["AVStr raw = ", char(34), "42", char(34), " AVStr bad = ", char(34), "no", char(34), " ASVStr hidden = ", char(34), "7", char(34), " out int(raw) out int(bad) out bool(", char(34), "true", char(34), ") out str(42) out int(hidden)"], "")
            VTime vm_output = vm.run_source(source)

            out len(vm_output)
            out get(vm_output, 0)
            out get(vm_output, 1)
            out get(vm_output, 2)
            out get(vm_output, 3)
            out get(vm_output, 4)
            "#,
        )
        .unwrap();

        assert_eq!(output, "5\n42\nNONE\ntrue\n42\nDENIED\n");
    }

    #[test]
    fn source_runtime_uses_apl_vm_secretup_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = join(["AVStr token = ", char(34), "root", char(34), " AVStr typ = ", char(34), char(34), " AVStr level = ", char(34), char(34), " secretup(token) typ, level = info(token) out level out token secretup(token) typ, level = info(token) out level out token =self="], "")
            VTime vm_output = vm.run_source(source)

            out len(vm_output)
            out get(vm_output, 0)
            out get(vm_output, 1)
            out get(vm_output, 2)
            out get(vm_output, 3)
            "#,
        )
        .unwrap();

        assert_eq!(output, "4\nASV\nDENIED\nSASV\nDENIED\n");
    }

    #[test]
    fn source_runtime_uses_apl_vm_string_helper_builtins_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = join(["AVStr text = ", char(34), "A P L", char(34), " List parts = split(text, ", char(34), " ", char(34), ") ASVStr hidden = ", char(34), "secret words", char(34), " out len(parts) out join(parts, ", char(34), "-", char(34), ") out contains(text, ", char(34), "P", char(34), ") out ord(", char(34), "A", char(34), ") out char(90) out split(hidden, ", char(34), " ", char(34), ")"], "")
            VTime vm_output = vm.run_source(source)

            out len(vm_output)
            out get(vm_output, 0)
            out get(vm_output, 1)
            out get(vm_output, 2)
            out get(vm_output, 3)
            out get(vm_output, 4)
            out get(vm_output, 5)
            "#,
        )
        .unwrap();

        assert_eq!(output, "6\n3\nA-P-L\ntrue\n65\nZ\nDENIED\n");
    }

    #[test]
    fn source_runtime_uses_apl_vm_tagged_list_elements_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = join(["AVStr typ = ", char(34), char(34), " AVStr level = ", char(34), char(34), " List items = [1:SASV, ", char(34), "public", char(34), "] add(items, ", char(34), "hidden", char(34), ":ASV) typ, level = info(items) out typ out level out items out get(items, 0) out get(items, 1) out get(items, 2)"], "")
            VTime vm_output = vm.run_source(source)

            out len(vm_output)
            out get(vm_output, 0)
            out get(vm_output, 1)
            out get(vm_output, 2)
            out get(vm_output, 3)
            out get(vm_output, 4)
            out get(vm_output, 5)
            "#,
        )
        .unwrap();

        assert_eq!(output, "6\nList\nSASV\nDENIED\nDENIED\npublic\nDENIED\n");
    }

    #[test]
    fn source_runtime_uses_apl_vm_input_placeholders_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = join(["AVStr public = input ASVStr secret = secret input AVStr typ = ", char(34), char(34), " AVStr level = ", char(34), char(34), " typ, level = info(public) out typ out level out public typ, level = info(secret) out typ out level out secret"], "")
            VTime vm_output = vm.run_source(source)

            out len(vm_output)
            out get(vm_output, 0)
            out get(vm_output, 1)
            out get(vm_output, 2)
            out get(vm_output, 3)
            out get(vm_output, 4)
            out get(vm_output, 5)
            "#,
        )
        .unwrap();

        assert_eq!(output, "6\nStr\nAV\nNONE\nStr\nASV\nDENIED\n");
    }

    #[test]
    fn source_runtime_uses_apl_vm_input_stream_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = "AVStr public = input ASVStr secret = secret input AVStr missing = input out public out secret out missing"
            VTime vm_output = vm.run_source_with_input(source, ["hello", "token"])

            out len(vm_output)
            out get(vm_output, 0)
            out get(vm_output, 1)
            out get(vm_output, 2)
            "#,
        )
        .unwrap();

        assert_eq!(output, "3\nhello\nDENIED\nNONE\n");
    }

    #[test]
    fn source_runtime_runs_compiled_bootstrap_scenario_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = "AVStr public = input ASVStr secret = secret input AVInt age = input List values = [age, 2, [3, 4], secret:ASV] func inc(x) { return x + 1 } VTime next = inc(age) VTime first = values[0] out public out next out first out secret out values out len(values) AVStr missing = input out missing"
            VTime vm_output = bootstrap.run_with_input(source, ["hello", "token", "41"])

            out len(vm_output)
            out get(vm_output, 0)
            out get(vm_output, 1)
            out get(vm_output, 2)
            out get(vm_output, 3)
            out get(vm_output, 4)
            out get(vm_output, 5)
            out get(vm_output, 6)
            "#,
        )
        .unwrap();

        assert_eq!(output, "7\nhello\n42\n41\nDENIED\nDENIED\nDENIED\nNONE\n");
    }

    #[test]
    fn source_runtime_uses_apl_bootstrap_facade_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = "AVInt x = input x += 1 out x"
            VTime tokens = bootstrap.tokens(source)
            VTime ast = bootstrap.ast(source)
            VTime program = bootstrap.ir(source)
            VTime vm_output = bootstrap.run_with_input(source, ["4"])

            out len(tokens)
            out parser.node_kind(get(ast, 0))
            out ir.opcode(get(program, 0))
            out get(vm_output, 0)
            "#,
        )
        .unwrap();

        assert_eq!(output, "9\nDecl\nDECL\n5\n");
    }

    #[test]
    fn source_runtime_uses_apl_bootstrap_report_facade_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr ok_source = "AVInt x = input x += 1 out x"
            AVStr stop_source = join(["out ", char(34), "before", char(34), " stop ", char(34), "done", char(34), " out ", char(34), "after", char(34)], "")
            AVStr fail_source = join(["out ", char(34), "before", char(34), " fail ", char(34), "bad", char(34), " out ", char(34), "after", char(34)], "")

            VTime ok_report = bootstrap.run_with_input_report(ok_source, ["4"])
            VTime stop_report = bootstrap.run_report(stop_source)
            VTime fail_report = bootstrap.run_report(fail_source)

            out get(ok_report, 0)
            out get(get(ok_report, 1), 0)
            out get(stop_report, 0)
            out get(get(stop_report, 1), 0)
            out get(get(stop_report, 1), 1)
            out get(fail_report, 0)
            out get(get(fail_report, 1), 0)
            out get(get(fail_report, 1), 1)
            "#,
        )
        .unwrap();

        assert_eq!(output, "OK\n5\nSTOP\nbefore\ndone\nFAIL\nbefore\nbad\n");
    }

    #[test]
    fn source_runtime_uses_apl_bootstrap_compile_report_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr ok_source = "AVInt x = 1 out x"
            AVStr bad_source = "AVInt x 1 out x"

            VTime ok_report = bootstrap.compile_report(ok_source)
            VTime bad_report = bootstrap.compile_report(bad_source)
            VTime bad_run = bootstrap.run_report(bad_source)

            out get(ok_report, 0)
            out ir.opcode(get(get(ok_report, 1), 0))
            out get(bad_report, 0)
            out get(bad_report, 1)
            out get(bad_run, 0)
            out get(get(bad_run, 1), 0)
            "#,
        )
        .unwrap();

        assert_eq!(
            output,
            "OK\nDECL\nFAIL\ndeclaration expects =\nFAIL\ndeclaration expects =\n"
        );
    }

    #[test]
    fn source_runtime_uses_apl_bootstrap_artifact_report_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr ok_source = "AVInt x = 1 out x"
            AVStr bad_source = "AVInt x 1 out x"

            VTime ok_report = bootstrap.artifact_report(ok_source)
            VTime bad_report = bootstrap.artifact_report(bad_source)
            VTime artifact = get(ok_report, 1)

            out get(ok_report, 0)
            out artifact[:7]
            out contains(artifact, "DECL")
            out contains(artifact, "OUT")
            out get(bad_report, 0)
            out get(bad_report, 1)
            "#,
        )
        .unwrap();

        assert_eq!(
            output,
            "OK\nAPLIR1:\ntrue\ntrue\nFAIL\ndeclaration expects =\n"
        );
    }

    #[test]
    fn source_runtime_uses_apl_checker_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr ok_source = "AVInt x = 1 List items = [x] func done() { return x } out done()"
            AVStr duplicate_var = "AVInt x = 1 AVStr x = 2 out x"
            AVStr duplicate_func = "func go() { return 1 } func go() { return 2 } out go()"
            AVStr nested_duplicate = "AVInt x = 1 if true { AVInt x = 2 } out x"
            AVStr nested_func_duplicate = "func go() { return 1 } if true { func go() { return 2 } } out go()"

            VTime ok_report = bootstrap.compile_report(ok_source)
            VTime var_report = bootstrap.compile_report(duplicate_var)
            VTime func_report = bootstrap.compile_report(duplicate_func)
            VTime nested_report = bootstrap.compile_report(nested_duplicate)
            VTime nested_func_report = bootstrap.compile_report(nested_func_duplicate)

            out get(ok_report, 0)
            out ir.opcode(get(get(ok_report, 1), 0))
            out get(var_report, 0)
            out get(var_report, 1)
            out get(func_report, 0)
            out get(func_report, 1)
            out get(nested_report, 0)
            out get(nested_report, 1)
            out get(nested_func_report, 0)
            out get(nested_func_report, 1)
            "#,
        )
        .unwrap();

        assert_eq!(
            output,
            "OK\nDECL\nFAIL\nduplicate name `x`\nFAIL\nduplicate name `go`\nFAIL\nduplicate name `x`\nFAIL\nduplicate name `go`\n"
        );
    }

    #[test]
    fn source_runtime_checker_rejects_unknown_mutation_targets_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr missing_assign = "missing = 1 out missing"
            AVStr list_assign = "List items = [] items = 1"
            AVStr func_assign = "func go() { return 1 } go = 2"
            AVStr secret_compound = "ASVInt hidden = 1 hidden += 1"
            AVStr string_compound = join(["AVStr text = ", char(34), char(34), " text += 1"], "")
            AVStr vtime_compound = "VTime temp = 1 temp += 1 out temp"
            AVStr missing_secretup = "secretup(missing)"
            AVStr list_secretup = "List items = [] secretup(items)"
            AVStr missing_info_target = join(["AVStr level = ", char(34), char(34), " typ, level = info(level)"], "")
            AVStr bad_info_target = join(["AVInt typ = 1 AVStr level = ", char(34), char(34), " typ, level = info(typ)"], "")
            AVStr missing_info_source = join(["AVStr typ = ", char(34), char(34), " AVStr level = ", char(34), char(34), " typ, level = info(missing)"], "")

            VTime assign_report = bootstrap.compile_report(missing_assign)
            VTime list_assign_report = bootstrap.compile_report(list_assign)
            VTime func_assign_report = bootstrap.compile_report(func_assign)
            VTime secret_compound_report = bootstrap.compile_report(secret_compound)
            VTime string_compound_report = bootstrap.compile_report(string_compound)
            VTime vtime_compound_report = bootstrap.run_report(vtime_compound)
            VTime secretup_report = bootstrap.compile_report(missing_secretup)
            VTime list_report = bootstrap.compile_report(list_secretup)
            VTime info_target_report = bootstrap.compile_report(missing_info_target)
            VTime bad_info_report = bootstrap.compile_report(bad_info_target)
            VTime info_source_report = bootstrap.compile_report(missing_info_source)

            out get(assign_report, 0)
            out get(assign_report, 1)
            out get(list_assign_report, 0)
            out get(list_assign_report, 1)
            out get(func_assign_report, 0)
            out get(func_assign_report, 1)
            out get(secret_compound_report, 0)
            out get(secret_compound_report, 1)
            out get(string_compound_report, 0)
            out get(string_compound_report, 1)
            out get(vtime_compound_report, 0)
            out get(get(vtime_compound_report, 1), 0)
            out get(secretup_report, 0)
            out get(secretup_report, 1)
            out get(list_report, 0)
            out get(list_report, 1)
            out get(info_target_report, 0)
            out get(info_target_report, 1)
            out get(bad_info_report, 0)
            out get(bad_info_report, 1)
            out get(info_source_report, 0)
            out get(info_source_report, 1)
            "#,
        )
        .unwrap();

        assert_eq!(
            output,
            "FAIL\nunknown assignment target `missing`\nFAIL\ninvalid assignment target `items`\nFAIL\ninvalid assignment target `go`\nFAIL\ninvalid compound assignment target `hidden`\nFAIL\ninvalid compound assignment target `text`\nOK\n2\nFAIL\nunknown secretup target `missing`\nFAIL\ninvalid secretup target `items`\nFAIL\nunknown info target `typ`\nFAIL\ninvalid info target `typ`\nFAIL\nunknown info source `missing`\n"
        );
    }

    #[test]
    fn source_runtime_checker_rejects_unknown_expression_refs_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr valid = "AVInt x = 1 VTime y = x + 1 out y"
            AVStr missing_out = "out missing"
            AVStr missing_decl = "AVInt x = missing"
            AVStr missing_assignment_value = "AVInt x = 1 x = missing"
            AVStr missing_list = "List items = [missing]"
            AVStr missing_if = "if missing { out 1 }"
            AVStr bad_self = "VTime temp = 1 out temp =self="
            AVStr missing_slice = "List items = [1] VTime part = items[:missing]"

            VTime valid_report = bootstrap.run_report(valid)
            VTime out_report = bootstrap.compile_report(missing_out)
            VTime decl_report = bootstrap.compile_report(missing_decl)
            VTime assign_report = bootstrap.compile_report(missing_assignment_value)
            VTime list_report = bootstrap.compile_report(missing_list)
            VTime if_report = bootstrap.compile_report(missing_if)
            VTime self_report = bootstrap.compile_report(bad_self)
            VTime slice_report = bootstrap.compile_report(missing_slice)

            out get(valid_report, 0)
            out get(get(valid_report, 1), 0)
            out get(out_report, 0)
            out get(out_report, 1)
            out get(decl_report, 0)
            out get(decl_report, 1)
            out get(assign_report, 0)
            out get(assign_report, 1)
            out get(list_report, 0)
            out get(list_report, 1)
            out get(if_report, 0)
            out get(if_report, 1)
            out get(self_report, 0)
            out get(self_report, 1)
            out get(slice_report, 0)
            out get(slice_report, 1)
            "#,
        )
        .unwrap();

        assert_eq!(
            output,
            "OK\n2\nFAIL\nunknown variable `missing`\nFAIL\nunknown variable `missing`\nFAIL\nunknown variable `missing`\nFAIL\nunknown variable `missing`\nFAIL\nunknown variable `missing`\nFAIL\ninvalid self target `temp`\nFAIL\nunknown variable `missing`\n"
        );
    }

    #[test]
    fn source_runtime_checker_validates_function_calls_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr valid_forward = "out go() func go() { return 7 }"
            AVStr builtin_ok = "out len([1, 2])"
            AVStr missing_func = "out missing()"
            AVStr var_as_func = "AVInt x = 1 out x()"
            AVStr bad_arity = "func go(a) { return a } out go()"
            AVStr missing_arg = "out len(missing)"

            VTime valid_report = bootstrap.run_report(valid_forward)
            VTime builtin_report = bootstrap.run_report(builtin_ok)
            VTime missing_func_report = bootstrap.compile_report(missing_func)
            VTime var_func_report = bootstrap.compile_report(var_as_func)
            VTime arity_report = bootstrap.compile_report(bad_arity)
            VTime missing_arg_report = bootstrap.compile_report(missing_arg)

            out get(valid_report, 0)
            out get(get(valid_report, 1), 0)
            out get(builtin_report, 0)
            out get(get(builtin_report, 1), 0)
            out get(missing_func_report, 0)
            out get(missing_func_report, 1)
            out get(var_func_report, 0)
            out get(var_func_report, 1)
            out get(arity_report, 0)
            out get(arity_report, 1)
            out get(missing_arg_report, 0)
            out get(missing_arg_report, 1)
            "#,
        )
        .unwrap();

        assert_eq!(
            output,
            "OK\n7\nOK\n2\nFAIL\nunknown function `missing`\nFAIL\ninvalid function `x`\nFAIL\nwrong argument count `go`\nFAIL\nunknown variable `missing`\n"
        );
    }

    #[test]
    fn source_runtime_uses_apl_vm_typed_input_coercion_prelude() {
        let output = run_source_with_prelude(
            r#"
            AVStr source = "AVInt good = input AVInt bad = input AVFloat ratio = input AVBool flag = input AVStr text = input AVInt assigned = 0 assigned = input out good out bad out ratio out flag out text out assigned"
            VTime vm_output = vm.run_source_with_input(source, ["42", "oops", "2.5", "true", "abc", "7"])

            out len(vm_output)
            out get(vm_output, 0)
            out get(vm_output, 1)
            out get(vm_output, 2)
            out get(vm_output, 3)
            out get(vm_output, 4)
            out get(vm_output, 5)
            "#,
        )
        .unwrap();

        assert_eq!(output, "6\n42\nNONE\n2.5\ntrue\nabc\n7\n");
    }

    #[test]
    fn checker_keeps_split_secret_values_off_output() {
        let program = apl_parser::parse_program(
            r#"
            ASVStr source = "secret words"
            VTime parts = split(source, " ")
            out get(parts, 0)
            "#,
        )
        .unwrap();

        validate_program(&program).unwrap_err();
    }

    #[test]
    fn source_runtime_handles_loop_break_and_continue_with_jump_opcodes() {
        let output = run_source(
            r#"
            AVInt x = 0
            AVInt total = 0

            while (x < 10) (20) {
              x += 1

              if x == 2 {
                continue
              }

              if x == 5 {
                break
              }

              total += x
            }

            out total
            "#,
        )
        .unwrap();

        assert_eq!(output, "8\n");
    }

    #[test]
    fn compiled_program_splits_entry_and_functions() {
        let program = checked_program(
            r#"
            func add_one(value) {
              return value + 1
            }

            AVInt x = add_one(4)
            out x
            "#,
        );
        let compiled = compile_program(&program).unwrap();

        assert_eq!(block_ops(&compiled, compiled.entry).len(), 2);
        assert!(compiled.functions.contains_key("add_one"));
        let function_body = compiled.functions["add_one"].body;
        assert_eq!(block_ops(&compiled, function_body).len(), 1);
        assert!(matches!(
            block_ops(&compiled, function_body)[0],
            StatementOp::Return(_)
        ));
    }

    #[test]
    fn compiled_program_uses_single_linear_code_segment() {
        let program = checked_program(
            r#"
            func add_one(value) {
              return value + 1
            }

            AVInt x = 0
            while (x < 2) (5) {
              x = add_one(x)
            }
            out x
            "#,
        );
        let compiled = compile_program(&program).unwrap();

        assert!(!compiled.code.is_empty());
        for block in &compiled.blocks {
            assert!(block.start <= block.end);
            assert!(block.end <= compiled.code.len());
        }
        assert!(compiled
            .blocks
            .windows(2)
            .all(|pair| pair[0].end <= pair[1].start));
    }

    #[test]
    fn compiled_while_uses_loop_jump_opcodes() {
        let program = checked_program(
            r#"
            AVInt x = 0
            while (x < 3) (10) {
              x += 1
            }
            out x
            "#,
        );
        let compiled = compile_program(&program).unwrap();
        let entry_start = compiled.blocks[compiled.entry].start;
        let entry_ops = block_ops(&compiled, compiled.entry);

        assert!(matches!(entry_ops[1], StatementOp::LoopCheck { .. }));
        assert!(matches!(entry_ops[2], StatementOp::ExecLoopBody { .. }));
        assert!(matches!(
            entry_ops[3],
            StatementOp::Jump { target_pc } if target_pc == entry_start + 1
        ));
        let StatementOp::LoopCheck { exit_pc, .. } = entry_ops[1] else {
            unreachable!();
        };
        assert_eq!(exit_pc, entry_start + 4);
        let StatementOp::ExecLoopBody {
            body,
            continue_pc,
            break_pc,
        } = entry_ops[2]
        else {
            unreachable!();
        };
        assert_eq!(continue_pc, entry_start + 1);
        assert_eq!(break_pc, entry_start + 4);
        assert_ne!(body, compiled.entry);
        assert!(matches!(
            block_ops(&compiled, body)[0],
            StatementOp::Assignment(_)
        ));
    }

    #[test]
    fn block_frame_advances_with_program_counter() {
        let program = checked_program(
            r#"
            AVInt x = 1
            out x
            "#,
        );
        let compiled = compile_program(&program).unwrap();
        let runtime = Runtime::new(vec![]);
        let mut frame = BlockFrame {
            block_id: compiled.entry,
            pc: compiled.blocks[compiled.entry].start,
            loop_iterations: HashMap::new(),
        };
        let mut runtime = runtime;
        runtime.program = Some(&compiled);

        assert!(matches!(
            runtime.next_statement_op(&mut frame).unwrap(),
            Some(StatementOp::VariableDecl(_))
        ));
        assert_eq!(frame.pc, compiled.blocks[compiled.entry].start + 1);
        assert!(matches!(
            runtime.next_statement_op(&mut frame).unwrap(),
            Some(StatementOp::Out(_))
        ));
        assert_eq!(frame.pc, compiled.blocks[compiled.entry].start + 2);
        assert_eq!(runtime.next_statement_op(&mut frame).unwrap(), None);
    }

    #[test]
    fn compiled_if_uses_jump_opcodes() {
        let program = checked_program(
            r#"
            AVBool flag = false
            if flag == true {
              out "yes"
            } else {
              out "no"
            }
            "#,
        );
        let compiled = compile_program(&program).unwrap();
        let entry_ops = block_ops(&compiled, compiled.entry);

        assert!(matches!(entry_ops[1], StatementOp::JumpIfFalse { .. }));
        assert!(matches!(entry_ops[2], StatementOp::ExecScopedBlock(_)));
        assert!(matches!(entry_ops[3], StatementOp::Jump { .. }));
        assert!(matches!(entry_ops[4], StatementOp::ExecScopedBlock(_)));

        let StatementOp::JumpIfFalse { target_pc, .. } = entry_ops[1] else {
            unreachable!();
        };
        assert_eq!(target_pc, compiled.blocks[compiled.entry].start + 4);
        let StatementOp::Jump { target_pc } = entry_ops[3] else {
            unreachable!();
        };
        assert_eq!(target_pc, compiled.blocks[compiled.entry].start + 5);
    }

    #[test]
    fn compiled_expressions_are_stack_opcodes() {
        let program = checked_program("AVInt value = (2 + 3) * 4");
        let compiled = compile_program(&program).unwrap();

        let StatementOp::VariableDecl(decl) = &block_ops(&compiled, compiled.entry)[0] else {
            panic!("expected variable declaration");
        };

        assert_eq!(
            decl.initial_value.ops,
            vec![
                ExpressionOp::PushLiteral(Value::Int(2)),
                ExpressionOp::PushLiteral(Value::Int(3)),
                ExpressionOp::Binary(BinaryOperator::Add),
                ExpressionOp::PushLiteral(Value::Int(4)),
                ExpressionOp::Binary(BinaryOperator::Mul),
            ]
        );
    }

    #[test]
    fn runs_compiled_ir_bytes() {
        let program = checked_program(
            r#"
            func add_one(value) {
              return value + 1
            }

            AVInt x = add_one(4)
            out x
            "#,
        );
        let ir = apl_ir::encode_program(&program).unwrap();
        let compiled = compile_ir_bytes(&ir).unwrap();
        let output = run_compiled_program(&compiled, vec![]).unwrap();

        assert_eq!(output.stdout, "5\n");
    }

    #[test]
    fn source_runtime_keeps_nested_secret_list_values_off_output() {
        let program = apl_parser::parse_program(
            r#"
            List outer = [[1:SASV], "public"]
            out outer
            "#,
        )
        .unwrap();
        validate_program(&program).unwrap_err();
    }

    #[test]
    fn runtime_keeps_dynamically_added_secret_list_values_off_output() {
        let program = Program::new(vec![
            Statement::ListDecl(ListDecl {
                name: "items".to_owned(),
                elements: vec![],
            }),
            Statement::AddToList {
                list_name: "items".to_owned(),
                element: ListElement {
                    value: Expression::Literal(Value::Str("hidden".to_owned())),
                    protection: Some(ProtectionLevel::Asv),
                },
            },
            Statement::Out(Expression::Variable("items".to_owned())),
        ]);

        assert_eq!(
            run_program(&program, vec![]),
            Err(RuntimeError::Message(
                "refusing to output secret value".to_owned()
            ))
        );
    }

    #[test]
    fn negative_index_too_far_returns_none() {
        let output = run_source(
            r#"
            List items = ["a", "b"]
            VTime item = items[-3]
            out item
            "#,
        )
        .unwrap();

        assert_eq!(output, "NONE\n");
    }

    #[test]
    fn stop_inside_function_stops_program() {
        let output = run_source(
            r#"
            func halt() {
              stop "done"
            }

            VTime unused = halt()
            out "after"
            "#,
        )
        .unwrap();

        assert_eq!(output, "done\n");
    }
}
