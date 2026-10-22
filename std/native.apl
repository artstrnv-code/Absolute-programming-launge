# APL-owned low IR.
# This pass turns linked recursive portable IR into linear stack operations.
# Platform backends can consume APLLOW1 without repeating frontend semantics.

AVStr native.IMAGE_MAGIC = "APLLOW1"
AVStr native.HEADER = "APLLOW1:"

AVStr native.OP_CONST = "CONST"
AVStr native.OP_NONE = "NONE"
AVStr native.OP_INPUT = "INPUT"
AVStr native.OP_SECRET_INPUT = "SECRET_INPUT"
AVStr native.OP_LOAD = "LOAD"
AVStr native.OP_UNARY = "UNARY"
AVStr native.OP_BINARY = "BINARY"
AVStr native.OP_CALL_BUILTIN = "CALL_BUILTIN"
AVStr native.OP_CALL_SLOT = "CALL_SLOT"
AVStr native.OP_LIST = "LIST"
AVStr native.OP_INDEX = "INDEX"
AVStr native.OP_SLICE = "SLICE"
AVStr native.OP_SELF = "SELF"
AVStr native.OP_TAG = "TAG"

AVStr native.OP_DECLARE = "DECLARE"
AVStr native.OP_DECLARE_LIST = "DECLARE_LIST"
AVStr native.OP_DECLARE_VTIME = "DECLARE_VTIME"
AVStr native.OP_ASSIGN = "ASSIGN"
AVStr native.OP_INFO_ASSIGN = "INFO_ASSIGN"
AVStr native.OP_DROP = "DROP"
AVStr native.OP_SECRETUP = "SECRETUP"
AVStr native.OP_OUT = "OUT"
AVStr native.OP_STOP = "STOP"
AVStr native.OP_FAIL = "FAIL"
AVStr native.OP_RETURN = "RETURN"

AVStr native.OP_LABEL = "LABEL"
AVStr native.OP_JUMP = "JUMP"
AVStr native.OP_JUMP_IF_FALSE = "JUMP_IF_FALSE"
AVStr native.OP_SCOPE_ENTER = "SCOPE_ENTER"
AVStr native.OP_SCOPE_EXIT = "SCOPE_EXIT"
AVStr native.OP_UNWIND_JUMP = "UNWIND_JUMP"
AVStr native.OP_LOOP_ENTER = "LOOP_ENTER"
AVStr native.OP_LOOP_CHECK = "LOOP_CHECK"
AVStr native.OP_LOOP_EXIT = "LOOP_EXIT"
AVStr native.OP_PICK_ENTER = "PICK_ENTER"
AVStr native.OP_PICK_NEXT = "PICK_NEXT"
AVStr native.OP_PICK_EXIT = "PICK_EXIT"

func native.opcode(instruction) {
  return get(instruction, 0)
}

func native.state(code, next_id) {
  return [code, next_id]
}

func native.emit(state, instruction) {
  VTime code = get(state, 0)
  add(code, instruction)
  return native.state(code, get(state, 1))
}

func native.reserve_id(state) {
  VTime identifier = get(state, 1)
  return [native.state(get(state, 0), identifier + 1), identifier]
}

func native.lower_exprs(expressions, state) {
  pick(expressions): expression {
    state = native.lower_expr(expression, state)
  }

  return state
}

func native.arg_targets(expressions) {
  VTime targets = []

  pick(expressions): expression {
    if ir.expr_opcode(expression) == ir.EXPR_LOAD {
      add(targets, get(expression, 2))
    } else {
      add(targets, NONE)
    }
  }

  return targets
}

