# APL-owned native runtime ABI lowering.
# APLABI1 replaces frontend strings with stable numeric identifiers and a
# deduplicated constant pool before platform assembly emission.

AVStr abi.IMAGE_MAGIC = "APLABI1"
AVStr abi.HEADER = "APLABI1:"

AVStr abi.TARGET_WINDOWS_X64 = "x86_64-pc-windows-msvc"
AVStr abi.TARGET_LINUX_X64 = "x86_64-unknown-linux-gnu"

# [id, triple, architecture, object format, calling convention,
#  word bytes, stack alignment, shadow-space bytes, argument registers]
List abi.TARGETS = [
  [0, abi.TARGET_WINDOWS_X64, "x86_64", "coff", "win64", 8, 16, 32, ["rcx", "rdx", "r8", "r9"]],
  [1, abi.TARGET_LINUX_X64, "x86_64", "elf", "sysv64", 8, 16, 0, ["rdi", "rsi", "rdx", "rcx", "r8", "r9"]]
]

AVInt abi.POOL_NAME = 0
AVInt abi.POOL_INT = 1
AVInt abi.POOL_FLOAT = 2
AVInt abi.POOL_STR = 3
AVInt abi.POOL_BOOL = 4

AVInt abi.PROTECTION_AV = 0
AVInt abi.PROTECTION_ASV = 1
AVInt abi.PROTECTION_SASV = 2

List abi.BUILTINS = ["get", "split", "join", "contains", "pow", "add", "len", "ord", "char", "pop", "int", "float", "bool", "str", "bytes", "json"]
List abi.UNARY_OPERATORS = ["-", "not"]
List abi.BINARY_OPERATORS = ["==", "!=", ">", "<", ">=", "<=", "and", "or", "+", "-", "*", "/", "**"]
List abi.ASSIGN_OPERATORS = ["=", "+=", "-=", "*=", "/="]

AVInt abi.OP_CONST = 0
AVInt abi.OP_NONE = 1
AVInt abi.OP_INPUT = 2
AVInt abi.OP_SECRET_INPUT = 3
AVInt abi.OP_LOAD = 4
AVInt abi.OP_UNARY = 5
AVInt abi.OP_BINARY = 6
AVInt abi.OP_CALL_BUILTIN = 7
AVInt abi.OP_CALL_SLOT = 8
AVInt abi.OP_LIST = 9
AVInt abi.OP_INDEX = 10
AVInt abi.OP_SLICE = 11
AVInt abi.OP_SELF = 12
AVInt abi.OP_TAG = 13
AVInt abi.OP_DECLARE = 14
AVInt abi.OP_DECLARE_LIST = 15
AVInt abi.OP_DECLARE_VTIME = 16
AVInt abi.OP_ASSIGN = 17
AVInt abi.OP_INFO_ASSIGN = 18
AVInt abi.OP_DROP = 19
AVInt abi.OP_SECRETUP = 20
AVInt abi.OP_OUT = 21
AVInt abi.OP_STOP = 22
AVInt abi.OP_FAIL = 23
AVInt abi.OP_RETURN = 24
AVInt abi.OP_LABEL = 25
AVInt abi.OP_JUMP = 26
AVInt abi.OP_JUMP_IF_FALSE = 27
AVInt abi.OP_SCOPE_ENTER = 28
AVInt abi.OP_SCOPE_EXIT = 29
AVInt abi.OP_UNWIND_JUMP = 30
AVInt abi.OP_LOOP_ENTER = 31
AVInt abi.OP_LOOP_CHECK = 32
AVInt abi.OP_LOOP_EXIT = 33
AVInt abi.OP_PICK_ENTER = 34
AVInt abi.OP_PICK_NEXT = 35
AVInt abi.OP_PICK_EXIT = 36

