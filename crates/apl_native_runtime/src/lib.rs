#![allow(clippy::missing_safety_doc)]

//! Bootstrap implementation of the APL native runtime ABI.
//!
//! All exported `apl_rt_*` functions share one safety contract: the caller must
//! pass a live `NativeContext` and valid immutable APLABI1 records/tables for
//! the complete duration of each call. Generated `APLASM1` code is the intended
//! caller and satisfies that contract.

use std::{
    collections::{HashMap, VecDeque},
    io::{self, IsTerminal, Read},
    ptr,
    rc::Rc,
    slice,
};

pub const INPUT_PROTOCOL: &[u8] = b"\0APLIN1:";

fn invalid_input_frame(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn take_frame_length(bytes: &[u8], cursor: &mut usize) -> io::Result<usize> {
    let remaining = bytes
        .get(*cursor..)
        .ok_or_else(|| invalid_input_frame("truncated APL input frame"))?;
    let separator = remaining
        .iter()
        .position(|byte| *byte == b':')
        .ok_or_else(|| invalid_input_frame("missing APL input frame separator"))?;
    if separator == 0 {
        return Err(invalid_input_frame("empty APL input frame length"));
    }
    let value = std::str::from_utf8(&remaining[..separator])
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .ok_or_else(|| invalid_input_frame("invalid APL input frame length"))?;
    *cursor += separator + 1;
    Ok(value)
}

fn decode_input_frame(bytes: &[u8]) -> io::Result<Option<Vec<String>>> {
    if !bytes.starts_with(INPUT_PROTOCOL) {
        return Ok(None);
    }
    let mut cursor = INPUT_PROTOCOL.len();
    let count = take_frame_length(bytes, &mut cursor)?;
    let minimum_frame_bytes = count
        .checked_mul(2)
        .ok_or_else(|| invalid_input_frame("APL input frame value count is too large"))?;
    if minimum_frame_bytes > bytes.len().saturating_sub(cursor) {
        return Err(invalid_input_frame(
            "APL input frame value count exceeds payload",
        ));
    }
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        let length = take_frame_length(bytes, &mut cursor)?;
        let end = cursor
            .checked_add(length)
            .filter(|end| *end <= bytes.len())
            .ok_or_else(|| invalid_input_frame("truncated APL input frame value"))?;
        let value = std::str::from_utf8(&bytes[cursor..end])
            .map_err(|_| invalid_input_frame("APL input frame value is not UTF-8"))?;
        values.push(value.to_owned());
        cursor = end;
    }
    if cursor != bytes.len() {
        return Err(invalid_input_frame("trailing bytes in APL input frame"));
    }
    Ok(Some(values))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Protection {
    Av,
    Asv,
    Sasv,
}

impl Protection {
    fn from_id(id: i64) -> Result<Self, String> {
        match id {
            0 => Ok(Self::Av),
            1 => Ok(Self::Asv),
            2 => Ok(Self::Sasv),
            _ => Err(format!("invalid protection id `{id}`")),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Av => "AV",
            Self::Asv => "ASV",
            Self::Sasv => "SASV",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValueType {
    Int,
    Float,
    Bool,
    Str,
    Bytes,
    Json,
}

#[derive(Debug, PartialEq)]
struct RuntimeString {
    text: String,
    chars: Vec<char>,
}

impl RuntimeString {
    fn new(text: String) -> Self {
        let chars = text.chars().collect();
        Self { text, chars }
    }

    fn as_str(&self) -> &str {
        &self.text
    }
}

impl ValueType {
    fn name(self) -> &'static str {
        match self {
            Self::Int => "Int",
            Self::Float => "Float",
            Self::Bool => "Bool",
            Self::Str => "Str",
            Self::Bytes => "Bytes",
            Self::Json => "Json",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Data {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(Rc<RuntimeString>),
    Bytes(Vec<u8>),
    Json(String),
    List(Rc<Vec<Value>>),
    None,
}

#[derive(Debug, Clone, PartialEq)]
struct Value {
    data: Data,
    protection: Protection,
    direct_input: bool,
}

impl Value {
    fn new(data: Data, protection: Protection) -> Self {
        Self {
            data,
            protection,
            direct_input: false,
        }
    }

    fn none(protection: Protection) -> Self {
        Self::new(Data::None, protection)
    }

    fn str(value: impl Into<String>) -> Self {
        Self::new(
            Data::Str(Rc::new(RuntimeString::new(value.into()))),
            Protection::Av,
        )
    }

    fn list(values: Vec<Self>) -> Self {
        let protection = values
            .iter()
            .map(|value| value.protection)
            .max()
            .unwrap_or(Protection::Av);
        Self::new(Data::List(Rc::new(values)), protection)
    }

    fn stored(mut self) -> Self {
        self.direct_input = false;
        self
    }

    fn with_protection(mut self, protection: Protection) -> Self {
        self.protection = if matches!(self.data, Data::List(_)) {
            self.protection.max(protection)
        } else {
            protection
        };
        self
    }

    fn matches_type(&self, value_type: ValueType) -> bool {
        matches!(
            (&self.data, value_type),
            (Data::Int(_), ValueType::Int)
                | (Data::Float(_), ValueType::Float)
                | (Data::Bool(_), ValueType::Bool)
                | (Data::Str(_), ValueType::Str)
                | (Data::Bytes(_), ValueType::Bytes)
                | (Data::Json(_), ValueType::Json)
        )
    }

    fn convert(self, value_type: ValueType) -> Self {
        let protection = self.protection;
        if matches!(self.data, Data::None) {
            return Self::none(protection);
        }
        let rendered = self.render();
        let data = match value_type {
            ValueType::Int => rendered.parse().ok().map(Data::Int),
            ValueType::Float => rendered.parse().ok().map(Data::Float),
            ValueType::Bool => match rendered.as_str() {
                "true" => Some(Data::Bool(true)),
                "false" => Some(Data::Bool(false)),
                _ => None,
            },
            ValueType::Str => Some(Data::Str(Rc::new(RuntimeString::new(rendered)))),
            ValueType::Bytes => Some(Data::Bytes(rendered.into_bytes())),
            ValueType::Json => Some(Data::Json(rendered)),
        };
        data.map(|data| Self::new(data, protection))
            .unwrap_or_else(|| Self::none(protection))
    }

    fn render(&self) -> String {
        match &self.data {
            Data::Int(value) => value.to_string(),
            Data::Float(value) => value.to_string(),
            Data::Bool(value) => value.to_string(),
            Data::Str(value) => value.text.clone(),
            Data::Json(value) => value.clone(),
            Data::Bytes(value) => format!("{value:?}"),
            Data::List(values) => format!(
                "[{}]",
                values
                    .iter()
                    .map(Value::render)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Data::None => "NONE".to_owned(),
        }
    }

    fn refresh_list_protection(&mut self) {
        if let Data::List(values) = &self.data {
            self.protection = values
                .iter()
                .map(|value| value.protection)
                .max()
                .unwrap_or(Protection::Av);
        }
    }
}

#[derive(Debug, Clone)]
struct AbsoluteSlot {
    value_type: ValueType,
    protection: Protection,
    initial: Value,
    current: Value,
}

#[derive(Debug)]
struct LoopState {
    limit: i64,
    iterations: i64,
    call_depth: usize,
}

#[derive(Debug)]
struct PickState {
    items: Vec<Value>,
    index: usize,
    call_depth: usize,
}

#[derive(Debug)]
enum Termination {
    Running,
    Stopped,
    Failed(String),
    Error(String),
}

#[repr(C)]
#[derive(Debug)]
pub struct NativeContext {
    stack: Vec<Value>,
    absolute: HashMap<String, AbsoluteSlot>,
    lists: HashMap<String, Value>,
    scopes: Vec<HashMap<String, Value>>,
    input: VecDeque<String>,
    stdout: String,
    termination: Termination,
    loops: HashMap<i64, Vec<LoopState>>,
    picks: HashMap<i64, Vec<PickState>>,
    call_frames: Vec<usize>,
    finished: bool,
    interactive_input: bool,
}

impl NativeContext {
    pub fn new(input: Vec<String>) -> Self {
        Self {
            stack: Vec::new(),
            absolute: HashMap::new(),
            lists: HashMap::new(),
            scopes: vec![HashMap::new()],
            input: input.into(),
            stdout: String::new(),
            termination: Termination::Running,
            loops: HashMap::new(),
            picks: HashMap::new(),
            call_frames: Vec::new(),
            finished: false,
            interactive_input: false,
        }
    }

    pub fn from_stdin() -> io::Result<Self> {
        if io::stdin().is_terminal() {
            let mut context = Self::new(Vec::new());
            context.interactive_input = true;
            return Ok(context);
        }
        let mut source = Vec::new();
        io::stdin().read_to_end(&mut source)?;
        if let Some(values) = decode_input_frame(&source)? {
            return Ok(Self::new(values));
        }
        let source =
            String::from_utf8(source).map_err(|_| invalid_input_frame("APL stdin is not UTF-8"))?;
        Ok(Self::new(source.lines().map(str::to_owned).collect()))
    }

    fn fail(&mut self, message: impl Into<String>) -> i32 {
        if matches!(self.termination, Termination::Running) {
            self.termination = Termination::Error(message.into());
        }
        -1
    }

    fn pop(&mut self) -> Result<Value, String> {
        self.stack
            .pop()
            .ok_or_else(|| "native operand stack underflow".to_owned())
    }

    fn pop_values(&mut self, count: usize) -> Result<Vec<Value>, String> {
        if self.stack.len() < count {
            return Err("native operand stack underflow".to_owned());
        }
        Ok(self.stack.split_off(self.stack.len() - count))
    }

    fn lookup(&self, name: &str) -> Result<Value, String> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value.clone().stored());
            }
        }
        if let Some(slot) = self.absolute.get(name) {
            return Ok(slot.current.clone().stored());
        }
        if let Some(value) = self.lists.get(name) {
            return Ok(value.clone().stored());
        }
        Err(format!("unknown variable `{name}`"))
    }

    fn vtime_mut(&mut self, name: &str) -> Option<&mut Value> {
        self.scopes
            .iter_mut()
            .rev()
            .find_map(|scope| scope.get_mut(name))
    }

    fn declare_vtime(&mut self, name: String, value: Value) {
        if self.scopes.is_empty() {
            self.scopes.push(HashMap::new());
        }
        self.scopes.last_mut().unwrap().insert(name, value.stored());
    }

    fn assign_existing(&mut self, name: &str, value: Value) -> Result<(), String> {
        if let Some(target) = self.vtime_mut(name) {
            *target = value.stored();
            return Ok(());
        }
        let Some(slot) = self.absolute.get_mut(name) else {
            return Err(format!("unknown variable `{name}`"));
        };
        if value.protection > slot.protection {
            return Err(format!("refusing to downgrade protection of `{name}`"));
        }
        slot.current = if value.matches_type(slot.value_type) || matches!(value.data, Data::None) {
            value.with_protection(slot.protection).stored()
        } else {
            Value::none(slot.protection)
        };
        Ok(())
    }

    fn mutate_list(&mut self, name: &str) -> Result<&mut Value, String> {
        if self
            .scopes
            .iter()
            .rev()
            .any(|scope| scope.contains_key(name))
        {
            return self
                .vtime_mut(name)
                .ok_or_else(|| format!("unknown list `{name}`"));
        }
        self.lists
            .get_mut(name)
            .ok_or_else(|| format!("unknown list `{name}`"))
    }
}

#[repr(C)]
pub struct AbiRecord {
    kind: u64,
    length: u64,
}

unsafe fn record_bytes<'a>(record: *const AbiRecord) -> Result<&'a [u8], String> {
    if record.is_null() {
        return Err("null ABI record".to_owned());
    }
    let length = (*record).length as usize;
    let payload = record.cast::<u8>().add(16);
    Ok(slice::from_raw_parts(payload, length))
}

unsafe fn record_name(record: *const AbiRecord) -> Result<String, String> {
    if record.is_null() || (*record).kind != 0 {
        return Err("invalid ABI name record".to_owned());
    }
    String::from_utf8(record_bytes(record)?.to_vec())
        .map_err(|_| "ABI name is not UTF-8".to_owned())
}

unsafe fn record_value(record: *const AbiRecord) -> Result<Value, String> {
    if record.is_null() {
        return Err("null ABI constant record".to_owned());
    }
    let bytes = record_bytes(record)?;
    let data = match (*record).kind {
        1 if bytes.len() == 8 => Data::Int(i64::from_le_bytes(bytes.try_into().unwrap())),
        2 if bytes.len() == 8 => Data::Float(f64::from_le_bytes(bytes.try_into().unwrap())),
        3 => {
            let value = String::from_utf8(bytes.to_vec())
                .map_err(|_| "ABI string is not UTF-8".to_owned())?;
            Data::Str(Rc::new(RuntimeString::new(value)))
        }
        4 if bytes.len() == 8 => Data::Bool(u64::from_le_bytes(bytes.try_into().unwrap()) != 0),
        kind => return Err(format!("invalid ABI constant kind `{kind}`")),
    };
    Ok(Value::new(data, Protection::Av))
}

unsafe fn target_name(
    targets: *const *const AbiRecord,
    index: usize,
) -> Result<Option<String>, String> {
    if targets.is_null() {
        return Ok(None);
    }
    let record = ptr::read(targets.add(index));
    if record.is_null() {
        Ok(None)
    } else {
        record_name(record).map(Some)
    }
}

unsafe fn context<'a>(context: *mut NativeContext) -> Result<&'a mut NativeContext, String> {
    context
        .as_mut()
        .ok_or_else(|| "null native runtime context".to_owned())
}

unsafe fn operation(
    context_ptr: *mut NativeContext,
    body: impl FnOnce(&mut NativeContext) -> Result<(), String>,
) -> i32 {
    let context = match context(context_ptr) {
        Ok(context) => context,
        Err(_) => return -1,
    };
    if !matches!(context.termination, Termination::Running) {
        return -1;
    }
    match body(context) {
        Ok(()) => 0,
        Err(error) => context.fail(error),
    }
}

fn eval_unary(operator: i64, value: Value) -> Result<Value, String> {
    let protection = value.protection;
    let data = match (operator, value.data) {
        (_, Data::None) => Data::None,
        (0, Data::Int(value)) => Data::Int(
            value
                .checked_neg()
                .ok_or_else(|| "integer overflow in negate".to_owned())?,
        ),
        (0, Data::Float(value)) => Data::Float(-value),
        (1, Data::Bool(value)) => Data::Bool(!value),
        _ => return Err("invalid unary operands".to_owned()),
    };
    Ok(Value::new(data, protection))
}

fn eval_binary(operator: i64, left: Value, right: Value) -> Result<Value, String> {
    let protection = left.protection.max(right.protection);
    let data = match operator {
        0 => Data::Bool(left.data == right.data),
        1 => Data::Bool(left.data != right.data),
        2..=5 => match (left.data, right.data) {
            (Data::Int(left), Data::Int(right)) => Data::Bool(match operator {
                2 => left > right,
                3 => left < right,
                4 => left >= right,
                _ => left <= right,
            }),
            (Data::Float(left), Data::Float(right)) => Data::Bool(match operator {
                2 => left > right,
                3 => left < right,
                4 => left >= right,
                _ => left <= right,
            }),
            _ => return Err("invalid ordering operands".to_owned()),
        },
        6 => match (left.data, right.data) {
            (Data::Bool(left), Data::Bool(right)) => Data::Bool(left && right),
            _ => return Err("invalid and operands".to_owned()),
        },
        7 => match (left.data, right.data) {
            (Data::Bool(left), Data::Bool(right)) => Data::Bool(left || right),
            _ => return Err("invalid or operands".to_owned()),
        },
        8 => {
            match (left.data, right.data) {
                (Data::Int(left), Data::Int(right)) => Data::Int(
                    left.checked_add(right)
                        .ok_or_else(|| "integer overflow in add".to_owned())?,
                ),
                (Data::Float(left), Data::Float(right)) => Data::Float(left + right),
                (Data::Int(left), Data::Float(right)) => Data::Float(left as f64 + right),
                (Data::Float(left), Data::Int(right)) => Data::Float(left + right as f64),
                (Data::Str(left), Data::Str(right)) => Data::Str(Rc::new(RuntimeString::new(
                    format!("{}{}", left.as_str(), right.as_str()),
                ))),
                _ => return Err("invalid add operands".to_owned()),
            }
        }
        9 => arithmetic(left.data, right.data, "sub", i64::checked_sub, |a, b| a - b)?,
        10 => arithmetic(left.data, right.data, "mul", i64::checked_mul, |a, b| a * b)?,
        11 => match (left.data, right.data) {
            (Data::Int(_), Data::Int(0)) => return Err("division by zero".to_owned()),
            (Data::Int(left), Data::Int(right)) => Data::Int(
                left.checked_div(right)
                    .ok_or_else(|| "integer overflow in div".to_owned())?,
            ),
            (Data::Float(left), Data::Float(right)) => Data::Float(left / right),
            (Data::Int(left), Data::Float(right)) => Data::Float(left as f64 / right),
            (Data::Float(left), Data::Int(right)) => Data::Float(left / right as f64),
            _ => return Err("invalid div operands".to_owned()),
        },
        12 => match (left.data, right.data) {
            (Data::Int(base), Data::Int(exponent)) if exponent >= 0 => u32::try_from(exponent)
                .ok()
                .and_then(|exponent| base.checked_pow(exponent))
                .map(Data::Int)
                .unwrap_or(Data::None),
            (Data::Float(base), Data::Int(exponent)) if exponent >= 0 => i32::try_from(exponent)
                .ok()
                .map(|exponent| Data::Float(base.powi(exponent)))
                .unwrap_or(Data::None),
            _ => Data::None,
        },
        _ => return Err(format!("invalid binary operator id `{operator}`")),
    };
    Ok(Value::new(data, protection))
}

fn arithmetic(
    left: Data,
    right: Data,
    name: &str,
    integer: fn(i64, i64) -> Option<i64>,
    float: fn(f64, f64) -> f64,
) -> Result<Data, String> {
    match (left, right) {
        (Data::Int(left), Data::Int(right)) => integer(left, right)
            .map(Data::Int)
            .ok_or_else(|| format!("integer overflow in {name}")),
        (Data::Float(left), Data::Float(right)) => Ok(Data::Float(float(left, right))),
        (Data::Int(left), Data::Float(right)) => Ok(Data::Float(float(left as f64, right))),
        (Data::Float(left), Data::Int(right)) => Ok(Data::Float(float(left, right as f64))),
        _ => Err(format!("invalid {name} operands")),
    }
}

fn resolve_index(length: usize, index: i64) -> Option<usize> {
    let index = if index < 0 {
        i64::try_from(length).ok()?.checked_add(index)?
    } else {
        index
    };
    usize::try_from(index).ok().filter(|index| *index < length)
}

fn eval_index(target: Value, index: Value) -> Value {
    let protection = target.protection.max(index.protection);
    let Data::Int(index) = index.data else {
        return Value::none(protection);
    };
    let data = match target.data {
        Data::List(values) => {
            return resolve_index(values.len(), index)
                .and_then(|index| values.get(index).cloned())
                .unwrap_or_else(|| Value::none(protection));
        }
        Data::Str(value) => resolve_index(value.chars.len(), index)
            .and_then(|index| value.chars.get(index).copied())
            .map(|value| Data::Str(Rc::new(RuntimeString::new(value.to_string())))),
        Data::Bytes(value) => resolve_index(value.len(), index)
            .and_then(|index| value.get(index).copied())
            .map(|value| Data::Int(i64::from(value))),
        _ => None,
    };
    data.map(|data| Value::new(data, protection))
        .unwrap_or_else(|| Value::none(protection))
}

fn slice_indices(length: usize, start: Option<i64>, end: Option<i64>, step: i64) -> Vec<usize> {
    let length = length as i64;
    let normalize = |value: i64| if value < 0 { length + value } else { value };
    let mut output = Vec::new();
    if step > 0 {
        let mut index = normalize(start.unwrap_or(0)).clamp(0, length);
        let end = normalize(end.unwrap_or(length)).clamp(0, length);
        while index < end {
            output.push(index as usize);
            index += step;
        }
    } else if length > 0 {
        let mut index = start
            .map(normalize)
            .unwrap_or(length - 1)
            .clamp(-1, length - 1);
        let end = end.map(normalize).unwrap_or(-1).clamp(-1, length - 1);
        while index > end {
            output.push(index as usize);
            index += step;
        }
    }
    output
}

fn eval_slice(target: Value, start: Value, end: Value, step: Value) -> Value {
    let protection = target
        .protection
        .max(start.protection)
        .max(end.protection)
        .max(step.protection);
    let optional = |value: Value| match value.data {
        Data::Int(value) => Some(Some(value)),
        Data::None => Some(None),
        _ => None,
    };
    let (Some(start), Some(end), Some(step)) = (optional(start), optional(end), optional(step))
    else {
        return Value::none(protection);
    };
    let step = step.unwrap_or(1);
    if step == 0 {
        return Value::none(protection);
    }
    let data = match target.data {
        Data::Str(value) => Data::Str(Rc::new(RuntimeString::new(
            slice_indices(value.chars.len(), start, end, step)
                .into_iter()
                .map(|index| value.chars[index])
                .collect(),
        ))),
        Data::Bytes(value) => Data::Bytes(
            slice_indices(value.len(), start, end, step)
                .into_iter()
                .map(|index| value[index])
                .collect(),
        ),
        Data::List(values) => Data::List(Rc::new(
            slice_indices(values.len(), start, end, step)
                .into_iter()
                .map(|index| values[index].clone())
                .collect(),
        )),
        _ => return Value::none(protection),
    };
    Value::new(data, protection)
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_push_const(
    context: *mut NativeContext,
    record: *const AbiRecord,
) -> i32 {
    operation(context, |context| {
        context.stack.push(record_value(record)?);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_push_none(context: *mut NativeContext) -> i32 {
    operation(context, |context| {
        context.stack.push(Value::none(Protection::Av));
        Ok(())
    })
}

fn read_input(context: &mut NativeContext, protection: Protection) {
    let input = context.input.pop_front().or_else(|| {
        if !context.interactive_input {
            return None;
        }
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .ok()
            .filter(|read| *read > 0)?;
        Some(line.trim_end_matches(['\r', '\n']).to_owned())
    });
    let mut value = input
        .map(Value::str)
        .unwrap_or_else(|| Value::none(protection))
        .with_protection(protection);
    value.direct_input = true;
    context.stack.push(value);
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_input(context: *mut NativeContext) -> i32 {
    operation(context, |context| {
        read_input(context, Protection::Av);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_secret_input(context: *mut NativeContext) -> i32 {
    operation(context, |context| {
        read_input(context, Protection::Asv);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_load(context: *mut NativeContext, name: *const AbiRecord) -> i32 {
    operation(context, |context| {
        let name = record_name(name)?;
        context.stack.push(context.lookup(&name)?);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_unary(context: *mut NativeContext, operator: i64) -> i32 {
    operation(context, |context| {
        let value = context.pop()?;
        context.stack.push(eval_unary(operator, value)?);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_binary(context: *mut NativeContext, operator: i64) -> i32 {
    operation(context, |context| {
        let right = context.pop()?;
        let left = context.pop()?;
        context.stack.push(eval_binary(operator, left, right)?);
        Ok(())
    })
}

unsafe fn builtin(
    context_ptr: *mut NativeContext,
    arity: usize,
    body: impl FnOnce(&mut NativeContext, Vec<Value>) -> Result<Value, String>,
) -> i32 {
    operation(context_ptr, |context| {
        let args = context.pop_values(arity)?;
        let result = body(context, args)?;
        context.stack.push(result);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_builtin_get(
    context: *mut NativeContext,
    _targets: *const *const AbiRecord,
) -> i32 {
    builtin(context, 2, |_, args| {
        Ok(eval_index(args[0].clone(), args[1].clone()))
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_builtin_split(
    context: *mut NativeContext,
    _targets: *const *const AbiRecord,
) -> i32 {
    builtin(context, 2, |_, args| {
        let protection = args[0].protection.max(args[1].protection);
        let (Data::Str(value), Data::Str(separator)) = (&args[0].data, &args[1].data) else {
            return Ok(Value::none(protection));
        };
        if separator.as_str().is_empty() {
            return Ok(Value::none(protection));
        }
        Ok(Value::list(
            value
                .as_str()
                .split(separator.as_str())
                .map(|part| Value::str(part).with_protection(protection))
                .collect(),
        ))
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_builtin_join(
    context: *mut NativeContext,
    _targets: *const *const AbiRecord,
) -> i32 {
    builtin(context, 2, |_, args| {
        let protection = args[0].protection.max(args[1].protection);
        let (Data::List(values), Data::Str(separator)) = (&args[0].data, &args[1].data) else {
            return Ok(Value::none(protection));
        };
        let mut parts = Vec::with_capacity(values.len());
        for value in values.iter() {
            let Data::Str(value) = &value.data else {
                return Ok(Value::none(protection));
            };
            parts.push(value.as_str());
        }
        Ok(Value::str(parts.join(separator.as_str())).with_protection(protection))
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_builtin_contains(
    context: *mut NativeContext,
    _targets: *const *const AbiRecord,
) -> i32 {
    builtin(context, 2, |_, args| {
        let protection = args[0].protection.max(args[1].protection);
        let (Data::Str(value), Data::Str(needle)) = (&args[0].data, &args[1].data) else {
            return Ok(Value::none(protection));
        };
        Ok(Value::new(
            Data::Bool(value.as_str().contains(needle.as_str())),
            protection,
        ))
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_builtin_pow(
    context: *mut NativeContext,
    _targets: *const *const AbiRecord,
) -> i32 {
    builtin(context, 2, |_, args| {
        let protection = args[0].protection.max(args[1].protection);
        let Data::Int(exponent) = args[1].data else {
            return Ok(Value::none(protection));
        };
        if exponent < 0 {
            return Ok(Value::none(protection));
        }
        let data = match args[0].data {
            Data::Int(base) => u32::try_from(exponent)
                .ok()
                .and_then(|exponent| base.checked_pow(exponent))
                .map(Data::Int),
            Data::Float(base) => i32::try_from(exponent)
                .ok()
                .map(|exponent| Data::Float(base.powi(exponent))),
            _ => None,
        };
        Ok(data
            .map(|data| Value::new(data, protection))
            .unwrap_or_else(|| Value::none(protection)))
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_builtin_add(
    context_ptr: *mut NativeContext,
    targets: *const *const AbiRecord,
) -> i32 {
    let name = match target_name(targets, 0) {
        Ok(Some(name)) => name,
        Ok(None) => {
            return context(context_ptr).map_or(-1, |ctx| ctx.fail("add expects list name"))
        }
        Err(error) => return context(context_ptr).map_or(-1, |ctx| ctx.fail(error)),
    };
    builtin(context_ptr, 2, |context, args| {
        let value = args[1].clone().stored();
        let list = context.mutate_list(&name)?;
        let Data::List(values) = &mut list.data else {
            return Err(format!("`{name}` is not a list"));
        };
        list.protection = list.protection.max(value.protection);
        Rc::make_mut(values).push(value);
        Ok(Value::none(Protection::Av))
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_builtin_len(
    context: *mut NativeContext,
    _targets: *const *const AbiRecord,
) -> i32 {
    builtin(context, 1, |_, args| {
        let protection = args[0].protection;
        let length = match &args[0].data {
            Data::Str(value) => value.chars.len(),
            Data::Bytes(value) => value.len(),
            Data::List(values) => values.len(),
            _ => return Ok(Value::none(protection)),
        };
        Ok(Value::new(Data::Int(length as i64), protection))
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_builtin_ord(
    context: *mut NativeContext,
    _targets: *const *const AbiRecord,
) -> i32 {
    builtin(context, 1, |_, args| {
        let protection = args[0].protection;
        let Data::Str(value) = &args[0].data else {
            return Ok(Value::none(protection));
        };
        Ok(value
            .chars
            .first()
            .copied()
            .map(|value| Value::new(Data::Int(i64::from(u32::from(value))), protection))
            .unwrap_or_else(|| Value::none(protection)))
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_builtin_char(
    context: *mut NativeContext,
    _targets: *const *const AbiRecord,
) -> i32 {
    builtin(context, 1, |_, args| {
        let protection = args[0].protection;
        let Data::Int(value) = args[0].data else {
            return Ok(Value::none(protection));
        };
        Ok(u32::try_from(value)
            .ok()
            .and_then(char::from_u32)
            .map(|value| Value::str(value.to_string()).with_protection(protection))
            .unwrap_or_else(|| Value::none(protection)))
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_builtin_pop(
    context_ptr: *mut NativeContext,
    targets: *const *const AbiRecord,
) -> i32 {
    let name = match target_name(targets, 0) {
        Ok(Some(name)) => name,
        Ok(None) => {
            return context(context_ptr).map_or(-1, |ctx| ctx.fail("pop expects list name"))
        }
        Err(error) => return context(context_ptr).map_or(-1, |ctx| ctx.fail(error)),
    };
    builtin(context_ptr, 1, |context, _| {
        let list = context.mutate_list(&name)?;
        let protection = list.protection;
        let Data::List(values) = &mut list.data else {
            return Ok(Value::none(protection));
        };
        let value = Rc::make_mut(values)
            .pop()
            .unwrap_or_else(|| Value::none(protection));
        list.refresh_list_protection();
        Ok(value)
    })
}

unsafe fn conversion_builtin(context: *mut NativeContext, value_type: ValueType) -> i32 {
    builtin(context, 1, |_, mut args| {
        Ok(args.remove(0).convert(value_type))
    })
}

macro_rules! conversion_export {
    ($name:ident, $type:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(
            context: *mut NativeContext,
            _targets: *const *const AbiRecord,
        ) -> i32 {
            conversion_builtin(context, ValueType::$type)
        }
    };
}

conversion_export!(apl_rt_builtin_int, Int);
conversion_export!(apl_rt_builtin_float, Float);
conversion_export!(apl_rt_builtin_bool, Bool);
conversion_export!(apl_rt_builtin_str, Str);
conversion_export!(apl_rt_builtin_bytes, Bytes);
conversion_export!(apl_rt_builtin_json, Json);

#[no_mangle]
pub unsafe extern "C" fn apl_rt_list(context: *mut NativeContext, count: i64) -> i32 {
    operation(context, |context| {
        let count = usize::try_from(count).map_err(|_| "invalid list size".to_owned())?;
        let values = context.pop_values(count)?;
        context.stack.push(Value::list(values));
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_index(context: *mut NativeContext) -> i32 {
    operation(context, |context| {
        let index = context.pop()?;
        let target = context.pop()?;
        context.stack.push(eval_index(target, index));
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_slice(context: *mut NativeContext) -> i32 {
    operation(context, |context| {
        let step = context.pop()?;
        let end = context.pop()?;
        let start = context.pop()?;
        let target = context.pop()?;
        context.stack.push(eval_slice(target, start, end, step));
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_self(context: *mut NativeContext, name: *const AbiRecord) -> i32 {
    operation(context, |context| {
        let name = record_name(name)?;
        let slot = context
            .absolute
            .get(&name)
            .ok_or_else(|| format!("unknown absolute variable `{name}`"))?;
        context.stack.push(Value::new(
            Data::Bool(slot.current.data == slot.initial.data),
            slot.protection,
        ));
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_tag(context: *mut NativeContext, protection: i64) -> i32 {
    operation(context, |context| {
        let protection = Protection::from_id(protection)?;
        let value = context.pop()?;
        if value.protection > protection {
            return Err("refusing to downgrade list element protection".to_owned());
        }
        context.stack.push(value.with_protection(protection));
        Ok(())
    })
}

fn declaration_type(id: i64) -> Result<(ValueType, Protection), String> {
    if !(0..18).contains(&id) {
        return Err(format!("invalid declaration type id `{id}`"));
    }
    let protection = Protection::from_id(id / 6)?;
    let value_type = match id % 6 {
        0 => ValueType::Int,
        1 => ValueType::Float,
        2 => ValueType::Bool,
        3 => ValueType::Str,
        4 => ValueType::Bytes,
        _ => ValueType::Json,
    };
    Ok((value_type, protection))
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_declare(
    context: *mut NativeContext,
    type_id: i64,
    name: *const AbiRecord,
) -> i32 {
    operation(context, |context| {
        let name = record_name(name)?;
        let (value_type, protection) = declaration_type(type_id)?;
        let mut value = context.pop()?;
        if value.protection > protection {
            return Err("refusing to downgrade declaration protection".to_owned());
        }
        if value.direct_input {
            value = value.convert(value_type);
        } else if !value.matches_type(value_type) && !matches!(value.data, Data::None) {
            value = Value::none(protection);
        }
        value = value.with_protection(protection).stored();
        context.absolute.insert(
            name,
            AbsoluteSlot {
                value_type,
                protection,
                initial: value.clone(),
                current: value,
            },
        );
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_declare_list(
    context: *mut NativeContext,
    name: *const AbiRecord,
) -> i32 {
    operation(context, |context| {
        let name = record_name(name)?;
        let value = context.pop()?.stored();
        if !matches!(value.data, Data::List(_)) {
            return Err("list declaration received non-list value".to_owned());
        }
        context.lists.insert(name, value);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_declare_vtime(
    context: *mut NativeContext,
    name: *const AbiRecord,
) -> i32 {
    operation(context, |context| {
        let name = record_name(name)?;
        let value = context.pop()?;
        context.declare_vtime(name, value);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_assign(
    context: *mut NativeContext,
    name: *const AbiRecord,
    operator: i64,
) -> i32 {
    operation(context, |context| {
        let name = record_name(name)?;
        let right = context.pop()?;
        let value = if operator == 0 {
            right
        } else if (1..=4).contains(&operator) {
            let current = context.lookup(&name)?;
            eval_binary(operator + 7, current, right)?
        } else {
            return Err(format!("invalid assignment operator id `{operator}`"));
        };
        context.assign_existing(&name, value)
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_info_assign(
    context: *mut NativeContext,
    type_target: *const AbiRecord,
    protection_target: *const AbiRecord,
    source: *const AbiRecord,
) -> i32 {
    operation(context, |context| {
        let type_target = record_name(type_target)?;
        let protection_target = record_name(protection_target)?;
        let source = record_name(source)?;
        let slot = context
            .absolute
            .get(&source)
            .ok_or_else(|| format!("unknown absolute variable `{source}`"))?;
        let type_name = slot.value_type.name();
        let protection_name = slot.protection.name();
        context.assign_existing(&type_target, Value::str(type_name))?;
        context.assign_existing(&protection_target, Value::str(protection_name))
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_drop(context: *mut NativeContext) -> i32 {
    operation(context, |context| {
        context.pop()?;
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_secretup(
    context: *mut NativeContext,
    name: *const AbiRecord,
) -> i32 {
    operation(context, |context| {
        let name = record_name(name)?;
        let slot = context
            .absolute
            .get_mut(&name)
            .ok_or_else(|| format!("unknown absolute variable `{name}`"))?;
        slot.protection = match slot.protection {
            Protection::Av => Protection::Asv,
            Protection::Asv => Protection::Sasv,
            Protection::Sasv => return Err(format!("`{name}` is already SASV")),
        };
        slot.initial.protection = slot.protection;
        slot.current.protection = slot.protection;
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_out(context: *mut NativeContext) -> i32 {
    operation(context, |context| {
        let value = context.pop()?;
        if value.protection != Protection::Av {
            return Err("refusing to output secret value".to_owned());
        }
        context.stdout.push_str(&value.render());
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_stop(context_ptr: *mut NativeContext) -> i32 {
    operation(context_ptr, |context| {
        let value = context.pop()?;
        if value.protection != Protection::Av {
            return Err("refusing to stop with secret reason".to_owned());
        }
        if !matches!(value.data, Data::None) {
            context.stdout.push_str(&value.render());
            context.stdout.push('\n');
        }
        context.termination = Termination::Stopped;
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_fail(context_ptr: *mut NativeContext) -> i32 {
    operation(context_ptr, |context| {
        let value = context.pop()?;
        if value.protection != Protection::Av {
            return Err("refusing to fail with secret reason".to_owned());
        }
        context.termination = Termination::Failed(value.render());
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_return(context: *mut NativeContext, _scope_count: i64) -> i32 {
    operation(context, |context| {
        let call_depth = context.call_frames.len();
        let depth = context
            .call_frames
            .pop()
            .ok_or_else(|| "return outside function".to_owned())?;
        context.scopes.truncate(depth);
        context.loops.retain(|_, states| {
            states.retain(|state| state.call_depth < call_depth);
            !states.is_empty()
        });
        context.picks.retain(|_, states| {
            states.retain(|state| state.call_depth < call_depth);
            !states.is_empty()
        });
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_branch_false(context_ptr: *mut NativeContext) -> i32 {
    let context = match context(context_ptr) {
        Ok(context) => context,
        Err(_) => return -1,
    };
    match context.pop() {
        Ok(Value {
            data: Data::Bool(true),
            ..
        }) => 0,
        Ok(Value {
            data: Data::Bool(false) | Data::None,
            ..
        }) => 1,
        Ok(_) => context.fail("condition is not Bool"),
        Err(error) => context.fail(error),
    }
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_scope_enter(context: *mut NativeContext) -> i32 {
    operation(context, |context| {
        context.scopes.push(HashMap::new());
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_scope_exit(context: *mut NativeContext) -> i32 {
    operation(context, |context| {
        if context.scopes.len() <= 1 {
            return Err("native scope stack underflow".to_owned());
        }
        context.scopes.pop();
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_unwind(context: *mut NativeContext, count: i64) -> i32 {
    operation(context, |context| {
        let count = usize::try_from(count).map_err(|_| "invalid unwind count".to_owned())?;
        if count >= context.scopes.len() {
            return Err("native scope stack underflow".to_owned());
        }
        context.scopes.truncate(context.scopes.len() - count);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_loop_enter(
    context: *mut NativeContext,
    id: i64,
    limit: i64,
) -> i32 {
    operation(context, |context| {
        context.loops.entry(id).or_default().push(LoopState {
            limit,
            iterations: 0,
            call_depth: context.call_frames.len(),
        });
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_loop_check(context_ptr: *mut NativeContext, id: i64) -> i32 {
    let context = match context(context_ptr) {
        Ok(context) => context,
        Err(_) => return -1,
    };
    let Some(state) = context
        .loops
        .get_mut(&id)
        .and_then(|states| states.last_mut())
    else {
        return context.fail(format!("unknown loop `{id}`"));
    };
    if state.limit != -1 && state.iterations >= state.limit {
        1
    } else {
        state.iterations += 1;
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_loop_exit(context: *mut NativeContext, id: i64) -> i32 {
    operation(context, |context| {
        let states = context
            .loops
            .get_mut(&id)
            .ok_or_else(|| format!("unknown loop `{id}`"))?;
        states.pop();
        if states.is_empty() {
            context.loops.remove(&id);
        }
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_pick_enter(context: *mut NativeContext, id: i64) -> i32 {
    operation(context, |context| {
        let value = context.pop()?;
        let protection = value.protection;
        let items = match value.data {
            Data::List(values) => values.as_ref().clone(),
            Data::Str(value) => value
                .chars
                .iter()
                .map(|value| Value::str(value.to_string()).with_protection(protection))
                .collect(),
            Data::Bytes(value) => value
                .into_iter()
                .map(|value| Value::new(Data::Int(i64::from(value)), protection))
                .collect(),
            _ => return Err("value is not pickable".to_owned()),
        };
        context.picks.entry(id).or_default().push(PickState {
            items,
            index: 0,
            call_depth: context.call_frames.len(),
        });
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_pick_next(
    context_ptr: *mut NativeContext,
    id: i64,
    item_name: *const AbiRecord,
) -> i32 {
    let context = match context(context_ptr) {
        Ok(context) => context,
        Err(_) => return -1,
    };
    let name = match record_name(item_name) {
        Ok(name) => name,
        Err(error) => return context.fail(error),
    };
    let Some(state) = context
        .picks
        .get_mut(&id)
        .and_then(|states| states.last_mut())
    else {
        return context.fail(format!("unknown pick `{id}`"));
    };
    let Some(value) = state.items.get(state.index).cloned() else {
        return 1;
    };
    state.index += 1;
    context.scopes.push(HashMap::new());
    context.declare_vtime(name, value);
    0
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_pick_exit(context: *mut NativeContext, id: i64) -> i32 {
    operation(context, |context| {
        let states = context
            .picks
            .get_mut(&id)
            .ok_or_else(|| format!("unknown pick `{id}`"))?;
        states.pop();
        if states.is_empty() {
            context.picks.remove(&id);
        }
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_function_enter(
    context: *mut NativeContext,
    params: *const *const AbiRecord,
    count: i64,
) -> i32 {
    operation(context, |context| {
        let count = usize::try_from(count).map_err(|_| "invalid parameter count".to_owned())?;
        let values = context.pop_values(count)?;
        let depth = context.scopes.len();
        context.call_frames.push(depth);
        context.scopes.push(HashMap::new());
        for (index, value) in values.into_iter().enumerate() {
            let name =
                target_name(params, index)?.ok_or_else(|| "null function parameter".to_owned())?;
            context.declare_vtime(name, value);
        }
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn apl_rt_finish(context_ptr: *mut NativeContext) -> i32 {
    let context = match context(context_ptr) {
        Ok(context) => context,
        Err(_) => return 1,
    };
    if !context.finished {
        print!("{}", context.stdout);
        match &context.termination {
            Termination::Failed(message) => eprintln!("{message}"),
            Termination::Error(message) => eprintln!("Runtime error: {message}"),
            Termination::Running | Termination::Stopped => {}
        }
        context.finished = true;
    }
    match context.termination {
        Termination::Running | Termination::Stopped => 0,
        Termination::Failed(_) | Termination::Error(_) => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_slices_follow_apl_semantics() {
        assert_eq!(slice_indices(4, None, None, -1), vec![3, 2, 1, 0]);
        assert_eq!(slice_indices(5, Some(1), Some(4), 2), vec![1, 3]);
    }

    #[test]
    fn values_do_not_lose_list_protection() {
        let value = Value::list(vec![Value::new(Data::Int(1), Protection::Sasv)]);
        assert_eq!(value.protection, Protection::Sasv);
        assert_eq!(
            value.with_protection(Protection::Av).protection,
            Protection::Sasv
        );
    }

    #[test]
    fn power_operator_uses_stable_id_and_preserves_protection() {
        let result = eval_binary(
            12,
            Value::new(Data::Int(2), Protection::Asv),
            Value::new(Data::Int(8), Protection::Av),
        )
        .unwrap();
        assert!(matches!(result.data, Data::Int(256)));
        assert_eq!(result.protection, Protection::Asv);

        let none = eval_binary(
            12,
            Value::new(Data::Int(2), Protection::Av),
            Value::new(Data::Int(-1), Protection::Sasv),
        )
        .unwrap();
        assert!(matches!(none.data, Data::None));
        assert_eq!(none.protection, Protection::Sasv);
    }

    #[test]
    fn framed_input_preserves_multiline_unicode_and_empty_values() {
        let frame = b"\0APLIN1:3:10:first\nline4:\xD0\x90\xD0\x9F0:";
        assert_eq!(
            decode_input_frame(frame).unwrap().unwrap(),
            vec!["first\nline", "\u{410}\u{41f}", ""]
        );
        assert!(decode_input_frame(b"ordinary\ninput").unwrap().is_none());
        assert!(decode_input_frame(b"\0APLIN1:1:4:bad").is_err());
    }

    #[test]
    fn loop_state_is_reentrant_for_recursive_function_slots() {
        let mut context = NativeContext::new(Vec::new());
        let context_ptr = &mut context as *mut NativeContext;
        unsafe {
            assert_eq!(apl_rt_loop_enter(context_ptr, 7, 2), 0);
            assert_eq!(apl_rt_loop_check(context_ptr, 7), 0);
            assert_eq!(apl_rt_loop_enter(context_ptr, 7, 1), 0);
            assert_eq!(apl_rt_loop_check(context_ptr, 7), 0);
            assert_eq!(apl_rt_loop_check(context_ptr, 7), 1);
            assert_eq!(apl_rt_loop_exit(context_ptr, 7), 0);
            assert_eq!(apl_rt_loop_check(context_ptr, 7), 0);
            assert_eq!(apl_rt_loop_check(context_ptr, 7), 1);
            assert_eq!(apl_rt_loop_exit(context_ptr, 7), 0);
        }
    }

    #[test]
    fn return_discards_control_state_owned_by_the_call() {
        let mut context = NativeContext::new(Vec::new());
        let context_ptr = &mut context as *mut NativeContext;
        unsafe {
            assert_eq!(apl_rt_loop_enter(context_ptr, 7, 3), 0);
            assert_eq!(apl_rt_loop_check(context_ptr, 7), 0);
            assert_eq!(apl_rt_function_enter(context_ptr, ptr::null(), 0), 0);
            assert_eq!(apl_rt_loop_enter(context_ptr, 7, 1), 0);
            context.stack.push(Value::str("xy"));
            assert_eq!(apl_rt_pick_enter(context_ptr, 9), 0);
            assert_eq!(apl_rt_return(context_ptr, 0), 0);
            assert!(context.picks.is_empty());
            assert_eq!(context.loops.get(&7).map(Vec::len), Some(1));
            assert_eq!(apl_rt_loop_check(context_ptr, 7), 0);
            assert_eq!(apl_rt_loop_exit(context_ptr, 7), 0);
        }
    }
}