func native.lower_expr(expression, state) {
  VTime opcode = ir.expr_opcode(expression)

  if opcode == ir.EXPR_LITERAL {
    return native.emit(state, [native.OP_CONST, get(expression, 1), get(expression, 2)])
  }

  if opcode == ir.EXPR_NONE {
    return native.emit(state, [native.OP_NONE])
  }

  if opcode == ir.EXPR_INPUT {
    return native.emit(state, [native.OP_INPUT])
  }

  if opcode == ir.EXPR_SECRET_INPUT {
    return native.emit(state, [native.OP_SECRET_INPUT])
  }

  if opcode == ir.EXPR_LOAD {
    return native.emit(state, [native.OP_LOAD, get(expression, 2)])
  }

  if opcode == ir.EXPR_UNARY {
    state = native.lower_expr(get(expression, 2), state)
    return native.emit(state, [native.OP_UNARY, get(expression, 1)])
  }

  if opcode == ir.EXPR_BINARY {
    state = native.lower_expr(get(expression, 2), state)
    state = native.lower_expr(get(expression, 3), state)
    return native.emit(state, [native.OP_BINARY, get(expression, 1)])
  }

  if opcode == ir.EXPR_CALL {
    VTime args = get(expression, 2)
    state = native.lower_exprs(args, state)
    return native.emit(state, [native.OP_CALL_BUILTIN, get(expression, 1), len(args), native.arg_targets(args)])
  }

  if opcode == ir.EXPR_CALL_SLOT {
    VTime args = get(expression, 2)
    state = native.lower_exprs(args, state)
    return native.emit(state, [native.OP_CALL_SLOT, get(expression, 1), len(args)])
  }

  if opcode == ir.EXPR_LIST {
    VTime items = get(expression, 1)
    state = native.lower_exprs(items, state)
    return native.emit(state, [native.OP_LIST, len(items)])
  }

  if opcode == ir.EXPR_INDEX {
    state = native.lower_expr(get(expression, 1), state)
    state = native.lower_expr(get(expression, 2), state)
    return native.emit(state, [native.OP_INDEX])
  }

  if opcode == ir.EXPR_SLICE {
    state = native.lower_expr(get(expression, 1), state)
    state = native.lower_expr(get(expression, 2), state)
    state = native.lower_expr(get(expression, 3), state)
    state = native.lower_expr(get(expression, 4), state)
    return native.emit(state, [native.OP_SLICE])
  }

  if opcode == ir.EXPR_SELF {
    return native.emit(state, [native.OP_SELF, get(expression, 2)])
  }

  if opcode == ir.EXPR_TAG {
    state = native.lower_expr(get(expression, 1), state)
    return native.emit(state, [native.OP_TAG, get(expression, 2)])
  }

  return state
}

func native.current_loop(loops) {
  return get(loops, len(loops) - 1)
}

