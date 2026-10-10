# Portable linked-artifact codec written in APL.
# Scalars are tagged and length-prefixed, so strings need no escaping.

AVStr artifact.HEADER = "APLLINK2:"
AVStr artifact.MODULE_HEADER = "APLMOD3:"
AVStr artifact.MODULE_HEADER_V2 = "APLMOD2:"
AVStr artifact.MODULE_HEADER_V1 = "APLMOD1:"
AVStr artifact.NODE_NONE = "n"
AVStr artifact.NODE_BOOL = "b"
AVStr artifact.NODE_INT = "i"
AVStr artifact.NODE_FLOAT = "f"
AVStr artifact.NODE_STR = "s"
AVStr artifact.NODE_LIST = "l"

func artifact.node_none() {
  return [artifact.NODE_NONE]
}

func artifact.node_bool(value) {
  return [artifact.NODE_BOOL, value]
}

func artifact.node_int(value) {
  return [artifact.NODE_INT, value]
}

func artifact.node_float(value) {
  return [artifact.NODE_FLOAT, value]
}

func artifact.node_str(value) {
  return [artifact.NODE_STR, value]
}

func artifact.node_list(items) {
  return [artifact.NODE_LIST, items]
}

func artifact.encode_sized(tag, value) {
  VTime text = str(value)
  return join([tag, str(len(text)), ":", text], "")
}

func artifact.encode_node(node) {
  VTime tag = get(node, 0)

  if tag == artifact.NODE_NONE {
    return artifact.NODE_NONE
  }

  if tag == artifact.NODE_BOOL {
    if get(node, 1) == true {
      return "b1"
    }

    return "b0"
  }

  if tag == artifact.NODE_INT {
    return artifact.encode_sized(tag, get(node, 1))
  }

  if tag == artifact.NODE_FLOAT {
    return artifact.encode_sized(tag, get(node, 1))
  }

  if tag == artifact.NODE_STR {
    return artifact.encode_sized(tag, get(node, 1))
  }

  if tag == artifact.NODE_LIST {
    VTime items = get(node, 1)
    VTime parts = [tag, str(len(items)), ":"]

    pick(items): item {
      add(parts, artifact.encode_node(item))
    }

    return join(parts, "")
  }

  return ""
}

func artifact.wrap_str_list(values) {
  VTime items = []

  pick(values): value {
    add(items, artifact.node_str(value))
  }

  return artifact.node_list(items)
}

func artifact.wrap_str_lists(values) {
  VTime items = []

  pick(values): value {
    add(items, artifact.wrap_str_list(value))
  }

  return artifact.node_list(items)
}

func artifact.wrap_int_list(values) {
  VTime items = []

  pick(values): value {
    add(items, artifact.node_int(value))
  }

  return artifact.node_list(items)
}

func artifact.wrap_exprs(expressions) {
  VTime items = []

  pick(expressions): expression {
    add(items, artifact.wrap_expr(expression))
  }

  return artifact.node_list(items)
}

func artifact.wrap_literal(kind, value) {
  if kind == parser.EXPR_INT {
    return artifact.node_int(value)
  }

  if kind == parser.EXPR_FLOAT {
    return artifact.node_float(value)
  }

  if kind == parser.EXPR_BOOL {
    return artifact.node_bool(value)
  }

  if kind == parser.EXPR_NONE {
    return artifact.node_none()
  }

  return artifact.node_str(value)
}