List abi.OPCODE_NAMES = [
  "CONST", "NONE", "INPUT", "SECRET_INPUT", "LOAD", "UNARY", "BINARY",
  "CALL_BUILTIN", "CALL_SLOT", "LIST", "INDEX", "SLICE", "SELF", "TAG",
  "DECLARE", "DECLARE_LIST", "DECLARE_VTIME", "ASSIGN", "INFO_ASSIGN",
  "DROP", "SECRETUP", "OUT", "STOP", "FAIL", "RETURN", "LABEL", "JUMP",
  "JUMP_IF_FALSE", "SCOPE_ENTER", "SCOPE_EXIT", "UNWIND_JUMP", "LOOP_ENTER",
  "LOOP_CHECK", "LOOP_EXIT", "PICK_ENTER", "PICK_NEXT", "PICK_EXIT"
]

func abi.index_of(values, value) {
  VTime index = 0

  while (index < len(values)) (-1) {
    if get(values, index) == value {
      return index
    }

    index += 1
  }

  return -1
}

func abi.target(target_name) {
  pick(abi.TARGETS): target {
    if get(target, 1) == target_name {
      return target
    }
  }

  return NONE
}

func abi.target_is_valid(target) {
  return abi.index_of(abi.TARGETS, target) >= 0
}

func abi.state(pool, code) {
  return [pool, code]
}

func abi.emit(state, instruction) {
  VTime code = get(state, 1)
  add(code, instruction)
  return abi.state(get(state, 0), code)
}

func abi.intern(state, entry) {
  VTime pool = get(state, 0)
  VTime index = abi.index_of(pool, entry)

  if index >= 0 {
    return [state, index]
  }

  VTime next_pool = pool[:]
  add(next_pool, entry)
  return [abi.state(next_pool, get(state, 1)), len(next_pool) - 1]
}

func abi.intern_name(state, name) {
  return abi.intern(state, [abi.POOL_NAME, name])
}

func abi.literal_entry(kind, value) {
  if kind == parser.EXPR_INT {
    return [abi.POOL_INT, value]
  }

  if kind == parser.EXPR_FLOAT {
    return [abi.POOL_FLOAT, value]
  }

  if kind == parser.EXPR_STR {
    return [abi.POOL_STR, value]
  }

  return [abi.POOL_BOOL, value]
}

func abi.protection_id(protection) {
  if protection == "AV" {
    return abi.PROTECTION_AV
  }

  if protection == "ASV" {
    return abi.PROTECTION_ASV
  }

  if protection == "SASV" {
    return abi.PROTECTION_SASV
  }

  return -1
}

func abi.lower_name_refs(names, state) {
  VTime refs = []

  pick(names): name {
    VTime interned = abi.intern_name(state, name)
    state = get(interned, 0)
    add(refs, get(interned, 1))
  }

  return [state, refs]
}

func abi.lower_targets(targets, state) {
  VTime refs = []

  pick(targets): target {
    if target == NONE {
      add(refs, -1)
    } else {
      VTime interned = abi.intern_name(state, target)
      state = get(interned, 0)
      add(refs, get(interned, 1))
    }
  }

  return [state, refs]
}

