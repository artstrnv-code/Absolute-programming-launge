# APL loaded-image verifier.
# It treats decoded artifacts as untrusted and validates their complete shape.

func verifier.is_int(value) {
  if value == NONE {
    return false
  }

  return int(str(value)) == value
}

func verifier.is_float(value) {
  if value == NONE {
    return false
  }

  return float(str(value)) == value
}

func verifier.is_str(value) {
  if value == NONE {
    return false
  }

  return str(value) == value
}

func verifier.is_bool(value) {
  if value == true {
    return true
  }

  return value == false
}

func verifier.tag_is_valid(tag) {
  if tag == "AV" {
    return true
  }

  if tag == "ASV" {
    return true
  }

  return tag == "SASV"
}

func verifier.unary_op_is_valid(op) {
  if op == "-" {
    return true
  }

  return op == "not"
}

func verifier.builtin_arity(name) {
  if name == "get" {
    return 2
  }

  if name == "split" {
    return 2
  }

  if name == "join" {
    return 2
  }

  if name == "contains" {
    return 2
  }

  if name == "pow" {
    return 2
  }

  if name == "add" {
    return 2
  }

  if name == "len" {
    return 1
  }

  if name == "ord" {
    return 1
  }

  if name == "char" {
    return 1
  }

  if name == "pop" {
    return 1
  }

  if name == "int" {
    return 1
  }

  if name == "float" {
    return 1
  }

  if name == "bool" {
    return 1
  }

  if name == "str" {
    return 1
  }

  if name == "bytes" {
    return 1
  }

  if name == "json" {
    return 1
  }

  return -1
}

func verifier.exprs_are_valid(expressions, function_count) {
  pick(expressions): expression {
    if verifier.expr_is_valid(expression, function_count) != true {
      return false
    }
  }

  return true
}

func verifier.literal_is_valid(kind, value) {
  if kind == parser.EXPR_INT {
    return verifier.is_int(value)
  }

  if kind == parser.EXPR_FLOAT {
    return verifier.is_float(value)
  }

  if kind == parser.EXPR_STR {
    return verifier.is_str(value)
  }

  if kind == parser.EXPR_BOOL {
    return verifier.is_bool(value)
  }

  return false
}

