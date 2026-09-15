# APL checker bootstrap helpers.
# This module validates semantic rules that the bootstrap runtime can enforce
# before lowering AST into IR.

AVStr checker.STATUS_OK = "OK"
AVStr checker.STATUS_FAIL = "FAIL"

func checker.entry_name(entry) {
  return get(entry, 0)
}

func checker.entry_role(entry) {
  return get(entry, 1)
}

func checker.entry_type(entry) {
  return get(entry, 2)
}

func checker.name_exists(names, name) {
  pick(names): existing {
    if checker.entry_name(existing) == name {
      return true
    }
  }

  return false
}

func checker.find_name(names, name) {
  pick(names): existing {
    if checker.entry_name(existing) == name {
      return existing
    }
  }

  return NONE
}

func checker.add_name(names, name, role, typ) {
  add(names, [name, role, typ])
  return names
}

func checker.declared_name(node) {
  VTime kind = parser.node_kind(node)

  if kind == parser.NODE_DECL {
    return get(node, 2)
  }

  if kind == parser.NODE_LIST_DECL {
    return get(node, 1)
  }

  if kind == parser.NODE_VTIME_DECL {
    return get(node, 1)
  }

  if kind == parser.NODE_FUNC {
    return get(node, 1)
  }

  return NONE
}

func checker.declared_role(node) {
  VTime kind = parser.node_kind(node)

  if kind == parser.NODE_DECL {
    return "Absolute"
  }

  if kind == parser.NODE_LIST_DECL {
    return "List"
  }

  if kind == parser.NODE_VTIME_DECL {
    return "VTime"
  }

  if kind == parser.NODE_FUNC {
    return "Func"
  }

  return NONE
}

func checker.declared_type(node) {
  VTime kind = parser.node_kind(node)

  if kind == parser.NODE_DECL {
    return get(node, 1)
  }

  if kind == parser.NODE_LIST_DECL {
    return "List"
  }

  if kind == parser.NODE_VTIME_DECL {
    return "VTime"
  }

  if kind == parser.NODE_FUNC {
    return "Func"
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

  names = checker.add_name(names, name, checker.declared_role(node), checker.declared_type(node))
  return checker.ok(names)
}

func checker.require_name(names, name, message) {
  if checker.name_exists(names, name) {
    return checker.ok(names)
  }

  return checker.fail(names, join([message, " `", name, "`"], ""))
}

func checker.type_is_public_numeric(typ) {
  if typ == "AVInt" {
    return true
  }

  if typ == "AVFloat" {
    return true
  }

  return false
}

func checker.validate_assignment_target(node, names) {
  VTime name = get(node, 1)
  VTime op = get(node, 2)
  VTime entry = checker.find_name(names, name)

  if entry == NONE {
    return checker.fail(names, join(["unknown assignment target `", name, "`"], ""))
  }

  if checker.entry_role(entry) == "Func" {
    return checker.fail(names, join(["invalid assignment target `", name, "`"], ""))
  }

  if checker.entry_role(entry) == "List" {
    return checker.fail(names, join(["invalid assignment target `", name, "`"], ""))
  }

  if op == "=" {
    return checker.ok(names)
  }

  if checker.entry_role(entry) == "VTime" {
    return checker.ok(names)
  }

  if checker.type_is_public_numeric(checker.entry_type(entry)) {
    return checker.ok(names)
  }

  return checker.fail(names, join(["invalid compound assignment target `", name, "`"], ""))
}

func checker.validate_secretup_target(node, names) {
  VTime name = get(node, 1)
  VTime entry = checker.find_name(names, name)

  if entry == NONE {
    return checker.fail(names, join(["unknown secretup target `", name, "`"], ""))
  }

  if checker.entry_role(entry) != "Absolute" {
    return checker.fail(names, join(["invalid secretup target `", name, "`"], ""))
  }

  return checker.ok(names)
}

func checker.validate_info_target(names, name) {
  VTime entry = checker.find_name(names, name)

  if entry == NONE {
    return checker.fail(names, join(["unknown info target `", name, "`"], ""))
  }

  if checker.entry_type(entry) != "AVStr" {
    return checker.fail(names, join(["invalid info target `", name, "`"], ""))
  }

  return checker.ok(names)
}

func checker.validate_info_assignment(node, names) {
  VTime type_state = checker.validate_info_target(names, get(node, 1))

  if get(type_state, 0) != checker.STATUS_OK {
    return type_state
  }

  VTime protection_state = checker.validate_info_target(names, get(node, 2))

  if get(protection_state, 0) != checker.STATUS_OK {
    return protection_state
  }

  VTime source = get(node, 3)
  VTime source_entry = checker.find_name(names, source)

  if source_entry == NONE {
    return checker.fail(names, join(["unknown info source `", source, "`"], ""))
  }

  if checker.entry_role(source_entry) != "Absolute" {
    return checker.fail(names, join(["invalid info source `", source, "`"], ""))
  }

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

  if kind == parser.NODE_ASSIGN {
    return checker.validate_assignment_target(node, names)
  }

  if kind == parser.NODE_SECRETUP {
    return checker.validate_secretup_target(node, names)
  }

  if kind == parser.NODE_INFO_ASSIGN {
    return checker.validate_info_assignment(node, names)
  }

  if kind == parser.NODE_FUNC {
    VTime local_names = names[:]

    pick(get(node, 2)): param {
      local_names = checker.add_name(local_names, param, "VTime", "VTime")
    }

    return checker.validate_block(get(node, 3), local_names)
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
    VTime local_names = checker.add_name(names[:], get(node, 2), "VTime", "VTime")
    return checker.validate_block(get(node, 3), local_names)
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