func abi.lower_instruction(instruction, state) {
  VTime opcode = native.opcode(instruction)

  if opcode == native.OP_CONST {
    VTime interned = abi.intern(state, abi.literal_entry(get(instruction, 1), get(instruction, 2)))
    state = get(interned, 0)
    return abi.emit(state, [abi.OP_CONST, get(interned, 1)])
  }

  if opcode == native.OP_NONE {
    return abi.emit(state, [abi.OP_NONE])
  }

  if opcode == native.OP_INPUT {
    return abi.emit(state, [abi.OP_INPUT])
  }

  if opcode == native.OP_SECRET_INPUT {
    return abi.emit(state, [abi.OP_SECRET_INPUT])
  }

  VTime named_opcodes = [native.OP_LOAD, native.OP_SELF, native.OP_DECLARE_LIST, native.OP_DECLARE_VTIME, native.OP_SECRETUP]

  if checker.value_exists(named_opcodes, opcode) {
    VTime interned = abi.intern_name(state, get(instruction, 1))
    state = get(interned, 0)
    VTime lowered_opcode = abi.OP_LOAD

    if opcode == native.OP_SELF {
      lowered_opcode = abi.OP_SELF
    } else if opcode == native.OP_DECLARE_LIST {
      lowered_opcode = abi.OP_DECLARE_LIST
    } else if opcode == native.OP_DECLARE_VTIME {
      lowered_opcode = abi.OP_DECLARE_VTIME
    } else if opcode == native.OP_SECRETUP {
      lowered_opcode = abi.OP_SECRETUP
    }

    return abi.emit(state, [lowered_opcode, get(interned, 1)])
  }

  if opcode == native.OP_UNARY {
    return abi.emit(state, [abi.OP_UNARY, abi.index_of(abi.UNARY_OPERATORS, get(instruction, 1))])
  }

  if opcode == native.OP_BINARY {
    return abi.emit(state, [abi.OP_BINARY, abi.index_of(abi.BINARY_OPERATORS, get(instruction, 1))])
  }

  if opcode == native.OP_CALL_BUILTIN {
    VTime lowered_targets = abi.lower_targets(get(instruction, 3), state)
    state = get(lowered_targets, 0)
    return abi.emit(state, [abi.OP_CALL_BUILTIN, abi.index_of(abi.BUILTINS, get(instruction, 1)), get(instruction, 2), get(lowered_targets, 1)])
  }

  if opcode == native.OP_CALL_SLOT {
    return abi.emit(state, [abi.OP_CALL_SLOT, get(instruction, 1), get(instruction, 2)])
  }

  if opcode == native.OP_LIST {
    return abi.emit(state, [abi.OP_LIST, get(instruction, 1)])
  }

  if opcode == native.OP_INDEX {
    return abi.emit(state, [abi.OP_INDEX])
  }

  if opcode == native.OP_SLICE {
    return abi.emit(state, [abi.OP_SLICE])
  }

  if opcode == native.OP_TAG {
    return abi.emit(state, [abi.OP_TAG, abi.protection_id(get(instruction, 1))])
  }

  if opcode == native.OP_DECLARE {
    VTime interned = abi.intern_name(state, get(instruction, 2))
    state = get(interned, 0)
    return abi.emit(state, [abi.OP_DECLARE, abi.index_of(parser.DECL_KEYWORDS, get(instruction, 1)), get(interned, 1)])
  }

  if opcode == native.OP_ASSIGN {
    VTime interned = abi.intern_name(state, get(instruction, 1))
    state = get(interned, 0)
    return abi.emit(state, [abi.OP_ASSIGN, get(interned, 1), abi.index_of(abi.ASSIGN_OPERATORS, get(instruction, 2))])
  }

  if opcode == native.OP_INFO_ASSIGN {
    VTime lowered = abi.lower_name_refs([get(instruction, 1), get(instruction, 2), get(instruction, 3)], state)
    state = get(lowered, 0)
    VTime refs = get(lowered, 1)
    return abi.emit(state, [abi.OP_INFO_ASSIGN, get(refs, 0), get(refs, 1), get(refs, 2)])
  }

  VTime direct_no_arg = [
    [native.OP_DROP, abi.OP_DROP], [native.OP_OUT, abi.OP_OUT],
    [native.OP_STOP, abi.OP_STOP], [native.OP_FAIL, abi.OP_FAIL],
    [native.OP_SCOPE_ENTER, abi.OP_SCOPE_ENTER], [native.OP_SCOPE_EXIT, abi.OP_SCOPE_EXIT]
  ]

  pick(direct_no_arg): mapping {
    if opcode == get(mapping, 0) {
      return abi.emit(state, [get(mapping, 1)])
    }
  }

  VTime direct_one_arg = [
    [native.OP_RETURN, abi.OP_RETURN], [native.OP_LABEL, abi.OP_LABEL],
    [native.OP_JUMP, abi.OP_JUMP], [native.OP_JUMP_IF_FALSE, abi.OP_JUMP_IF_FALSE],
    [native.OP_LOOP_EXIT, abi.OP_LOOP_EXIT], [native.OP_PICK_ENTER, abi.OP_PICK_ENTER],
    [native.OP_PICK_EXIT, abi.OP_PICK_EXIT]
  ]

  pick(direct_one_arg): mapping {
    if opcode == get(mapping, 0) {
      return abi.emit(state, [get(mapping, 1), get(instruction, 1)])
    }
  }

  if opcode == native.OP_UNWIND_JUMP {
    return abi.emit(state, [abi.OP_UNWIND_JUMP, get(instruction, 1), get(instruction, 2)])
  }

  if opcode == native.OP_LOOP_ENTER {
    return abi.emit(state, [abi.OP_LOOP_ENTER, get(instruction, 1), get(instruction, 2)])
  }

  if opcode == native.OP_LOOP_CHECK {
    return abi.emit(state, [abi.OP_LOOP_CHECK, get(instruction, 1), get(instruction, 2)])
  }

  if opcode == native.OP_PICK_NEXT {
    VTime interned = abi.intern_name(state, get(instruction, 2))
    state = get(interned, 0)
    return abi.emit(state, [abi.OP_PICK_NEXT, get(instruction, 1), get(interned, 1), get(instruction, 3)])
  }

  return state
}

