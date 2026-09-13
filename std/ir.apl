# APL IR bootstrap helpers.
# This module lowers the small AST shape from std/parser.apl into a simple
# list-based bootstrap IR. It is a stepping stone toward an APL-owned compiler.

AVStr ir.OP_DECL = "DECL"
AVStr ir.OP_LIST_DECL = "LIST_DECL"
AVStr ir.OP_VTIME_DECL = "VTIME_DECL"
AVStr ir.OP_INFO_ASSIGN = "INFO_ASSIGN"
AVStr ir.OP_ASSIGN = "ASSIGN"
AVStr ir.OP_EXPR = "EXPR"
AVStr ir.OP_OUT = "OUT"
AVStr ir.OP_STOP = "STOP"
AVStr ir.OP_FAIL = "FAIL"
AVStr ir.OP_IF = "IF"
AVStr ir.OP_WHILE = "WHILE"
AVStr ir.OP_PICK = "PICK"
AVStr ir.OP_BREAK = "BREAK"
AVStr ir.OP_CONTINUE = "CONTINUE"
AVStr ir.OP_FUNC = "FUNC"
AVStr ir.OP_RETURN = "RETURN"
AVStr ir.OP_SECRETUP = "SECRETUP"
AVStr ir.OP_ERROR = "ERROR"

AVStr ir.EXPR_LITERAL = "LITERAL"
AVStr ir.EXPR_LOAD = "LOAD"
AVStr ir.EXPR_NONE = "NONE"
AVStr ir.EXPR_INPUT = "INPUT"
AVStr ir.EXPR_SECRET_INPUT = "SECRET_INPUT"
AVStr ir.EXPR_UNARY = "UNARY"
AVStr ir.EXPR_BINARY = "BINARY"
AVStr ir.EXPR_CALL = "CALL"
AVStr ir.EXPR_LIST = "LIST"
AVStr ir.EXPR_INDEX = "INDEX"
AVStr ir.EXPR_SLICE = "SLICE"
AVStr ir.EXPR_SELF = "SELF"
AVStr ir.EXPR_TAG = "TAG"

func ir.opcode(instruction) {
  return get(instruction, 0)
}

func ir.expr_opcode(expression) {
  return get(expression, 0)
}

func ir.expr_value(expression) {
  return get(expression, 2)
}

func ir.compile_expr(expression) {
  VTime kind = parser.expr_kind(expression)
  VTime value = parser.expr_value(expression)

  if kind == parser.EXPR_VAR {
    return [ir.EXPR_LOAD, kind, value]
  }

  if kind == parser.EXPR_NONE {
    return [ir.EXPR_NONE, kind, NONE]
  }

  if kind == parser.EXPR_INPUT {
    return [ir.EXPR_INPUT, kind, NONE]
  }

  if kind == parser.EXPR_SECRET_INPUT {
    return [ir.EXPR_SECRET_INPUT, kind, NONE]
  }

  if kind == parser.EXPR_UNARY {
    return [ir.EXPR_UNARY, get(expression, 1), ir.compile_expr(get(expression, 2))]
  }

  if kind == parser.EXPR_BINARY {
    return [ir.EXPR_BINARY, get(expression, 1), ir.compile_expr(get(expression, 2)), ir.compile_expr(get(expression, 3))]
  }

  if kind == parser.EXPR_CALL {
    VTime args = []

    pick(get(expression, 2)): arg {
      add(args, ir.compile_expr(arg))
    }

    return [ir.EXPR_CALL, get(expression, 1), args]
  }

  if kind == parser.EXPR_LIST {
    VTime items = []

    pick(get(expression, 1)): item {
      add(items, ir.compile_expr(item))
    }

    return [ir.EXPR_LIST, items]
  }

  if kind == parser.EXPR_INDEX {
    return [ir.EXPR_INDEX, ir.compile_expr(get(expression, 1)), ir.compile_expr(get(expression, 2))]
  }

  if kind == parser.EXPR_SLICE {
    return [ir.EXPR_SLICE, ir.compile_expr(get(expression, 1)), ir.compile_expr(get(expression, 2)), ir.compile_expr(get(expression, 3)), ir.compile_expr(get(expression, 4))]
  }

  if kind == parser.EXPR_SELF {
    return [ir.EXPR_SELF, kind, value]
  }

  if kind == parser.EXPR_TAG {
    return [ir.EXPR_TAG, ir.compile_expr(get(expression, 1)), get(expression, 2)]
  }

  return [ir.EXPR_LITERAL, kind, value]
}