func artifact.wrap_expr(expression) {
  VTime opcode = ir.expr_opcode(expression)
  VTime items = [artifact.node_str(opcode)]

  if opcode == ir.EXPR_LITERAL {
    add(items, artifact.node_str(get(expression, 1)))
    add(items, artifact.wrap_literal(get(expression, 1), get(expression, 2)))
    return artifact.node_list(items)
  }

  if opcode == ir.EXPR_LOAD {
    add(items, artifact.node_str(get(expression, 1)))
    add(items, artifact.node_str(get(expression, 2)))
    return artifact.node_list(items)
  }

  if opcode == ir.EXPR_SELF {
    add(items, artifact.node_str(get(expression, 1)))
    add(items, artifact.node_str(get(expression, 2)))
    return artifact.node_list(items)
  }

  if opcode == ir.EXPR_NONE {
    add(items, artifact.node_str(get(expression, 1)))
    add(items, artifact.node_none())
    return artifact.node_list(items)
  }

  if opcode == ir.EXPR_INPUT {
    add(items, artifact.node_str(get(expression, 1)))
    add(items, artifact.node_none())
    return artifact.node_list(items)
  }

  if opcode == ir.EXPR_SECRET_INPUT {
    add(items, artifact.node_str(get(expression, 1)))
    add(items, artifact.node_none())
    return artifact.node_list(items)
  }

  if opcode == ir.EXPR_UNARY {
    add(items, artifact.node_str(get(expression, 1)))
    add(items, artifact.wrap_expr(get(expression, 2)))
    return artifact.node_list(items)
  }

  if opcode == ir.EXPR_BINARY {
    add(items, artifact.node_str(get(expression, 1)))
    add(items, artifact.wrap_expr(get(expression, 2)))
    add(items, artifact.wrap_expr(get(expression, 3)))
    return artifact.node_list(items)
  }

  if opcode == ir.EXPR_CALL {
    add(items, artifact.node_str(get(expression, 1)))
    add(items, artifact.wrap_exprs(get(expression, 2)))
    return artifact.node_list(items)
  }

  if opcode == ir.EXPR_CALL_SLOT {
    add(items, artifact.node_int(get(expression, 1)))
    add(items, artifact.wrap_exprs(get(expression, 2)))
    return artifact.node_list(items)
  }

  if opcode == ir.EXPR_LIST {
    add(items, artifact.wrap_exprs(get(expression, 1)))
    return artifact.node_list(items)
  }

  if opcode == ir.EXPR_INDEX {
    add(items, artifact.wrap_expr(get(expression, 1)))
    add(items, artifact.wrap_expr(get(expression, 2)))
    return artifact.node_list(items)
  }

  if opcode == ir.EXPR_SLICE {
    add(items, artifact.wrap_expr(get(expression, 1)))
    add(items, artifact.wrap_expr(get(expression, 2)))
    add(items, artifact.wrap_expr(get(expression, 3)))
    add(items, artifact.wrap_expr(get(expression, 4)))
    return artifact.node_list(items)
  }

  if opcode == ir.EXPR_TAG {
    add(items, artifact.wrap_expr(get(expression, 1)))
    add(items, artifact.node_str(get(expression, 2)))
    return artifact.node_list(items)
  }

  return artifact.node_list(items)
}

func artifact.wrap_instruction(instruction) {
  VTime opcode = ir.opcode(instruction)
  VTime items = [artifact.node_str(opcode)]

  if opcode == ir.OP_DECL {
    add(items, artifact.node_str(get(instruction, 1)))
    add(items, artifact.node_str(get(instruction, 2)))
    add(items, artifact.wrap_expr(get(instruction, 3)))
    return artifact.node_list(items)
  }

  if opcode == ir.OP_LIST_DECL {
    add(items, artifact.node_str(get(instruction, 1)))
    add(items, artifact.wrap_expr(get(instruction, 2)))
    return artifact.node_list(items)
  }

  if opcode == ir.OP_VTIME_DECL {
    add(items, artifact.node_str(get(instruction, 1)))
    add(items, artifact.wrap_expr(get(instruction, 2)))
    return artifact.node_list(items)
  }

  if opcode == ir.OP_INFO_ASSIGN {
    add(items, artifact.node_str(get(instruction, 1)))
    add(items, artifact.node_str(get(instruction, 2)))
    add(items, artifact.node_str(get(instruction, 3)))
    return artifact.node_list(items)
  }

  if opcode == ir.OP_ASSIGN {
    add(items, artifact.node_str(get(instruction, 1)))
    add(items, artifact.node_str(get(instruction, 2)))
    add(items, artifact.wrap_expr(get(instruction, 3)))
    return artifact.node_list(items)
  }

  if opcode == ir.OP_EXPR {
    add(items, artifact.wrap_expr(get(instruction, 1)))
    return artifact.node_list(items)
  }

  if opcode == ir.OP_OUT {
    add(items, artifact.wrap_expr(get(instruction, 1)))
    return artifact.node_list(items)
  }

  if opcode == ir.OP_STOP {
    add(items, artifact.wrap_expr(get(instruction, 1)))
    return artifact.node_list(items)
  }

  if opcode == ir.OP_FAIL {
    add(items, artifact.wrap_expr(get(instruction, 1)))
    return artifact.node_list(items)
  }

  if opcode == ir.OP_RETURN {
    add(items, artifact.wrap_expr(get(instruction, 1)))
    return artifact.node_list(items)
  }

  if opcode == ir.OP_SECRETUP {
    add(items, artifact.node_str(get(instruction, 1)))
    return artifact.node_list(items)
  }

  if opcode == ir.OP_IF {
    add(items, artifact.wrap_expr(get(instruction, 1)))
    add(items, artifact.wrap_program(get(instruction, 2)))
    add(items, artifact.wrap_program(get(instruction, 3)))
    return artifact.node_list(items)
  }

  if opcode == ir.OP_WHILE {
    add(items, artifact.wrap_expr(get(instruction, 1)))
    add(items, artifact.node_int(get(instruction, 2)))
    add(items, artifact.wrap_program(get(instruction, 3)))
    return artifact.node_list(items)
  }

  if opcode == ir.OP_PICK {
    add(items, artifact.wrap_expr(get(instruction, 1)))
    add(items, artifact.node_str(get(instruction, 2)))
    add(items, artifact.wrap_program(get(instruction, 3)))
    return artifact.node_list(items)
  }

  if opcode == ir.OP_FUNC {
    add(items, artifact.node_str(get(instruction, 1)))
    add(items, artifact.wrap_str_list(get(instruction, 2)))
    add(items, artifact.wrap_program(get(instruction, 3)))
    return artifact.node_list(items)
  }

  if opcode == ir.OP_ERROR {
    add(items, artifact.node_str(get(instruction, 1)))
    return artifact.node_list(items)
  }

  return artifact.node_list(items)
}