func native.lower_instruction(instruction, state, loops, scope_depth) {
  VTime opcode = ir.opcode(instruction)

  if opcode == ir.OP_DECL {
    state = native.lower_expr(get(instruction, 3), state)
    return native.emit(state, [native.OP_DECLARE, get(instruction, 1), get(instruction, 2)])
  }

  if opcode == ir.OP_LIST_DECL {
    state = native.lower_expr(get(instruction, 2), state)
    return native.emit(state, [native.OP_DECLARE_LIST, get(instruction, 1)])
  }

  if opcode == ir.OP_VTIME_DECL {
    state = native.lower_expr(get(instruction, 2), state)
    return native.emit(state, [native.OP_DECLARE_VTIME, get(instruction, 1)])
  }

  if opcode == ir.OP_INFO_ASSIGN {
    return native.emit(state, [native.OP_INFO_ASSIGN, get(instruction, 1), get(instruction, 2), get(instruction, 3)])
  }

  if opcode == ir.OP_ASSIGN {
    state = native.lower_expr(get(instruction, 3), state)
    return native.emit(state, [native.OP_ASSIGN, get(instruction, 1), get(instruction, 2)])
  }

  if opcode == ir.OP_EXPR {
    state = native.lower_expr(get(instruction, 1), state)
    return native.emit(state, [native.OP_DROP])
  }

  if opcode == ir.OP_SECRETUP {
    return native.emit(state, [native.OP_SECRETUP, get(instruction, 1)])
  }

  if opcode == ir.OP_OUT {
    state = native.lower_expr(get(instruction, 1), state)
    return native.emit(state, [native.OP_OUT])
  }

  if opcode == ir.OP_STOP {
    state = native.lower_expr(get(instruction, 1), state)
    return native.emit(state, [native.OP_STOP])
  }

  if opcode == ir.OP_FAIL {
    state = native.lower_expr(get(instruction, 1), state)
    return native.emit(state, [native.OP_FAIL])
  }

  if opcode == ir.OP_RETURN {
    state = native.lower_expr(get(instruction, 1), state)
    return native.emit(state, [native.OP_RETURN, scope_depth])
  }

  if opcode == ir.OP_BREAK {
    VTime loop = native.current_loop(loops)
    return native.emit(state, [native.OP_UNWIND_JUMP, scope_depth - get(loop, 2), get(loop, 1)])
  }

  if opcode == ir.OP_CONTINUE {
    VTime loop = native.current_loop(loops)
    return native.emit(state, [native.OP_UNWIND_JUMP, scope_depth - get(loop, 2), get(loop, 0)])
  }

  if opcode == ir.OP_IF {
    VTime reserved = native.reserve_id(state)
    state = get(reserved, 0)
    VTime else_label = get(reserved, 1)
    reserved = native.reserve_id(state)
    state = get(reserved, 0)
    VTime end_label = get(reserved, 1)

    state = native.lower_expr(get(instruction, 1), state)
    state = native.emit(state, [native.OP_JUMP_IF_FALSE, else_label])
    state = native.emit(state, [native.OP_SCOPE_ENTER])
    state = native.lower_program(get(instruction, 2), state, loops, scope_depth + 1)
    state = native.emit(state, [native.OP_SCOPE_EXIT])
    state = native.emit(state, [native.OP_JUMP, end_label])
    state = native.emit(state, [native.OP_LABEL, else_label])
    state = native.emit(state, [native.OP_SCOPE_ENTER])
    state = native.lower_program(get(instruction, 3), state, loops, scope_depth + 1)
    state = native.emit(state, [native.OP_SCOPE_EXIT])
    return native.emit(state, [native.OP_LABEL, end_label])
  }

  if opcode == ir.OP_WHILE {
    VTime reserved = native.reserve_id(state)
    state = get(reserved, 0)
    VTime loop_id = get(reserved, 1)
    reserved = native.reserve_id(state)
    state = get(reserved, 0)
    VTime condition_label = get(reserved, 1)
    reserved = native.reserve_id(state)
    state = get(reserved, 0)
    VTime exit_label = get(reserved, 1)
    VTime inner_loops = loops[:]
    add(inner_loops, [condition_label, exit_label, scope_depth])

    state = native.emit(state, [native.OP_LOOP_ENTER, loop_id, get(instruction, 2)])
    state = native.emit(state, [native.OP_LABEL, condition_label])
    state = native.lower_expr(get(instruction, 1), state)
    state = native.emit(state, [native.OP_JUMP_IF_FALSE, exit_label])
    state = native.emit(state, [native.OP_LOOP_CHECK, loop_id, exit_label])
    state = native.emit(state, [native.OP_SCOPE_ENTER])
    state = native.lower_program(get(instruction, 3), state, inner_loops, scope_depth + 1)
    state = native.emit(state, [native.OP_SCOPE_EXIT])
    state = native.emit(state, [native.OP_JUMP, condition_label])
    state = native.emit(state, [native.OP_LABEL, exit_label])
    return native.emit(state, [native.OP_LOOP_EXIT, loop_id])
  }

  if opcode == ir.OP_PICK {
    VTime reserved = native.reserve_id(state)
    state = get(reserved, 0)
    VTime loop_id = get(reserved, 1)
    reserved = native.reserve_id(state)
    state = get(reserved, 0)
    VTime next_label = get(reserved, 1)
    reserved = native.reserve_id(state)
    state = get(reserved, 0)
    VTime exit_label = get(reserved, 1)
    VTime inner_loops = loops[:]
    add(inner_loops, [next_label, exit_label, scope_depth])

    state = native.lower_expr(get(instruction, 1), state)
    state = native.emit(state, [native.OP_PICK_ENTER, loop_id])
    state = native.emit(state, [native.OP_LABEL, next_label])
    state = native.emit(state, [native.OP_PICK_NEXT, loop_id, get(instruction, 2), exit_label])
    state = native.lower_program(get(instruction, 3), state, inner_loops, scope_depth + 1)
    state = native.emit(state, [native.OP_SCOPE_EXIT])
    state = native.emit(state, [native.OP_JUMP, next_label])
    state = native.emit(state, [native.OP_LABEL, exit_label])
    return native.emit(state, [native.OP_PICK_EXIT, loop_id])
  }

  return state
}