func abi.lower_code(code, state) {
  pick(code): instruction {
    state = abi.lower_instruction(instruction, state)
  }

  return state
}

func abi.lower_functions(functions, state) {
  VTime lowered = []

  pick(functions): function {
    VTime name_ref = abi.intern_name(state, get(function, 1))
    state = get(name_ref, 0)
    VTime params = abi.lower_name_refs(get(function, 2), state)
    state = get(params, 0)
    VTime function_state = abi.lower_code(get(function, 3), abi.state(get(state, 0), []))
    state = abi.state(get(function_state, 0), get(state, 1))
    add(lowered, [get(function, 0), get(name_ref, 1), get(params, 1), get(function_state, 1)])
  }

  return [state, lowered]
}

func abi.lower_image_report(low_image, target_name) {
  if native.image_is_valid(low_image) != true {
    return [vm.FLOW_FAIL, "invalid low IR image"]
  }

  VTime target = abi.target(target_name)

  if target == NONE {
    return [vm.FLOW_FAIL, join(["unsupported native target: ", target_name], "")]
  }

  VTime state = abi.lower_code(get(low_image, 1), abi.state([], []))
  VTime lowered_functions = abi.lower_functions(get(low_image, 2), state)
  state = get(lowered_functions, 0)
  VTime image = [abi.IMAGE_MAGIC, target, get(state, 0), get(state, 1), get(lowered_functions, 1), get(low_image, 3)]

  if abi.image_is_valid(image) != true {
    return [vm.FLOW_FAIL, "invalid native ABI image"]
  }

  return [vm.FLOW_OK, image]
}

func abi.lower_encoded_report(encoded, target_name) {
  VTime decoded = native.decode_image_report(encoded)

  if get(decoded, 0) != vm.FLOW_OK {
    return decoded
  }

  VTime lowered = abi.lower_image_report(get(decoded, 1), target_name)

  if get(lowered, 0) != vm.FLOW_OK {
    return lowered
  }

  return abi.encode_image_report(get(lowered, 1))
}

func abi.nonnegative_int(value) {
  return (verifier.is_int(value)) and (value >= 0)
}

func abi.id_is_valid(value, count) {
  return (abi.nonnegative_int(value)) and (value < count)
}

