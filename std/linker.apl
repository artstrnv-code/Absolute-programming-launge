# APL bootstrap linker.
# It retains functions reachable from entry code, then resolves their names to
# stable numeric slots before VM execution.

AVStr linker.IMAGE_MAGIC = "APLLOAD2"
AVStr linker.MANIFEST_MAGIC = "APLMAN1"

func linker.name_index(names, name) {
  VTime index = len(names) - 1

  while (index >= 0) (-1) {
    if get(names, index) == name {
      return index
    }

    index -= 1
  }

  return NONE
}

func linker.link_exprs(expressions, function_names) {
  VTime linked = []

  pick(expressions): expression {
    add(linked, linker.link_expr(expression, function_names))
  }

  return linked
}

func linker.link_expr(expression, function_names) {
  VTime opcode = ir.expr_opcode(expression)

  if opcode == ir.EXPR_UNARY {
    return [opcode, get(expression, 1), linker.link_expr(get(expression, 2), function_names)]
  }

  if opcode == ir.EXPR_BINARY {
    return [opcode, get(expression, 1), linker.link_expr(get(expression, 2), function_names), linker.link_expr(get(expression, 3), function_names)]
  }

  if opcode == ir.EXPR_CALL {
    VTime function_index = linker.name_index(function_names, get(expression, 1))
    VTime args = linker.link_exprs(get(expression, 2), function_names)

    if function_index != NONE {
      return [ir.EXPR_CALL_SLOT, function_index, args]
    }

    return [opcode, get(expression, 1), args]
  }

  if opcode == ir.EXPR_CALL_SLOT {
    return [opcode, get(expression, 1), linker.link_exprs(get(expression, 2), function_names)]
  }

  if opcode == ir.EXPR_LIST {
    return [opcode, linker.link_exprs(get(expression, 1), function_names)]
  }

  if opcode == ir.EXPR_INDEX {
    return [opcode, linker.link_expr(get(expression, 1), function_names), linker.link_expr(get(expression, 2), function_names)]
  }

  if opcode == ir.EXPR_SLICE {
    return [opcode, linker.link_expr(get(expression, 1), function_names), linker.link_expr(get(expression, 2), function_names), linker.link_expr(get(expression, 3), function_names), linker.link_expr(get(expression, 4), function_names)]
  }

  if opcode == ir.EXPR_TAG {
    return [opcode, linker.link_expr(get(expression, 1), function_names), get(expression, 2)]
  }

  return expression
}

func linker.link_instruction(instruction, function_names) {
  VTime opcode = ir.opcode(instruction)

  if opcode == ir.OP_DECL {
    return [opcode, get(instruction, 1), get(instruction, 2), linker.link_expr(get(instruction, 3), function_names)]
  }

  if opcode == ir.OP_LIST_DECL {
    return [opcode, get(instruction, 1), linker.link_expr(get(instruction, 2), function_names)]
  }

  if opcode == ir.OP_VTIME_DECL {
    return [opcode, get(instruction, 1), linker.link_expr(get(instruction, 2), function_names)]
  }

  if opcode == ir.OP_ASSIGN {
    return [opcode, get(instruction, 1), get(instruction, 2), linker.link_expr(get(instruction, 3), function_names)]
  }

  if opcode == ir.OP_EXPR {
    return [opcode, linker.link_expr(get(instruction, 1), function_names)]
  }

  if opcode == ir.OP_OUT {
    return [opcode, linker.link_expr(get(instruction, 1), function_names)]
  }

  if opcode == ir.OP_STOP {
    return [opcode, linker.link_expr(get(instruction, 1), function_names)]
  }

  if opcode == ir.OP_FAIL {
    return [opcode, linker.link_expr(get(instruction, 1), function_names)]
  }

  if opcode == ir.OP_RETURN {
    return [opcode, linker.link_expr(get(instruction, 1), function_names)]
  }

  if opcode == ir.OP_IF {
    return [opcode, linker.link_expr(get(instruction, 1), function_names), linker.link_program(get(instruction, 2), function_names), linker.link_program(get(instruction, 3), function_names)]
  }

  if opcode == ir.OP_WHILE {
    return [opcode, linker.link_expr(get(instruction, 1), function_names), get(instruction, 2), linker.link_program(get(instruction, 3), function_names)]
  }

  if opcode == ir.OP_PICK {
    return [opcode, linker.link_expr(get(instruction, 1), function_names), get(instruction, 2), linker.link_program(get(instruction, 3), function_names)]
  }

  if opcode == ir.OP_FUNC {
    return [opcode, get(instruction, 1), get(instruction, 2), linker.link_program(get(instruction, 3), function_names)]
  }

  return instruction
}

func linker.link_program(program, function_names) {
  VTime linked = []

  pick(program): instruction {
    add(linked, linker.link_instruction(instruction, function_names))
  }

  return linked
}

func linker.collect_exprs_calls(expressions, function_names, reachable) {
  pick(expressions): expression {
    reachable = linker.collect_expr_calls(expression, function_names, reachable)
  }

  return reachable
}