func artifact.wrap_program(program) {
  VTime items = []

  pick(program): instruction {
    add(items, artifact.wrap_instruction(instruction))
  }

  return artifact.node_list(items)
}

func artifact.wrap_programs(programs) {
  VTime items = []

  pick(programs): program {
    add(items, artifact.wrap_program(program))
  }

  return artifact.node_list(items)
}

func artifact.wrap_functions(functions) {
  return artifact.node_list([
    artifact.wrap_str_list(get(functions, 0)),
    artifact.wrap_str_lists(get(functions, 1)),
    artifact.wrap_programs(get(functions, 2)),
    artifact.node_bool(get(functions, 3))
  ])
}

func artifact.wrap_loaded(loaded) {
  return artifact.node_list([
    artifact.node_str(get(loaded, 0)),
    artifact.wrap_program(get(loaded, 1)),
    artifact.wrap_functions(get(loaded, 2))
  ])
}

func artifact.decode_ok(value, index) {
  return [true, value, index, ""]
}

func artifact.decode_fail(index, message) {
  return [false, NONE, index, message]
}

func artifact.read_count(text, index) {
  VTime start = index

  while (index < len(text)) (-1) {
    VTime ch = text[index]

    if ch == ":" {
      if index == start {
        return artifact.decode_fail(index, "missing size")
      }

      VTime value = int(text[start:index])

      if value == NONE {
        return artifact.decode_fail(index, "invalid size")
      }

      return artifact.decode_ok(value, index + 1)
    }

    if lexer.is_digit(ch) != true {
      return artifact.decode_fail(index, "invalid size")
    }

    index += 1
  }

  return artifact.decode_fail(index, "missing size separator")
}

func artifact.read_payload(text, index) {
  VTime size_state = artifact.read_count(text, index)

  if get(size_state, 0) != true {
    return size_state
  }

  VTime size = get(size_state, 1)
  VTime start = get(size_state, 2)

  if size > len(text) - start {
    return artifact.decode_fail(start, "truncated payload")
  }

  VTime end = start + size
  return artifact.decode_ok(text[start:end], end)
}