func abi.pool_entry_is_valid(entry) {
  if native.is_list(entry) != true {
    return false
  }

  if len(entry) != 2 {
    return false
  }

  VTime kind = get(entry, 0)
  VTime value = get(entry, 1)

  if kind == abi.POOL_NAME {
    return verifier.is_str(value)
  }

  if kind == abi.POOL_INT {
    return verifier.is_int(value)
  }

  if kind == abi.POOL_FLOAT {
    return verifier.is_float(value)
  }

  if kind == abi.POOL_STR {
    return verifier.is_str(value)
  }

  if kind == abi.POOL_BOOL {
    return verifier.is_bool(value)
  }

  return false
}

func abi.pool_is_valid(pool) {
  if native.is_list(pool) != true {
    return false
  }

  VTime seen = []

  pick(pool): entry {
    if abi.pool_entry_is_valid(entry) != true {
      return false
    }

    if checker.value_exists(seen, entry) {
      return false
    }

    add(seen, entry)
  }

  return true
}

func abi.pool_ref_is(pool, ref, kind) {
  if abi.id_is_valid(ref, len(pool)) != true {
    return false
  }

  return get(get(pool, ref), 0) == kind
}

func abi.name_ref_is_valid(pool, ref) {
  return abi.pool_ref_is(pool, ref, abi.POOL_NAME)
}

func abi.instruction_is_valid(instruction, pool, function_arities) {
  if native.is_list(instruction) != true {
    return false
  }

  if len(instruction) == 0 {
    return false
  }

  VTime opcode = get(instruction, 0)

  if abi.id_is_valid(opcode, len(abi.OPCODE_NAMES)) != true {
    return false
  }

  if opcode == abi.OP_CONST {
    if len(instruction) != 2 {
      return false
    }

    VTime ref = get(instruction, 1)

    if abi.id_is_valid(ref, len(pool)) != true {
      return false
    }

    return get(get(pool, ref), 0) != abi.POOL_NAME
  }

  VTime no_arg = [abi.OP_NONE, abi.OP_INPUT, abi.OP_SECRET_INPUT, abi.OP_INDEX, abi.OP_SLICE, abi.OP_DROP, abi.OP_OUT, abi.OP_STOP, abi.OP_FAIL, abi.OP_SCOPE_ENTER, abi.OP_SCOPE_EXIT]

  if checker.value_exists(no_arg, opcode) {
    return len(instruction) == 1
  }

  VTime named = [abi.OP_LOAD, abi.OP_SELF, abi.OP_DECLARE_LIST, abi.OP_DECLARE_VTIME, abi.OP_SECRETUP]

  if checker.value_exists(named, opcode) {
    return (len(instruction) == 2) and (abi.name_ref_is_valid(pool, get(instruction, 1)))
  }

  if opcode == abi.OP_UNARY {
    return (len(instruction) == 2) and (abi.id_is_valid(get(instruction, 1), len(abi.UNARY_OPERATORS)))
  }

  if opcode == abi.OP_BINARY {
    return (len(instruction) == 2) and (abi.id_is_valid(get(instruction, 1), len(abi.BINARY_OPERATORS)))
  }

  if opcode == abi.OP_CALL_BUILTIN {
    if len(instruction) != 4 {
      return false
    }

    VTime builtin_id = get(instruction, 1)
    VTime arg_count = get(instruction, 2)
    VTime targets = get(instruction, 3)

    if abi.id_is_valid(builtin_id, len(abi.BUILTINS)) != true {
      return false
    }

    if (abi.nonnegative_int(arg_count) != true) or (verifier.builtin_arity(get(abi.BUILTINS, builtin_id)) != arg_count) {
      return false
    }

    if native.is_list(targets) != true {
      return false
    }

    if len(targets) != arg_count {
      return false
    }

    pick(targets): target {
      if target != -1 {
        if abi.name_ref_is_valid(pool, target) != true {
          return false
        }
      }
    }

    VTime builtin_name = get(abi.BUILTINS, builtin_id)

    if (builtin_name == "add") or (builtin_name == "pop") {
      if get(targets, 0) == -1 {
        return false
      }
    }

    return true
  }

  if opcode == abi.OP_CALL_SLOT {
    if len(instruction) != 3 {
      return false
    }

    VTime slot = get(instruction, 1)

    if abi.id_is_valid(slot, len(function_arities)) != true {
      return false
    }

    return get(instruction, 2) == get(function_arities, slot)
  }

  if opcode == abi.OP_LIST {
    return (len(instruction) == 2) and (abi.nonnegative_int(get(instruction, 1)))
  }

  if opcode == abi.OP_TAG {
    return (len(instruction) == 2) and (abi.id_is_valid(get(instruction, 1), 3))
  }

  if opcode == abi.OP_DECLARE {
    return ((len(instruction) == 3) and (abi.id_is_valid(get(instruction, 1), len(parser.DECL_KEYWORDS)))) and (abi.name_ref_is_valid(pool, get(instruction, 2)))
  }

  if opcode == abi.OP_ASSIGN {
    return ((len(instruction) == 3) and (abi.name_ref_is_valid(pool, get(instruction, 1)))) and (abi.id_is_valid(get(instruction, 2), len(abi.ASSIGN_OPERATORS)))
  }

  if opcode == abi.OP_INFO_ASSIGN {
    if len(instruction) != 4 {
      return false
    }

    return ((abi.name_ref_is_valid(pool, get(instruction, 1))) and (abi.name_ref_is_valid(pool, get(instruction, 2)))) and (abi.name_ref_is_valid(pool, get(instruction, 3)))
  }

  VTime indexed = [abi.OP_RETURN, abi.OP_LABEL, abi.OP_JUMP, abi.OP_JUMP_IF_FALSE, abi.OP_LOOP_EXIT, abi.OP_PICK_ENTER, abi.OP_PICK_EXIT]

  if checker.value_exists(indexed, opcode) {
    return (len(instruction) == 2) and (abi.nonnegative_int(get(instruction, 1)))
  }

  if opcode == abi.OP_UNWIND_JUMP {
    return ((len(instruction) == 3) and (abi.nonnegative_int(get(instruction, 1)))) and (abi.nonnegative_int(get(instruction, 2)))
  }

  if opcode == abi.OP_LOOP_ENTER {
    return (((len(instruction) == 3) and (abi.nonnegative_int(get(instruction, 1)))) and (verifier.is_int(get(instruction, 2)))) and (get(instruction, 2) >= -1)
  }

  if opcode == abi.OP_LOOP_CHECK {
    return ((len(instruction) == 3) and (abi.nonnegative_int(get(instruction, 1)))) and (abi.nonnegative_int(get(instruction, 2)))
  }

  if opcode == abi.OP_PICK_NEXT {
    return (((len(instruction) == 4) and (abi.nonnegative_int(get(instruction, 1)))) and (abi.name_ref_is_valid(pool, get(instruction, 2)))) and (abi.nonnegative_int(get(instruction, 3)))
  }

  return false
}