func verifier.expr_is_valid(expression, function_count) {
  VTime opcode = ir.expr_opcode(expression)

  if opcode == ir.EXPR_LITERAL {
    if len(expression) != 3 {
      return false
    }

    return verifier.literal_is_valid(get(expression, 1), get(expression, 2))
  }

  if opcode == ir.EXPR_LOAD {
    if len(expression) != 3 {
      return false
    }

    if get(expression, 1) != parser.EXPR_VAR {
      return false
    }

    return verifier.is_str(get(expression, 2))
  }

  if opcode == ir.EXPR_SELF {
    if len(expression) != 3 {
      return false
    }

    if get(expression, 1) != parser.EXPR_SELF {
      return false
    }

    return verifier.is_str(get(expression, 2))
  }

  if opcode == ir.EXPR_NONE {
    if len(expression) != 3 {
      return false
    }

    if get(expression, 1) != parser.EXPR_NONE {
      return false
    }

    return get(expression, 2) == NONE
  }

  if opcode == ir.EXPR_INPUT {
    if len(expression) != 3 {
      return false
    }

    if get(expression, 1) != parser.EXPR_INPUT {
      return false
    }

    return get(expression, 2) == NONE
  }

  if opcode == ir.EXPR_SECRET_INPUT {
    if len(expression) != 3 {
      return false
    }

    if get(expression, 1) != parser.EXPR_SECRET_INPUT {
      return false
    }

    return get(expression, 2) == NONE
  }

  if opcode == ir.EXPR_UNARY {
    if len(expression) != 3 {
      return false
    }

    if verifier.unary_op_is_valid(get(expression, 1)) != true {
      return false
    }

    return verifier.expr_is_valid(get(expression, 2), function_count)
  }

  if opcode == ir.EXPR_BINARY {
    if len(expression) != 4 {
      return false
    }

    if parser.is_binary_op(get(expression, 1)) != true {
      return false
    }

    if verifier.expr_is_valid(get(expression, 2), function_count) != true {
      return false
    }

    return verifier.expr_is_valid(get(expression, 3), function_count)
  }

  if opcode == ir.EXPR_CALL {
    if len(expression) != 3 {
      return false
    }

    VTime expected = verifier.builtin_arity(get(expression, 1))

    if expected < 0 {
      return false
    }

    if len(get(expression, 2)) != expected {
      return false
    }

    return verifier.exprs_are_valid(get(expression, 2), function_count)
  }

  if opcode == ir.EXPR_CALL_SLOT {
    if len(expression) != 3 {
      return false
    }

    VTime function_index = get(expression, 1)

    if verifier.is_int(function_index) != true {
      return false
    }

    if function_index < 0 {
      return false
    }

    if function_index >= function_count {
      return false
    }

    return verifier.exprs_are_valid(get(expression, 2), function_count)
  }

  if opcode == ir.EXPR_LIST {
    if len(expression) != 2 {
      return false
    }

    return verifier.exprs_are_valid(get(expression, 1), function_count)
  }

  if opcode == ir.EXPR_INDEX {
    if len(expression) != 3 {
      return false
    }

    if verifier.expr_is_valid(get(expression, 1), function_count) != true {
      return false
    }

    return verifier.expr_is_valid(get(expression, 2), function_count)
  }

  if opcode == ir.EXPR_SLICE {
    if len(expression) != 5 {
      return false
    }

    if verifier.expr_is_valid(get(expression, 1), function_count) != true {
      return false
    }

    if verifier.expr_is_valid(get(expression, 2), function_count) != true {
      return false
    }

    if verifier.expr_is_valid(get(expression, 3), function_count) != true {
      return false
    }

    return verifier.expr_is_valid(get(expression, 4), function_count)
  }

  if opcode == ir.EXPR_TAG {
    if len(expression) != 3 {
      return false
    }

    if verifier.expr_is_valid(get(expression, 1), function_count) != true {
      return false
    }

    return verifier.tag_is_valid(get(expression, 2))
  }

  return false
}