func native.lower_program(program, state, loops, scope_depth) {
  pick(program): instruction {
    state = native.lower_instruction(instruction, state, loops, scope_depth)
  }

  return state
}

func native.lower_functions(functions, next_id) {
  VTime names = get(functions, 0)
  VTime params = get(functions, 1)
  VTime bodies = get(functions, 2)
  VTime lowered = []
  VTime index = 0

  while (index < len(names)) (-1) {
    VTime state = native.lower_program(get(bodies, index), native.state([], next_id), [], 0)
    state = native.emit(state, [native.OP_NONE])
    state = native.emit(state, [native.OP_RETURN, 0])
    next_id = get(state, 1)
    add(lowered, [index, get(names, index), get(params, index), get(state, 0)])
    index += 1
  }

  return [lowered, next_id]
}

func native.lower_loaded_image_report(loaded) {
  if vm.loaded_image_is_valid(loaded) != true {
    return [vm.FLOW_FAIL, "invalid loaded image"]
  }

  VTime entry_state = native.lower_program(vm.loaded_program(loaded), native.state([], 0), [], 0)
  VTime lowered_functions = native.lower_functions(vm.loaded_functions(loaded), get(entry_state, 1))
  VTime manifest = artifact.loaded_library_manifest(loaded)

  if manifest == NONE {
    manifest = [linker.MANIFEST_MAGIC, [["base", []]]]
  }

  VTime image = [native.IMAGE_MAGIC, get(entry_state, 0), get(lowered_functions, 0), manifest]

  if native.image_is_valid(image) != true {
    return [vm.FLOW_FAIL, "invalid low IR image"]
  }

  return [vm.FLOW_OK, image]
}

func native.lower_loaded_image(loaded) {
  return get(native.lower_loaded_image_report(loaded), 1)
}

func native.nonnegative_int(value) {
  return (verifier.is_int(value)) and (value >= 0)
}

func native.is_list(value) {
  if value == NONE {
    return false
  }

  if verifier.is_bool(value) {
    return false
  }

  if verifier.is_int(value) {
    return false
  }

  if verifier.is_float(value) {
    return false
  }

  return verifier.is_str(value) != true
}