func abi.target_label(instruction) {
  VTime opcode = get(instruction, 0)

  if (opcode == abi.OP_JUMP) or (opcode == abi.OP_JUMP_IF_FALSE) {
    return get(instruction, 1)
  }

  if (opcode == abi.OP_UNWIND_JUMP) or (opcode == abi.OP_LOOP_CHECK) {
    return get(instruction, 2)
  }

  if opcode == abi.OP_PICK_NEXT {
    return get(instruction, 3)
  }

  return NONE
}

func abi.stack_requirement(instruction) {
  VTime opcode = get(instruction, 0)

  if (opcode == abi.OP_UNARY) or (opcode == abi.OP_TAG) {
    return 1
  }

  if (opcode == abi.OP_BINARY) or (opcode == abi.OP_INDEX) {
    return 2
  }

  if opcode == abi.OP_SLICE {
    return 4
  }

  if (opcode == abi.OP_CALL_BUILTIN) or (opcode == abi.OP_CALL_SLOT) {
    return get(instruction, 2)
  }

  if opcode == abi.OP_LIST {
    return get(instruction, 1)
  }

  VTime consumers = [abi.OP_DECLARE, abi.OP_DECLARE_LIST, abi.OP_DECLARE_VTIME, abi.OP_ASSIGN, abi.OP_DROP, abi.OP_OUT, abi.OP_STOP, abi.OP_FAIL, abi.OP_RETURN, abi.OP_JUMP_IF_FALSE, abi.OP_PICK_ENTER]

  if checker.value_exists(consumers, opcode) {
    return 1
  }

  return 0
}