func artifact.decode_node(text, index) {
  if index >= len(text) {
    return artifact.decode_fail(index, "missing node")
  }

  VTime tag = text[index]

  if tag == artifact.NODE_NONE {
    return artifact.decode_ok(NONE, index + 1)
  }

  if tag == artifact.NODE_BOOL {
    if index + 1 >= len(text) {
      return artifact.decode_fail(index, "truncated bool")
    }

    VTime bit = text[index + 1]

    if bit == "1" {
      return artifact.decode_ok(true, index + 2)
    }

    if bit == "0" {
      return artifact.decode_ok(false, index + 2)
    }

    return artifact.decode_fail(index, "invalid bool")
  }

  if tag == artifact.NODE_STR {
    return artifact.read_payload(text, index + 1)
  }

  if tag == artifact.NODE_INT {
    VTime payload = artifact.read_payload(text, index + 1)

    if get(payload, 0) != true {
      return payload
    }

    VTime value = int(get(payload, 1))

    if value == NONE {
      return artifact.decode_fail(index, "invalid int")
    }

    return artifact.decode_ok(value, get(payload, 2))
  }

  if tag == artifact.NODE_FLOAT {
    VTime payload = artifact.read_payload(text, index + 1)

    if get(payload, 0) != true {
      return payload
    }

    VTime value = float(get(payload, 1))

    if value == NONE {
      return artifact.decode_fail(index, "invalid float")
    }

    return artifact.decode_ok(value, get(payload, 2))
  }

  if tag == artifact.NODE_LIST {
    VTime count_state = artifact.read_count(text, index + 1)

    if get(count_state, 0) != true {
      return count_state
    }

    VTime count = get(count_state, 1)
    VTime cursor = get(count_state, 2)
    VTime items = []
    VTime item_index = 0

    while (item_index < count) (-1) {
      VTime item_state = artifact.decode_node(text, cursor)

      if get(item_state, 0) != true {
        return item_state
      }

      add(items, get(item_state, 1))
      cursor = get(item_state, 2)
      item_index += 1
    }

    return artifact.decode_ok(items, cursor)
  }

  return artifact.decode_fail(index, "unknown node tag")
}

func artifact.encode_loaded_report(loaded) {
  if vm.loaded_image_is_valid(loaded) != true {
    return [vm.FLOW_FAIL, "invalid loaded image"]
  }

  VTime payload = artifact.encode_node(artifact.wrap_loaded(loaded))
  return [vm.FLOW_OK, join([artifact.HEADER, payload], "")]
}

func artifact.encode_loaded(loaded) {
  return get(artifact.encode_loaded_report(loaded), 1)
}

func artifact.decode_loaded_report(encoded) {
  VTime header_size = len(artifact.HEADER)

  if len(encoded) < header_size {
    return [vm.FLOW_FAIL, "invalid linked artifact header"]
  }

  if encoded[:header_size] != artifact.HEADER {
    return [vm.FLOW_FAIL, "invalid linked artifact header"]
  }

  VTime decoded = artifact.decode_node(encoded, header_size)

  if get(decoded, 0) != true {
    return [vm.FLOW_FAIL, get(decoded, 3)]
  }

  if get(decoded, 2) != len(encoded) {
    return [vm.FLOW_FAIL, "trailing linked artifact data"]
  }

  VTime loaded = get(decoded, 1)

  if vm.loaded_image_is_valid(loaded) != true {
    return [vm.FLOW_FAIL, "invalid linked artifact image"]
  }

  return [vm.FLOW_OK, loaded]
}

func artifact.decode_loaded(encoded) {
  return get(artifact.decode_loaded_report(encoded), 1)
}

func artifact.function_arities(functions) {
  VTime arities = []

  pick(vm.func_params(functions)): params {
    add(arities, len(params))
  }

  return arities
}

func artifact.wrap_symbol_table(symbols) {
  VTime wrapped = []

  pick(symbols): symbol {
    VTime typ = get(symbol, 2)
    VTime wrapped_type = artifact.node_str(typ)

    if get(symbol, 1) == "Func" {
      wrapped_type = artifact.node_int(typ)
    }

    VTime fields = [
      artifact.node_str(get(symbol, 0)),
      artifact.node_str(get(symbol, 1)),
      wrapped_type
    ]

    if len(symbol) >= 4 {
      add(fields, artifact.node_str(get(symbol, 3)))
    }

    add(wrapped, artifact.node_list(fields))
  }

  return artifact.node_list(wrapped)
}

func artifact.program_symbols(program) {
  VTime symbols = []

  pick(program): instruction {
    VTime opcode = ir.opcode(instruction)

    if opcode == ir.OP_DECL {
      add(symbols, [get(instruction, 2), "Absolute", get(instruction, 1)])
    }

    if opcode == ir.OP_LIST_DECL {
      add(symbols, [get(instruction, 1), "List", "List"])
    }

    if opcode == ir.OP_FUNC {
      add(symbols, [get(instruction, 1), "Func", len(get(instruction, 2))])
    }
  }

  return symbols
}

func artifact.protection_for_type(typ) {
  if contains(typ, "SASV") {
    return "SASV"
  }

  if contains(typ, "ASV") {
    return "ASV"
  }

  return "AV"
}