func native.instruction_is_valid(instruction, function_arities) {
  if native.is_list(instruction) != true {
    return false
  }

  VTime opcode = native.opcode(instruction)

  if opcode == native.OP_CONST {
    return (len(instruction) == 3) and (verifier.literal_is_valid(get(instruction, 1), get(instruction, 2)))
  }

  VTime no_arg = [native.OP_NONE, native.OP_INPUT, native.OP_SECRET_INPUT, native.OP_INDEX, native.OP_SLICE, native.OP_DROP, native.OP_OUT, native.OP_STOP, native.OP_FAIL, native.OP_SCOPE_ENTER, native.OP_SCOPE_EXIT]

  if checker.value_exists(no_arg, opcode) {
    return len(instruction) == 1
  }

  VTime named = [native.OP_LOAD, native.OP_SELF, native.OP_SECRETUP, native.OP_DECLARE_LIST, native.OP_DECLARE_VTIME]

  if checker.value_exists(named, opcode) {
    return (len(instruction) == 2) and (verifier.is_str(get(instruction, 1)))
  }

  if opcode == native.OP_UNARY {
    return (len(instruction) == 2) and (verifier.unary_op_is_valid(get(instruction, 1)))
  }

  if opcode == native.OP_BINARY {
    return (len(instruction) == 2) and (checker.value_exists(parser.BINARY_OPS, get(instruction, 1)))
  }

  if opcode == native.OP_CALL_BUILTIN {
    if len(instruction) != 4 {
      return false
    }

    VTime arg_count = get(instruction, 2)
    VTime targets = get(instruction, 3)

    if (verifier.builtin_arity(get(instruction, 1)) != arg_count) or (native.nonnegative_int(arg_count) != true) {
      return false
    }

    if native.is_list(targets) != true {
      return false
    }

    if len(targets) != arg_count {
      return false
    }

    pick(targets): target {
      if target != NONE {
        if verifier.is_str(target) != true {
          return false
        }
      }
    }

    VTime name = get(instruction, 1)

    if (name == "add") or (name == "pop") {
      if verifier.is_str(get(targets, 0)) != true {
        return false
      }
    }

    return true
  }

  if opcode == native.OP_CALL_SLOT {
    if len(instruction) != 3 {
      return false
    }

    VTime slot = get(instruction, 1)

    if (native.nonnegative_int(slot) != true) or (slot >= len(function_arities)) {
      return false
    }

    return get(instruction, 2) == get(function_arities, slot)
  }

  if opcode == native.OP_LIST {
    return (len(instruction) == 2) and (native.nonnegative_int(get(instruction, 1)))
  }

  if opcode == native.OP_TAG {
    return (len(instruction) == 2) and (verifier.tag_is_valid(get(instruction, 1)))
  }

  if opcode == native.OP_DECLARE {
    return ((len(instruction) == 3) and (parser.is_decl_keyword(get(instruction, 1)))) and (verifier.is_str(get(instruction, 2)))
  }

  if opcode == native.OP_ASSIGN {
    return ((len(instruction) == 3) and (verifier.is_str(get(instruction, 1)))) and (parser.is_assign_op(get(instruction, 2)))
  }

  if opcode == native.OP_INFO_ASSIGN {
    return (((len(instruction) == 4) and (verifier.is_str(get(instruction, 1)))) and (verifier.is_str(get(instruction, 2)))) and (verifier.is_str(get(instruction, 3)))
  }

  VTime indexed = [native.OP_RETURN, native.OP_LABEL, native.OP_JUMP, native.OP_JUMP_IF_FALSE, native.OP_LOOP_EXIT, native.OP_PICK_ENTER, native.OP_PICK_EXIT]

  if checker.value_exists(indexed, opcode) {
    return (len(instruction) == 2) and (native.nonnegative_int(get(instruction, 1)))
  }

  if opcode == native.OP_UNWIND_JUMP {
    return ((len(instruction) == 3) and (native.nonnegative_int(get(instruction, 1)))) and (native.nonnegative_int(get(instruction, 2)))
  }

  if opcode == native.OP_LOOP_ENTER {
    return (((len(instruction) == 3) and (native.nonnegative_int(get(instruction, 1)))) and (verifier.is_int(get(instruction, 2)))) and (get(instruction, 2) >= -1)
  }

  if opcode == native.OP_LOOP_CHECK {
    return ((len(instruction) == 3) and (native.nonnegative_int(get(instruction, 1)))) and (native.nonnegative_int(get(instruction, 2)))
  }

  if opcode == native.OP_PICK_NEXT {
    return (((len(instruction) == 4) and (native.nonnegative_int(get(instruction, 1)))) and (verifier.is_str(get(instruction, 2)))) and (native.nonnegative_int(get(instruction, 3)))
  }

  return false
}