func abi.stack_production(instruction) {
  VTime opcode = get(instruction, 0)
  VTime producers = [abi.OP_CONST, abi.OP_NONE, abi.OP_INPUT, abi.OP_SECRET_INPUT, abi.OP_LOAD, abi.OP_SELF, abi.OP_UNARY, abi.OP_BINARY, abi.OP_CALL_BUILTIN, abi.OP_CALL_SLOT, abi.OP_LIST, abi.OP_INDEX, abi.OP_SLICE, abi.OP_TAG]

  if checker.value_exists(producers, opcode) {
    return 1
  }

  return 0
}

func abi.ends_stack_expression(opcode) {
  VTime boundaries = [abi.OP_DECLARE, abi.OP_DECLARE_LIST, abi.OP_DECLARE_VTIME, abi.OP_ASSIGN, abi.OP_INFO_ASSIGN, abi.OP_DROP, abi.OP_SECRETUP, abi.OP_OUT, abi.OP_STOP, abi.OP_FAIL, abi.OP_RETURN, abi.OP_LABEL, abi.OP_JUMP, abi.OP_JUMP_IF_FALSE, abi.OP_SCOPE_ENTER, abi.OP_SCOPE_EXIT, abi.OP_UNWIND_JUMP, abi.OP_LOOP_ENTER, abi.OP_LOOP_CHECK, abi.OP_LOOP_EXIT, abi.OP_PICK_ENTER, abi.OP_PICK_NEXT, abi.OP_PICK_EXIT]
  return checker.value_exists(boundaries, opcode)
}

func abi.code_stack_is_valid(code) {
  VTime depth = 0

  pick(code): instruction {
    VTime required = abi.stack_requirement(instruction)

    if depth < required {
      return false
    }

    depth = depth - required + abi.stack_production(instruction)

    if abi.ends_stack_expression(get(instruction, 0)) {
      if depth != 0 {
        return false
      }
    }
  }

  return depth == 0
}

func abi.code_is_valid(code, pool, function_arities) {
  if native.is_list(code) != true {
    return false
  }

  VTime labels = []

  pick(code): instruction {
    if abi.instruction_is_valid(instruction, pool, function_arities) != true {
      return false
    }

    if get(instruction, 0) == abi.OP_LABEL {
      VTime label = get(instruction, 1)

      if checker.value_exists(labels, label) {
        return false
      }

      add(labels, label)
    }
  }

  pick(code): instruction {
    VTime target = abi.target_label(instruction)

    if target != NONE {
      if checker.value_exists(labels, target) != true {
        return false
      }
    }
  }

  return abi.code_stack_is_valid(code)
}

func abi.functions_are_valid(functions, pool) {
  if native.is_list(functions) != true {
    return false
  }

  VTime names = []
  VTime arities = []
  VTime index = 0

  while (index < len(functions)) (-1) {
    VTime function = get(functions, index)

    if native.is_list(function) != true {
      return false
    }

    if len(function) != 4 {
      return false
    }

    if get(function, 0) != index {
      return false
    }

    VTime name_ref = get(function, 1)

    if abi.name_ref_is_valid(pool, name_ref) != true {
      return false
    }

    if checker.value_exists(names, name_ref) {
      return false
    }

    add(names, name_ref)
    VTime params = get(function, 2)

    if native.is_list(params) != true {
      return false
    }

    VTime seen_params = []

    pick(params): param_ref {
      if abi.name_ref_is_valid(pool, param_ref) != true {
        return false
      }

      if checker.value_exists(seen_params, param_ref) {
        return false
      }

      add(seen_params, param_ref)
    }

    add(arities, len(params))
    index += 1
  }

  pick(functions): function {
    if abi.code_is_valid(get(function, 3), pool, arities) != true {
      return false
    }
  }

  return true
}