func artifact.default_program_symbols(program) {
  VTime symbols = []

  pick(artifact.program_symbols(program)): symbol {
    VTime protection = "AV"

    if get(symbol, 1) == "Absolute" {
      protection = artifact.protection_for_type(get(symbol, 2))
    }

    if get(symbol, 1) == "List" {
      protection = "SASV"
    }

    add(symbols, [get(symbol, 0), get(symbol, 1), get(symbol, 2), protection])
  }

  return symbols
}

func artifact.module_symbols_are_valid(symbols) {
  VTime names = []

  pick(symbols): symbol {
    if len(symbol) != 3 {
      return false
    }

    VTime name = get(symbol, 0)
    VTime role = get(symbol, 1)
    VTime typ = get(symbol, 2)

    if verifier.is_str(name) != true {
      return false
    }

    if checker.name_is_valid(name) != true {
      return false
    }

    if verifier.is_str(role) != true {
      return false
    }

    if role == "Func" {
      if verifier.is_int(typ) != true {
        return false
      }

      if typ < 0 {
        return false
      }
    } else if role == "Absolute" {
      if parser.is_decl_keyword(typ) != true {
        return false
      }
    } else if role == "List" {
      if typ != "List" {
        return false
      }
    } else {
      return false
    }

    add(names, name)
  }

  return verifier.values_are_unique(names)
}

func artifact.protection_is_valid(protection) {
  if protection == "AV" {
    return true
  }

  if protection == "ASV" {
    return true
  }

  return protection == "SASV"
}

func artifact.module_v3_symbols_are_valid(symbols) {
  VTime names = []

  pick(symbols): symbol {
    if len(symbol) != 4 {
      return false
    }

    VTime name = get(symbol, 0)
    VTime role = get(symbol, 1)
    VTime typ = get(symbol, 2)
    VTime protection = get(symbol, 3)

    if verifier.is_str(name) != true {
      return false
    }

    if checker.name_is_valid(name) != true {
      return false
    }

    if verifier.is_str(role) != true {
      return false
    }

    if artifact.protection_is_valid(protection) != true {
      return false
    }

    if role == "Func" {
      if verifier.is_int(typ) != true {
        return false
      }

      if typ < 0 {
        return false
      }

      if protection != "AV" {
        return false
      }
    } else if role == "Absolute" {
      if parser.is_decl_keyword(typ) != true {
        return false
      }

      if checker.protection_rank(protection) < checker.protection_rank(artifact.protection_for_type(typ)) {
        return false
      }
    } else if role == "List" {
      if typ != "List" {
        return false
      }
    } else {
      return false
    }

    add(names, name)
  }

  return verifier.values_are_unique(names)
}

func artifact.module_v3_symbols_match_program(program, symbols) {
  VTime expected = artifact.program_symbols(program)

  if len(expected) != len(symbols) {
    return false
  }

  pick(expected): expected_symbol {
    VTime found = false

    pick(symbols): symbol {
      if get(symbol, 0) == get(expected_symbol, 0) {
        if get(symbol, 1) != get(expected_symbol, 1) {
          return false
        }

        if get(symbol, 2) != get(expected_symbol, 2) {
          return false
        }

        found = true
      }
    }

    if found != true {
      return false
    }
  }

  return true
}

func artifact.module_v1_image_is_valid(image) {
  if len(image) != 4 {
    return false
  }

  if get(image, 0) != "APLMOD1" {
    return false
  }

  VTime program = get(image, 1)
  VTime names = get(image, 2)
  VTime arities = get(image, 3)

  if verifier.str_list_is_valid(names) != true {
    return false
  }

  if verifier.values_are_unique(names) != true {
    return false
  }

  pick(names): name {
    if checker.name_is_valid(name) != true {
      return false
    }
  }

  if len(arities) != len(names) {
    return false
  }

  pick(arities): arity {
    if verifier.is_int(arity) != true {
      return false
    }

    if arity < 0 {
      return false
    }
  }

  if verifier.program_contains_call_slot(program) {
    return false
  }

  VTime loaded_report = vm.load_ir_report(program)

  if get(loaded_report, 0) != vm.FLOW_OK {
    return false
  }

  VTime functions = vm.loaded_functions(get(loaded_report, 1))

  if vm.func_names(functions) != names {
    return false
  }

  return artifact.function_arities(functions) == arities
}

