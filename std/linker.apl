# APL bootstrap linker.
# It resolves user-function names to stable numeric slots before VM execution.

AVStr linker.IMAGE_MAGIC = "APLLOAD2"

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

func linker.link_functions(functions) {
  VTime names = get(functions, 0)
  VTime bodies = []

  pick(get(functions, 2)): body {
    add(bodies, linker.link_program(body, names))
  }

  return [names, get(functions, 1), bodies, get(functions, 3)]
}

func linker.link_image(entry, functions) {
  VTime names = get(functions, 0)
  return [linker.IMAGE_MAGIC, linker.link_program(entry, names), linker.link_functions(functions)]
}