func verifier.instruction_is_valid(instruction, function_count) {
  VTime opcode = ir.opcode(instruction)

  if opcode == ir.OP_DECL {
    if len(instruction) != 4 {
      return false
    }

    if parser.is_decl_keyword(get(instruction, 1)) != true {
      return false
    }

    if verifier.is_str(get(instruction, 2)) != true {
      return false
    }

    return verifier.expr_is_valid(get(instruction, 3), function_count)
  }

  if opcode == ir.OP_LIST_DECL {
    if len(instruction) != 3 {
      return false
    }

    if verifier.is_str(get(instruction, 1)) != true {
      return false
    }

    return verifier.expr_is_valid(get(instruction, 2), function_count)
  }

  if opcode == ir.OP_VTIME_DECL {
    if len(instruction) != 3 {
      return false
    }

    if verifier.is_str(get(instruction, 1)) != true {
      return false
    }

    return verifier.expr_is_valid(get(instruction, 2), function_count)
  }

  if opcode == ir.OP_INFO_ASSIGN {
    if len(instruction) != 4 {
      return false
    }

    if verifier.is_str(get(instruction, 1)) != true {
      return false
    }

    if verifier.is_str(get(instruction, 2)) != true {
      return false
    }

    return verifier.is_str(get(instruction, 3))
  }

  if opcode == ir.OP_ASSIGN {
    if len(instruction) != 4 {
      return false
    }

    if verifier.is_str(get(instruction, 1)) != true {
      return false
    }

    if parser.is_assign_op(get(instruction, 2)) != true {
      return false
    }

    return verifier.expr_is_valid(get(instruction, 3), function_count)
  }

  if opcode == ir.OP_EXPR {
    if len(instruction) != 2 {
      return false
    }

    return verifier.expr_is_valid(get(instruction, 1), function_count)
  }

  if opcode == ir.OP_OUT {
    if len(instruction) != 2 {
      return false
    }

    return verifier.expr_is_valid(get(instruction, 1), function_count)
  }

  if opcode == ir.OP_STOP {
    if len(instruction) != 2 {
      return false
    }

    return verifier.expr_is_valid(get(instruction, 1), function_count)
  }

  if opcode == ir.OP_FAIL {
    if len(instruction) != 2 {
      return false
    }

    return verifier.expr_is_valid(get(instruction, 1), function_count)
  }

  if opcode == ir.OP_RETURN {
    if len(instruction) != 2 {
      return false
    }

    return verifier.expr_is_valid(get(instruction, 1), function_count)
  }

  if opcode == ir.OP_SECRETUP {
    if len(instruction) != 2 {
      return false
    }

    return verifier.is_str(get(instruction, 1))
  }

  if opcode == ir.OP_BREAK {
    return len(instruction) == 1
  }

  if opcode == ir.OP_CONTINUE {
    return len(instruction) == 1
  }

  if opcode == ir.OP_IF {
    if len(instruction) != 4 {
      return false
    }

    if verifier.expr_is_valid(get(instruction, 1), function_count) != true {
      return false
    }

    if verifier.program_is_valid(get(instruction, 2), function_count) != true {
      return false
    }

    return verifier.program_is_valid(get(instruction, 3), function_count)
  }

  if opcode == ir.OP_WHILE {
    if len(instruction) != 4 {
      return false
    }

    if verifier.expr_is_valid(get(instruction, 1), function_count) != true {
      return false
    }

    VTime limit = get(instruction, 2)

    if verifier.is_int(limit) != true {
      return false
    }

    if limit < -1 {
      return false
    }

    return verifier.program_is_valid(get(instruction, 3), function_count)
  }

  if opcode == ir.OP_PICK {
    if len(instruction) != 4 {
      return false
    }

    if verifier.expr_is_valid(get(instruction, 1), function_count) != true {
      return false
    }

    if verifier.is_str(get(instruction, 2)) != true {
      return false
    }

    return verifier.program_is_valid(get(instruction, 3), function_count)
  }

  return false
}

func verifier.program_is_valid(program, function_count) {
  pick(program): instruction {
    if verifier.instruction_is_valid(instruction, function_count) != true {
      return false
    }
  }

  return true
}

func verifier.exprs_contain_call_slot(expressions) {
  pick(expressions): expression {
    if verifier.expr_contains_call_slot(expression) {
      return true
    }
  }

  return false
}

func verifier.expr_contains_call_slot(expression) {
  VTime opcode = ir.expr_opcode(expression)

  if opcode == ir.EXPR_CALL_SLOT {
    return true
  }

  if opcode == ir.EXPR_UNARY {
    return verifier.expr_contains_call_slot(get(expression, 2))
  }

  if opcode == ir.EXPR_BINARY {
    if verifier.expr_contains_call_slot(get(expression, 2)) {
      return true
    }

    return verifier.expr_contains_call_slot(get(expression, 3))
  }

  if opcode == ir.EXPR_CALL {
    return verifier.exprs_contain_call_slot(get(expression, 2))
  }

  if opcode == ir.EXPR_LIST {
    return verifier.exprs_contain_call_slot(get(expression, 1))
  }

  if opcode == ir.EXPR_INDEX {
    if verifier.expr_contains_call_slot(get(expression, 1)) {
      return true
    }

    return verifier.expr_contains_call_slot(get(expression, 2))
  }

  if opcode == ir.EXPR_SLICE {
    if verifier.expr_contains_call_slot(get(expression, 1)) {
      return true
    }

    if verifier.expr_contains_call_slot(get(expression, 2)) {
      return true
    }

    if verifier.expr_contains_call_slot(get(expression, 3)) {
      return true
    }

    return verifier.expr_contains_call_slot(get(expression, 4))
  }

  if opcode == ir.EXPR_TAG {
    return verifier.expr_contains_call_slot(get(expression, 1))
  }

  return false
}