func artifact.module_v1_symbols(image) {
  VTime symbols = []
  VTime names = get(image, 2)
  VTime arities = get(image, 3)
  VTime index = 0

  while (index < len(names)) (-1) {
    add(symbols, [get(names, index), "Func", get(arities, index), "AV"])
    index += 1
  }

  return symbols
}

func artifact.module_image_is_valid(image) {
  if get(image, 0) == "APLMOD1" {
    return artifact.module_v1_image_is_valid(image)
  }

  if len(image) != 3 {
    return false
  }

  if get(image, 0) == "APLMOD2" {
    VTime program = get(image, 1)
    VTime symbols = get(image, 2)

    if artifact.module_symbols_are_valid(symbols) != true {
      return false
    }

    if verifier.program_contains_call_slot(program) {
      return false
    }

    VTime loaded_report = vm.load_ir_report(program)

    if get(loaded_report, 0) != vm.FLOW_OK {
      return false
    }

    return artifact.program_symbols(program) == symbols
  }

  if get(image, 0) != "APLMOD3" {
    return false
  }

  VTime program = get(image, 1)
  VTime symbols = get(image, 2)

  if artifact.module_v3_symbols_are_valid(symbols) != true {
    return false
  }

  if verifier.program_contains_call_slot(program) {
    return false
  }

  VTime loaded_report = vm.load_ir_report(program)

  if get(loaded_report, 0) != vm.FLOW_OK {
    return false
  }

  return artifact.module_v3_symbols_match_program(program, symbols)
}

func artifact.module_program(image) {
  return get(image, 1)
}

func artifact.module_symbols(image) {
  if get(image, 0) == "APLMOD1" {
    return artifact.module_v1_symbols(image)
  }

  if get(image, 0) == "APLMOD2" {
    VTime normalized = []

    pick(get(image, 2)): symbol {
      VTime protection = "AV"

      if get(symbol, 1) == "Absolute" {
        protection = artifact.protection_for_type(get(symbol, 2))
      }

      if get(symbol, 1) == "List" {
        protection = "SASV"
      }

      add(normalized, [get(symbol, 0), get(symbol, 1), get(symbol, 2), protection])
    }

    return normalized
  }

  return get(image, 2)
}

func artifact.encode_module_with_symbols_report(program, symbols) {
  VTime loaded_report = vm.load_ir_report(program)

  if get(loaded_report, 0) != vm.FLOW_OK {
    return [vm.FLOW_FAIL, "invalid module program"]
  }

  if verifier.program_contains_call_slot(program) {
    return [vm.FLOW_FAIL, "module program is already linked"]
  }

  if artifact.module_v3_symbols_are_valid(symbols) != true {
    return [vm.FLOW_FAIL, "invalid module symbol table"]
  }

  if artifact.module_v3_symbols_match_program(program, symbols) != true {
    return [vm.FLOW_FAIL, "module symbol table does not match program"]
  }

  VTime wrapped = artifact.node_list([
    artifact.node_str("APLMOD3"),
    artifact.wrap_program(program),
    artifact.wrap_symbol_table(symbols)
  ])

  return [vm.FLOW_OK, join([artifact.MODULE_HEADER, artifact.encode_node(wrapped)], "")]
}

func artifact.encode_module_report(program) {
  return artifact.encode_module_with_symbols_report(program, artifact.default_program_symbols(program))
}

func artifact.encode_module(program) {
  return get(artifact.encode_module_report(program), 1)
}

func artifact.decode_module_report(encoded) {
  VTime header_size = len(artifact.MODULE_HEADER)

  if len(encoded) < header_size {
    return [vm.FLOW_FAIL, "invalid module artifact header"]
  }

  VTime header = encoded[:header_size]

  if header != artifact.MODULE_HEADER {
    if header != artifact.MODULE_HEADER_V2 {
      if header != artifact.MODULE_HEADER_V1 {
        return [vm.FLOW_FAIL, "invalid module artifact header"]
      }
    }
  }

  VTime decoded = artifact.decode_node(encoded, header_size)

  if get(decoded, 0) != true {
    return [vm.FLOW_FAIL, get(decoded, 3)]
  }

  if get(decoded, 2) != len(encoded) {
    return [vm.FLOW_FAIL, "trailing module artifact data"]
  }

  VTime image = get(decoded, 1)

  if artifact.module_image_is_valid(image) != true {
    return [vm.FLOW_FAIL, "invalid module artifact image"]
  }

  return [vm.FLOW_OK, image]
}

func artifact.decode_module(encoded) {
  return get(artifact.decode_module_report(encoded), 1)
}