func linker.collect_expr_calls(expression, function_names, reachable) {
  VTime opcode = ir.expr_opcode(expression)

  if opcode == ir.EXPR_UNARY {
    return linker.collect_expr_calls(get(expression, 2), function_names, reachable)
  }

  if opcode == ir.EXPR_BINARY {
    reachable = linker.collect_expr_calls(get(expression, 2), function_names, reachable)
    return linker.collect_expr_calls(get(expression, 3), function_names, reachable)
  }

  if opcode == ir.EXPR_CALL {
    VTime name = get(expression, 1)

    if linker.name_index(function_names, name) != NONE {
      if linker.name_index(reachable, name) == NONE {
        add(reachable, name)
      }
    }

    return linker.collect_exprs_calls(get(expression, 2), function_names, reachable)
  }

  if opcode == ir.EXPR_CALL_SLOT {
    return linker.collect_exprs_calls(get(expression, 2), function_names, reachable)
  }

  if opcode == ir.EXPR_LIST {
    return linker.collect_exprs_calls(get(expression, 1), function_names, reachable)
  }

  if opcode == ir.EXPR_INDEX {
    reachable = linker.collect_expr_calls(get(expression, 1), function_names, reachable)
    return linker.collect_expr_calls(get(expression, 2), function_names, reachable)
  }

  if opcode == ir.EXPR_SLICE {
    reachable = linker.collect_expr_calls(get(expression, 1), function_names, reachable)
    reachable = linker.collect_expr_calls(get(expression, 2), function_names, reachable)
    reachable = linker.collect_expr_calls(get(expression, 3), function_names, reachable)
    return linker.collect_expr_calls(get(expression, 4), function_names, reachable)
  }

  if opcode == ir.EXPR_TAG {
    return linker.collect_expr_calls(get(expression, 1), function_names, reachable)
  }

  return reachable
}

func linker.collect_instruction_calls(instruction, function_names, reachable) {
  VTime opcode = ir.opcode(instruction)

  if opcode == ir.OP_DECL {
    return linker.collect_expr_calls(get(instruction, 3), function_names, reachable)
  }

  if opcode == ir.OP_LIST_DECL {
    return linker.collect_expr_calls(get(instruction, 2), function_names, reachable)
  }

  if opcode == ir.OP_VTIME_DECL {
    return linker.collect_expr_calls(get(instruction, 2), function_names, reachable)
  }

  if opcode == ir.OP_ASSIGN {
    return linker.collect_expr_calls(get(instruction, 3), function_names, reachable)
  }

  if opcode == ir.OP_EXPR {
    return linker.collect_expr_calls(get(instruction, 1), function_names, reachable)
  }

  if opcode == ir.OP_OUT {
    return linker.collect_expr_calls(get(instruction, 1), function_names, reachable)
  }

  if opcode == ir.OP_STOP {
    return linker.collect_expr_calls(get(instruction, 1), function_names, reachable)
  }

  if opcode == ir.OP_FAIL {
    return linker.collect_expr_calls(get(instruction, 1), function_names, reachable)
  }

  if opcode == ir.OP_RETURN {
    return linker.collect_expr_calls(get(instruction, 1), function_names, reachable)
  }

  if opcode == ir.OP_IF {
    reachable = linker.collect_expr_calls(get(instruction, 1), function_names, reachable)
    reachable = linker.collect_program_calls(get(instruction, 2), function_names, reachable)
    return linker.collect_program_calls(get(instruction, 3), function_names, reachable)
  }

  if opcode == ir.OP_WHILE {
    reachable = linker.collect_expr_calls(get(instruction, 1), function_names, reachable)
    return linker.collect_program_calls(get(instruction, 3), function_names, reachable)
  }

  if opcode == ir.OP_PICK {
    reachable = linker.collect_expr_calls(get(instruction, 1), function_names, reachable)
    return linker.collect_program_calls(get(instruction, 3), function_names, reachable)
  }

  if opcode == ir.OP_FUNC {
    return linker.collect_program_calls(get(instruction, 3), function_names, reachable)
  }

  return reachable
}

func linker.collect_program_calls(program, function_names, reachable) {
  pick(program): instruction {
    reachable = linker.collect_instruction_calls(instruction, function_names, reachable)
  }

  return reachable
}

func linker.reachable_function_names(entry, functions) {
  VTime function_names = get(functions, 0)
  VTime bodies = get(functions, 2)
  VTime reachable = linker.collect_program_calls(entry, function_names, [])
  VTime index = 0

  while (index < len(reachable)) (-1) {
    VTime function_index = linker.name_index(function_names, get(reachable, index))

    if function_index != NONE {
      reachable = linker.collect_program_calls(get(bodies, function_index), function_names, reachable)
    }

    index += 1
  }

  return reachable
}

func linker.retain_functions(functions, reachable) {
  VTime source_names = get(functions, 0)
  VTime source_params = get(functions, 1)
  VTime source_bodies = get(functions, 2)
  VTime names = []
  VTime params = []
  VTime bodies = []
  VTime index = 0

  while (index < len(source_names)) (-1) {
    if linker.name_index(reachable, get(source_names, index)) != NONE {
      add(names, get(source_names, index))
      add(params, get(source_params, index))
      add(bodies, get(source_bodies, index))
    }

    index += 1
  }

  return [names, params, bodies, get(functions, 3)]
}

func linker.link_functions(functions) {
  VTime names = get(functions, 0)
  VTime bodies = []

  pick(get(functions, 2)): body {
    add(bodies, linker.link_program(body, names))
  }

  return [names, get(functions, 1), bodies, get(functions, 3)]
}

func linker.link_complete_image(entry, functions) {
  VTime names = get(functions, 0)
  return [linker.IMAGE_MAGIC, linker.link_program(entry, names), linker.link_functions(functions)]
}

func linker.link_image(entry, functions) {
  VTime reachable = linker.reachable_function_names(entry, functions)
  VTime retained = linker.retain_functions(functions, reachable)
  VTime names = get(retained, 0)
  return [linker.IMAGE_MAGIC, linker.link_program(entry, names), linker.link_functions(retained)]
}