func native.target_label(instruction) {
  VTime opcode = native.opcode(instruction)

  if (opcode == native.OP_JUMP) or (opcode == native.OP_JUMP_IF_FALSE) {
    return get(instruction, 1)
  }

  if (opcode == native.OP_UNWIND_JUMP) or (opcode == native.OP_LOOP_CHECK) {
    return get(instruction, 2)
  }

  if opcode == native.OP_PICK_NEXT {
    return get(instruction, 3)
  }

  return NONE
}

func native.stack_requirement(instruction) {
  VTime opcode = native.opcode(instruction)

  if (opcode == native.OP_UNARY) or (opcode == native.OP_TAG) {
    return 1
  }

  if (opcode == native.OP_BINARY) or (opcode == native.OP_INDEX) {
    return 2
  }

  if opcode == native.OP_SLICE {
    return 4
  }

  if (opcode == native.OP_CALL_BUILTIN) or (opcode == native.OP_CALL_SLOT) {
    return get(instruction, 2)
  }

  if opcode == native.OP_LIST {
    return get(instruction, 1)
  }

  VTime consumers = [native.OP_DECLARE, native.OP_DECLARE_LIST, native.OP_DECLARE_VTIME, native.OP_ASSIGN, native.OP_DROP, native.OP_OUT, native.OP_STOP, native.OP_FAIL, native.OP_RETURN, native.OP_JUMP_IF_FALSE, native.OP_PICK_ENTER]

  if checker.value_exists(consumers, opcode) {
    return 1
  }

  return 0
}

func native.stack_production(instruction) {
  VTime opcode = native.opcode(instruction)
  VTime producers = [native.OP_CONST, native.OP_NONE, native.OP_INPUT, native.OP_SECRET_INPUT, native.OP_LOAD, native.OP_SELF, native.OP_UNARY, native.OP_BINARY, native.OP_CALL_BUILTIN, native.OP_CALL_SLOT, native.OP_LIST, native.OP_INDEX, native.OP_SLICE, native.OP_TAG]

  if checker.value_exists(producers, opcode) {
    return 1
  }

  return 0
}

func native.ends_stack_expression(opcode) {
  VTime boundaries = [native.OP_DECLARE, native.OP_DECLARE_LIST, native.OP_DECLARE_VTIME, native.OP_ASSIGN, native.OP_INFO_ASSIGN, native.OP_DROP, native.OP_SECRETUP, native.OP_OUT, native.OP_STOP, native.OP_FAIL, native.OP_RETURN, native.OP_LABEL, native.OP_JUMP, native.OP_JUMP_IF_FALSE, native.OP_SCOPE_ENTER, native.OP_SCOPE_EXIT, native.OP_UNWIND_JUMP, native.OP_LOOP_ENTER, native.OP_LOOP_CHECK, native.OP_LOOP_EXIT, native.OP_PICK_ENTER, native.OP_PICK_NEXT, native.OP_PICK_EXIT]
  return checker.value_exists(boundaries, opcode)
}

func native.code_stack_is_valid(code) {
  VTime depth = 0

  pick(code): instruction {
    VTime required = native.stack_requirement(instruction)

    if depth < required {
      return false
    }

    depth = depth - required + native.stack_production(instruction)

    if native.ends_stack_expression(native.opcode(instruction)) {
      if depth != 0 {
        return false
      }
    }
  }

  return depth == 0
}

func native.code_is_valid(code, function_arities) {
  if native.is_list(code) != true {
    return false
  }

  VTime labels = []

  pick(code): instruction {
    if native.instruction_is_valid(instruction, function_arities) != true {
      return false
    }

    if native.opcode(instruction) == native.OP_LABEL {
      VTime label = get(instruction, 1)

      if checker.value_exists(labels, label) {
        return false
      }

      add(labels, label)
    }
  }

  pick(code): instruction {
    VTime target = native.target_label(instruction)

    if target != NONE {
      if checker.value_exists(labels, target) != true {
        return false
      }
    }
  }

  return native.code_stack_is_valid(code)
}

