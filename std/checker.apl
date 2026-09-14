# APL checker bootstrap helpers.
# This module validates semantic rules that the bootstrap runtime can enforce
# before lowering AST into IR.

AVStr checker.STATUS_OK = "OK"
AVStr checker.STATUS_FAIL = "FAIL"

func checker.name_exists(names, name) {
  pick(names): existing {
    if existing == name {
      return true
    }
  }

  return false
}

func checker.declared_name(node) {
  VTime kind = parser.node_kind(node)

  if kind == parser.NODE_DECL {
    return get(node, 2)
  }

  if kind == parser.NODE_LIST_DECL {
    return get(node, 1)
  }

  if kind == parser.NODE_FUNC {
    return get(node, 1)
  }

  return NONE
}

func checker.fail(names, message) {
  return [checker.STATUS_FAIL, names, message]
}

func checker.ok(names) {
  return [checker.STATUS_OK, names, NONE]
}

func checker.validate_decl_name(node, names) {
  VTime name = checker.declared_name(node)

  if name == NONE {
    return checker.ok(names)
  }

  if checker.name_exists(names, name) {
    return checker.fail(names, join(["duplicate name `", name, "`"], ""))
  }

  add(names, name)
  return checker.ok(names)
}

func checker.validate_statement(node, names) {
  VTime kind = parser.node_kind(node)

  if kind == parser.NODE_ERROR {
    return checker.fail(names, get(node, 1))
  }

  VTime name_state = checker.validate_decl_name(node, names)
  names = get(name_state, 1)

  if get(name_state, 0) != checker.STATUS_OK {
    return name_state
  }

  if kind == parser.NODE_FUNC {
    return checker.validate_block(get(node, 3), names)
  }

  if kind == parser.NODE_IF {
    VTime body_state = checker.validate_block(get(node, 2), names)
    names = get(body_state, 1)

    if get(body_state, 0) != checker.STATUS_OK {
      return body_state
    }

    return checker.validate_block(get(node, 3), names)
  }

  if kind == parser.NODE_WHILE {
    return checker.validate_block(get(node, 3), names)
  }

  if kind == parser.NODE_PICK {
    return checker.validate_block(get(node, 3), names)
  }

  return checker.ok(names)
}

func checker.validate_block(statements, names) {
  pick(statements): statement {
    VTime state = checker.validate_statement(statement, names)
    names = get(state, 1)

    if get(state, 0) != checker.STATUS_OK {
      return state
    }
  }

  return checker.ok(names)
}

func checker.validate_report(statements) {
  VTime state = checker.validate_block(statements, [])

  if get(state, 0) != checker.STATUS_OK {
    return [get(state, 0), get(state, 2)]
  }

  return [checker.STATUS_OK, statements]
}