func ir.compile_optional_expr(expression) {
  if expression == NONE {
    return [ir.EXPR_NONE, parser.EXPR_NONE, NONE]
  }

  return ir.compile_expr(expression)
}

func ir.compile_node(node) {
  VTime kind = parser.node_kind(node)

  if kind == parser.NODE_DECL {
    return [ir.OP_DECL, get(node, 1), get(node, 2), ir.compile_expr(get(node, 3))]
  }

  if kind == parser.NODE_LIST_DECL {
    return [ir.OP_LIST_DECL, get(node, 1), ir.compile_expr(get(node, 2))]
  }

  if kind == parser.NODE_VTIME_DECL {
    return [ir.OP_VTIME_DECL, get(node, 1), ir.compile_expr(get(node, 2))]
  }

  if kind == parser.NODE_INFO_ASSIGN {
    return [ir.OP_INFO_ASSIGN, get(node, 1), get(node, 2), get(node, 3)]
  }

  if kind == parser.NODE_ASSIGN {
    return [ir.OP_ASSIGN, get(node, 1), get(node, 2), ir.compile_expr(get(node, 3))]
  }

  if kind == parser.NODE_EXPR {
    return [ir.OP_EXPR, ir.compile_expr(get(node, 1))]
  }

  if kind == parser.NODE_OUT {
    return [ir.OP_OUT, ir.compile_expr(get(node, 1))]
  }

  if kind == parser.NODE_STOP {
    return [ir.OP_STOP, ir.compile_optional_expr(get(node, 1))]
  }

  if kind == parser.NODE_FAIL {
    return [ir.OP_FAIL, ir.compile_expr(get(node, 1))]
  }

  if kind == parser.NODE_BREAK {
    return [ir.OP_BREAK]
  }

  if kind == parser.NODE_CONTINUE {
    return [ir.OP_CONTINUE]
  }

  if kind == parser.NODE_FUNC {
    return [ir.OP_FUNC, get(node, 1), get(node, 2), ir.compile_ast(get(node, 3))]
  }

  if kind == parser.NODE_RETURN {
    return [ir.OP_RETURN, ir.compile_expr(get(node, 1))]
  }

  if kind == parser.NODE_SECRETUP {
    return [ir.OP_SECRETUP, get(node, 1)]
  }

  if kind == parser.NODE_IF {
    return [ir.OP_IF, ir.compile_expr(get(node, 1)), ir.compile_ast(get(node, 2)), ir.compile_ast(get(node, 3))]
  }

  if kind == parser.NODE_WHILE {
    return [ir.OP_WHILE, ir.compile_expr(get(node, 1)), get(node, 2), ir.compile_ast(get(node, 3))]
  }

  if kind == parser.NODE_PICK {
    return [ir.OP_PICK, ir.compile_expr(get(node, 1)), get(node, 2), ir.compile_ast(get(node, 3))]
  }

  if kind == parser.NODE_ERROR {
    return [ir.OP_ERROR, get(node, 1)]
  }

  return [ir.OP_ERROR, "unknown AST node"]
}

func ir.compile_ast(statements) {
  VTime instructions = []

  pick(statements): statement {
    VTime instruction = ir.compile_node(statement)
    add(instructions, instruction)

    if ir.opcode(instruction) == ir.OP_ERROR {
      return instructions
    }
  }

  return instructions
}

func ir.compile_source(source) {
  return ir.compile_ast(parser.parse_source(source))
}