func native.functions_are_valid(functions) {
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

    VTime name = get(function, 1)

    if verifier.is_str(name) != true {
      return false
    }

    if checker.value_exists(names, name) {
      return false
    }

    add(names, name)

    if native.is_list(get(function, 2)) != true {
      return false
    }

    if verifier.str_list_is_valid(get(function, 2)) != true {
      return false
    }

    if verifier.values_are_unique(get(function, 2)) != true {
      return false
    }

    add(arities, len(get(function, 2)))

    index += 1
  }

  pick(functions): function {
    if native.code_is_valid(get(function, 3), arities) != true {
      return false
    }
  }

  return true
}

func native.function_arities(functions) {
  VTime arities = []

  pick(functions): function {
    add(arities, len(get(function, 2)))
  }

  return arities
}

func native.image_is_valid(image) {
  if native.is_list(image) != true {
    return false
  }

  if len(image) != 4 {
    return false
  }

  if get(image, 0) != native.IMAGE_MAGIC {
    return false
  }

  VTime functions = get(image, 2)

  if native.functions_are_valid(functions) != true {
    return false
  }

  if native.code_is_valid(get(image, 1), native.function_arities(functions)) != true {
    return false
  }

  if native.is_list(get(image, 3)) != true {
    return false
  }

  return verifier.library_manifest_is_valid(get(image, 3))
}

func native.encode_image_report(image) {
  if native.image_is_valid(image) != true {
    return [vm.FLOW_FAIL, "invalid low IR image"]
  }

  VTime payload = artifact.encode_node(artifact.wrap_value(image))
  return [vm.FLOW_OK, join([native.HEADER, payload], "")]
}

func native.encode_image(image) {
  return get(native.encode_image_report(image), 1)
}

func native.decode_image_report(encoded) {
  VTime header_size = len(native.HEADER)

  if len(encoded) < header_size {
    return [vm.FLOW_FAIL, "invalid low IR header"]
  }

  if encoded[:header_size] != native.HEADER {
    return [vm.FLOW_FAIL, "invalid low IR header"]
  }

  VTime decoded = artifact.decode_node(encoded, header_size)

  if get(decoded, 0) != true {
    return [vm.FLOW_FAIL, get(decoded, 3)]
  }

  if get(decoded, 2) != len(encoded) {
    return [vm.FLOW_FAIL, "trailing low IR data"]
  }

  VTime image = get(decoded, 1)

  if native.image_is_valid(image) != true {
    return [vm.FLOW_FAIL, "invalid low IR image"]
  }

  return [vm.FLOW_OK, image]
}

func native.decode_image(encoded) {
  return get(native.decode_image_report(encoded), 1)
}

func native.lower_linked_artifact_report(encoded) {
  VTime loaded = artifact.decode_loaded_report(encoded)

  if get(loaded, 0) != vm.FLOW_OK {
    return loaded
  }

  VTime lowered = native.lower_loaded_image_report(get(loaded, 1))

  if get(lowered, 0) != vm.FLOW_OK {
    return lowered
  }

  return native.encode_image_report(get(lowered, 1))
}

func native.describe_code(code, indent) {
  VTime parts = []
  VTime index = 0

  pick(code): instruction {
    add(parts, join([indent, str(index), " ", str(instruction), "\n"], ""))
    index += 1
  }

  return join(parts, "")
}

func native.describe_image_report(encoded) {
  VTime decoded = native.decode_image_report(encoded)

  if get(decoded, 0) != vm.FLOW_OK {
    return decoded
  }

  VTime image = get(decoded, 1)
  VTime parts = ["low-ir APLLOW1\nentry\n", native.describe_code(get(image, 1), "  "), "functions\n"]

  pick(get(image, 2)): function {
    add(parts, join(["  slot ", str(get(function, 0)), " ", get(function, 1), str(get(function, 2)), "\n"], ""))
    add(parts, native.describe_code(get(function, 3), "    "))
  }

  add(parts, join(["manifest ", str(get(image, 3)), "\n"], ""))
  return [vm.FLOW_OK, join(parts, "")]
}