func verifier.instruction_contains_call_slot(instruction) {
  VTime opcode = ir.opcode(instruction)

  if opcode == ir.OP_DECL {
    return verifier.expr_contains_call_slot(get(instruction, 3))
  }

  if opcode == ir.OP_LIST_DECL {
    return verifier.expr_contains_call_slot(get(instruction, 2))
  }

  if opcode == ir.OP_VTIME_DECL {
    return verifier.expr_contains_call_slot(get(instruction, 2))
  }

  if opcode == ir.OP_ASSIGN {
    return verifier.expr_contains_call_slot(get(instruction, 3))
  }

  if opcode == ir.OP_EXPR {
    return verifier.expr_contains_call_slot(get(instruction, 1))
  }

  if opcode == ir.OP_OUT {
    return verifier.expr_contains_call_slot(get(instruction, 1))
  }

  if opcode == ir.OP_STOP {
    return verifier.expr_contains_call_slot(get(instruction, 1))
  }

  if opcode == ir.OP_FAIL {
    return verifier.expr_contains_call_slot(get(instruction, 1))
  }

  if opcode == ir.OP_RETURN {
    return verifier.expr_contains_call_slot(get(instruction, 1))
  }

  if opcode == ir.OP_IF {
    if verifier.expr_contains_call_slot(get(instruction, 1)) {
      return true
    }

    if verifier.program_contains_call_slot(get(instruction, 2)) {
      return true
    }

    return verifier.program_contains_call_slot(get(instruction, 3))
  }

  if opcode == ir.OP_WHILE {
    if verifier.expr_contains_call_slot(get(instruction, 1)) {
      return true
    }

    return verifier.program_contains_call_slot(get(instruction, 3))
  }

  if opcode == ir.OP_PICK {
    if verifier.expr_contains_call_slot(get(instruction, 1)) {
      return true
    }

    return verifier.program_contains_call_slot(get(instruction, 3))
  }

  if opcode == ir.OP_FUNC {
    return verifier.program_contains_call_slot(get(instruction, 3))
  }

  return false
}

func verifier.program_contains_call_slot(program) {
  pick(program): instruction {
    if verifier.instruction_contains_call_slot(instruction) {
      return true
    }
  }

  return false
}

func verifier.str_list_is_valid(values) {
  pick(values): value {
    if verifier.is_str(value) != true {
      return false
    }
  }

  return true
}

func verifier.values_are_unique(values) {
  VTime index = 0

  while (index < len(values)) (-1) {
    VTime previous = 0

    while (previous < index) (-1) {
      if get(values, previous) == get(values, index) {
        return false
      }

      previous += 1
    }

    index += 1
  }

  return true
}

func verifier.params_are_valid(params_list) {
  pick(params_list): params {
    if verifier.str_list_is_valid(params) != true {
      return false
    }

    if verifier.values_are_unique(params) != true {
      return false
    }
  }

  return true
}

func verifier.functions_are_valid(functions) {
  if len(functions) != 4 {
    return false
  }

  VTime names = get(functions, 0)
  VTime params_list = get(functions, 1)
  VTime bodies = get(functions, 2)
  VTime valid = get(functions, 3)

  if valid != true {
    return false
  }

  if verifier.str_list_is_valid(names) != true {
    return false
  }

  if verifier.values_are_unique(names) != true {
    return false
  }

  if len(params_list) != len(names) {
    return false
  }

  if len(bodies) != len(names) {
    return false
  }

  if verifier.params_are_valid(params_list) != true {
    return false
  }

  pick(bodies): body {
    if verifier.program_is_valid(body, len(names)) != true {
      return false
    }
  }

  return true
}

func verifier.loaded_image_is_valid(loaded) {
  if len(loaded) != 3 {
    return false
  }

  if get(loaded, 0) != linker.IMAGE_MAGIC {
    return false
  }

  VTime functions = get(loaded, 2)

  if verifier.functions_are_valid(functions) != true {
    return false
  }

  return verifier.program_is_valid(get(loaded, 1), len(get(functions, 0)))
}