func abi.function_arities(functions) {
  VTime arities = []

  pick(functions): function {
    add(arities, len(get(function, 2)))
  }

  return arities
}

func abi.image_is_valid(image) {
  if native.is_list(image) != true {
    return false
  }

  if len(image) != 6 {
    return false
  }

  if get(image, 0) != abi.IMAGE_MAGIC {
    return false
  }

  if abi.target_is_valid(get(image, 1)) != true {
    return false
  }

  VTime pool = get(image, 2)

  if abi.pool_is_valid(pool) != true {
    return false
  }

  VTime functions = get(image, 4)

  if abi.functions_are_valid(functions, pool) != true {
    return false
  }

  if abi.code_is_valid(get(image, 3), pool, abi.function_arities(functions)) != true {
    return false
  }

  if native.is_list(get(image, 5)) != true {
    return false
  }

  return verifier.library_manifest_is_valid(get(image, 5))
}

func abi.encode_image_report(image) {
  if abi.image_is_valid(image) != true {
    return [vm.FLOW_FAIL, "invalid native ABI image"]
  }

  VTime payload = artifact.encode_node(artifact.wrap_value(image))
  return [vm.FLOW_OK, join([abi.HEADER, payload], "")]
}

func abi.decode_image_report(encoded) {
  VTime header_size = len(abi.HEADER)

  if len(encoded) < header_size {
    return [vm.FLOW_FAIL, "invalid native ABI header"]
  }

  if encoded[:header_size] != abi.HEADER {
    return [vm.FLOW_FAIL, "invalid native ABI header"]
  }

  VTime decoded = artifact.decode_node(encoded, header_size)

  if get(decoded, 0) != true {
    return [vm.FLOW_FAIL, get(decoded, 3)]
  }

  if get(decoded, 2) != len(encoded) {
    return [vm.FLOW_FAIL, "trailing native ABI data"]
  }

  VTime image = get(decoded, 1)

  if abi.image_is_valid(image) != true {
    return [vm.FLOW_FAIL, "invalid native ABI image"]
  }

  return [vm.FLOW_OK, image]
}

func abi.pool_value(pool, ref) {
  return get(get(pool, ref), 1)
}

func abi.describe_code(code, indent) {
  VTime parts = []
  VTime index = 0

  pick(code): instruction {
    VTime opcode = get(instruction, 0)
    add(parts, join([indent, str(index), " ", get(abi.OPCODE_NAMES, opcode), " ", str(instruction), "\n"], ""))
    index += 1
  }

  return join(parts, "")
}

func abi.describe_image_report(encoded) {
  VTime decoded = abi.decode_image_report(encoded)

  if get(decoded, 0) != vm.FLOW_OK {
    return decoded
  }

  VTime image = get(decoded, 1)
  VTime target = get(image, 1)
  VTime pool = get(image, 2)
  VTime parts = ["native-abi APLABI1\ntarget ", get(target, 1), " ", get(target, 4), "\nconstants\n"]
  VTime index = 0

  pick(pool): entry {
    add(parts, join(["  ", str(index), " ", str(entry), "\n"], ""))
    index += 1
  }

  add(parts, "entry\n")
  add(parts, abi.describe_code(get(image, 3), "  "))
  add(parts, "functions\n")

  pick(get(image, 4)): function {
    add(parts, join(["  slot ", str(get(function, 0)), " ", abi.pool_value(pool, get(function, 1)), " refs=", str(get(function, 2)), "\n"], ""))
    add(parts, abi.describe_code(get(function, 3), "    "))
  }

  add(parts, join(["manifest ", str(get(image, 5)), "\n"], ""))
  return [vm.FLOW_OK, join(parts, "")]
}
