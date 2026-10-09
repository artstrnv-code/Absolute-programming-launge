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
    return len(get(node, 2))
  }

  return NONE
}

func checker.fail(names, message) {
  return [checker.STATUS_FAIL, names, message]
}

func checker.ok(names) {
  return [checker.STATUS_OK, names, NONE]
}

func checker.merge_global_names(names, scoped_names) {
  VTime merged = names[:]
  VTime index = len(names)

  while (index < len(scoped_names)) (-1) {
    VTime entry = get(scoped_names, index)

    if checker.entry_role(entry) != "VTime" {
      add(merged, entry)
    }

    index += 1
  }

  return merged
}

func checker.validate_decl_name(node, names, allow_predeclared_func) {
  VTime name = checker.declared_name(node)
  VTime kind = parser.node_kind(node)

  if name == NONE {
    return checker.ok(names)
  }

  if kind == parser.NODE_FUNC {
    VTime existing = checker.find_name(names, name)

    if existing != NONE {
      if (allow_predeclared_func == true) and (checker.entry_role(existing) == "Func") {
        return checker.ok(names)
      }
    }
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

func checker.builtin_arity(name) {
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

func checker.is_builtin_call(name) {
  return checker.builtin_arity(name) >= 0
}

func checker.decl_value_type(typ) {
  if contains(typ, "Float") {
    return "Float"
  }

  if contains(typ, "Bool") {
    return "Bool"
  }

  if contains(typ, "Bytes") {
    return "Bytes"
  }

  if contains(typ, "Json") {
    return "Json"
  }

  if contains(typ, "Int") {
    return "Int"
  }

  if contains(typ, "Str") {
    return "Str"
  }

  return "VTime"
}

func checker.entry_value_type(entry) {
  VTime role = checker.entry_role(entry)

  if role == "Absolute" {
    return checker.decl_value_type(checker.entry_type(entry))
  }

  if role == "List" {
    return "List"
  }

  return "VTime"
}

func checker.call_result_type(name, args, names) {
  if name == "int" {
    return "Int"
  }

  if name == "float" {
    return "Float"
  }

  if name == "bool" {
    return "Bool"
  }

  if name == "str" {
    return "Str"
  }

  if name == "bytes" {
    return "Bytes"
  }

  if name == "json" {
    return "Json"
  }

  if name == "split" {
    return "List"
  }

  if name == "len" {
    return "Int"
  }

  if name == "contains" {
    return "Bool"
  }

  if name == "join" {
    return "Str"
  }

  if name == "ord" {
    return "Int"
  }

  if name == "char" {
    return "Str"
  }

  if name == "pow" {
    VTime base_type = checker.expr_type(get(args, 0), names)

    if base_type == "Int" {
      return "Int"
    }

    if base_type == "Float" {
      return "Float"
    }
  }

  return "VTime"
}

func checker.expr_type(expression, names) {
  VTime kind = parser.expr_kind(expression)

  if kind == parser.EXPR_INT {
    return "Int"
  }

  if kind == parser.EXPR_FLOAT {
    return "Float"
  }

  if kind == parser.EXPR_STR {
    return "Str"
  }

  if kind == parser.EXPR_BOOL {
    return "Bool"
  }

  if kind == parser.EXPR_NONE {
    return "None"
  }

  if kind == parser.EXPR_INPUT {
    return "Input"
  }

  if kind == parser.EXPR_SECRET_INPUT {
    return "SecretInput"
  }

  if kind == parser.EXPR_VAR {
    VTime entry = checker.find_name(names, parser.expr_value(expression))

    if entry == NONE {
      return "VTime"
    }

    return checker.entry_value_type(entry)
  }

  if kind == parser.EXPR_SELF {
    return "Bool"
  }

  if kind == parser.EXPR_TAG {
    return checker.expr_type(get(expression, 1), names)
  }

  if kind == parser.EXPR_LIST {
    return "List"
  }

  if kind == parser.EXPR_INDEX {
    return "VTime"
  }

  if kind == parser.EXPR_SLICE {
    VTime target_type = checker.expr_type(get(expression, 1), names)

    if target_type == "Str" {
      return "Str"
    }

    if target_type == "Bytes" {
      return "Bytes"
    }

    if target_type == "List" {
      return "List"
    }

    return "VTime"
  }

  if kind == parser.EXPR_UNARY {
    if get(expression, 1) == "not" {
      return "Bool"
    }

    return checker.expr_type(get(expression, 2), names)
  }

  if kind == parser.EXPR_BINARY {
    VTime op = get(expression, 1)

    if op == "==" {
      return "Bool"
    }

    if op == "!=" {
      return "Bool"
    }

    if op == ">" {
      return "Bool"
    }

    if op == "<" {
      return "Bool"
    }

    if op == ">=" {
      return "Bool"
    }

    if op == "<=" {
      return "Bool"
    }

    if op == "and" {
      return "Bool"
    }

    if op == "or" {
      return "Bool"
    }

    VTime left_type = checker.expr_type(get(expression, 2), names)
    VTime right_type = checker.expr_type(get(expression, 3), names)

    if left_type == right_type {
      if left_type == "Int" {
        return "Int"
      }

      if left_type == "Float" {
        return "Float"
      }
    }

    return "VTime"
  }

  if kind == parser.EXPR_CALL {
    return checker.call_result_type(get(expression, 1), get(expression, 2), names)
  }

  return "VTime"
}

func checker.type_allows(actual, expected) {
  if actual == "VTime" {
    return true
  }

  return actual == expected
}

func checker.type_is_pickable(actual) {
  if actual == "VTime" {
    return true
  }

  if actual == "Str" {
    return true
  }

  if actual == "Bytes" {
    return true
  }

  return actual == "List"
}

func checker.type_is_numeric(actual) {
  if actual == "VTime" {
    return true
  }

  if actual == "Int" {
    return true
  }

  return actual == "Float"
}

func checker.validate_builtin_argument_types(expression, names) {
  VTime name = get(expression, 1)

  if checker.is_builtin_call(name) != true {
    return checker.ok(names)
  }

  if name == "int" {
    return checker.ok(names)
  }

  if name == "float" {
    return checker.ok(names)
  }

  if name == "bool" {
    return checker.ok(names)
  }

  if name == "str" {
    return checker.ok(names)
  }

  if name == "bytes" {
    return checker.ok(names)
  }

  if name == "json" {
    return checker.ok(names)
  }

  VTime args = get(expression, 2)
  VTime first_type = checker.expr_type(get(args, 0), names)

  if name == "len" {
    if checker.type_is_pickable(first_type) {
      return checker.ok(names)
    }

    return checker.fail(names, "len(value) requires Str, Bytes, List, or VTime")
  }

  if name == "ord" {
    if checker.type_allows(first_type, "Str") {
      return checker.ok(names)
    }

    return checker.fail(names, "ord(value) requires Str")
  }

  if name == "char" {
    if checker.type_allows(first_type, "Int") {
      return checker.ok(names)
    }

    return checker.fail(names, "char(value) requires Int")
  }

  if name == "pop" {
    if checker.type_allows(first_type, "List") {
      return checker.ok(names)
    }

    return checker.fail(names, "pop(list) requires List")
  }

  if name == "add" {
    if checker.type_allows(first_type, "List") {
      return checker.ok(names)
    }

    return checker.fail(names, "add(list, value) requires List")
  }

  VTime second_type = checker.expr_type(get(args, 1), names)

  if name == "get" {
    if (checker.type_allows(first_type, "List")) and (checker.type_allows(second_type, "Int")) {
      return checker.ok(names)
    }

    return checker.fail(names, "get(list, index) requires List and Int")
  }

  if name == "split" {
    if (checker.type_allows(first_type, "Str")) and (checker.type_allows(second_type, "Str")) {
      return checker.ok(names)
    }

    return checker.fail(names, "split(value, separator) requires Str and Str")
  }

  if name == "join" {
    if (checker.type_allows(first_type, "List")) and (checker.type_allows(second_type, "Str")) {
      return checker.ok(names)
    }

    return checker.fail(names, "join(values, separator) requires List and Str")
  }

  if name == "contains" {
    if (checker.type_allows(first_type, "Str")) and (checker.type_allows(second_type, "Str")) {
      return checker.ok(names)
    }

    return checker.fail(names, "contains(value, needle) requires Str and Str")
  }

  if name == "pow" {
    if (checker.type_is_numeric(first_type)) and (checker.type_allows(second_type, "Int")) {
      return checker.ok(names)
    }

    return checker.fail(names, "pow(base, exponent) requires Int/Float and Int")
  }

  return checker.ok(names)
}

func checker.validate_call_target(expression, names) {
  VTime name = get(expression, 1)
  VTime args = get(expression, 2)
  VTime builtin_arity = checker.builtin_arity(name)

  if builtin_arity >= 0 {
    if len(args) != builtin_arity {
      return checker.fail(names, join(["wrong argument count `", name, "`"], ""))
    }

    return checker.validate_builtin_argument_types(expression, names)
  }

  VTime entry = checker.find_name(names, name)

  if entry == NONE {
    return checker.fail(names, join(["unknown function `", name, "`"], ""))
  }

  if checker.entry_role(entry) != "Func" {
    return checker.fail(names, join(["invalid function `", name, "`"], ""))
  }

  if checker.entry_type(entry) != len(args) {
    return checker.fail(names, join(["wrong argument count `", name, "`"], ""))
  }

  return checker.ok(names)
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

func checker.validate_expr(expression, names) {
  VTime kind = parser.expr_kind(expression)

  if kind == parser.EXPR_VAR {
    VTime name = parser.expr_value(expression)

    if checker.name_exists(names, name) {
      return checker.ok(names)
    }

    return checker.fail(names, join(["unknown variable `", name, "`"], ""))
  }

  if kind == parser.EXPR_SELF {
    VTime name = parser.expr_value(expression)
    VTime entry = checker.find_name(names, name)

    if entry == NONE {
      return checker.fail(names, join(["unknown variable `", name, "`"], ""))
    }

    if checker.entry_role(entry) != "Absolute" {
      return checker.fail(names, join(["invalid self target `", name, "`"], ""))
    }

    return checker.ok(names)
  }

  if kind == parser.EXPR_UNARY {
    return checker.validate_expr(get(expression, 2), names)
  }

  if kind == parser.EXPR_BINARY {
    VTime left_state = checker.validate_expr(get(expression, 2), names)

    if get(left_state, 0) != checker.STATUS_OK {
      return left_state
    }

    return checker.validate_expr(get(expression, 3), names)
  }

  if kind == parser.EXPR_CALL {
    VTime call_state = checker.validate_call_target(expression, names)

    if get(call_state, 0) != checker.STATUS_OK {
      return call_state
    }

    pick(get(expression, 2)): arg {
      VTime arg_state = checker.validate_expr(arg, names)

      if get(arg_state, 0) != checker.STATUS_OK {
        return arg_state
      }
    }

    return checker.ok(names)
  }

  if kind == parser.EXPR_LIST {
    pick(get(expression, 1)): item {
      VTime item_state = checker.validate_expr(item, names)

      if get(item_state, 0) != checker.STATUS_OK {
        return item_state
      }
    }

    return checker.ok(names)
  }

  if kind == parser.EXPR_INDEX {
    VTime target_state = checker.validate_expr(get(expression, 1), names)

    if get(target_state, 0) != checker.STATUS_OK {
      return target_state
    }

    return checker.validate_expr(get(expression, 2), names)
  }

  if kind == parser.EXPR_SLICE {
    VTime target_state = checker.validate_expr(get(expression, 1), names)

    if get(target_state, 0) != checker.STATUS_OK {
      return target_state
    }

    VTime start = get(expression, 2)
    VTime end = get(expression, 3)
    VTime step = get(expression, 4)

    if start != NONE {
      VTime start_state = checker.validate_expr(start, names)

      if get(start_state, 0) != checker.STATUS_OK {
        return start_state
      }
    }

    if end != NONE {
      VTime end_state = checker.validate_expr(end, names)

      if get(end_state, 0) != checker.STATUS_OK {
        return end_state
      }
    }

    if step != NONE {
      VTime step_state = checker.validate_expr(step, names)

      if get(step_state, 0) != checker.STATUS_OK {
        return step_state
      }
    }

    return checker.ok(names)
  }

  if kind == parser.EXPR_TAG {
    return checker.validate_expr(get(expression, 1), names)
  }

  return checker.ok(names)
}

func checker.validate_optional_expr(expression, names) {
  if expression == NONE {
    return checker.ok(names)
  }

  return checker.validate_expr(expression, names)
}

func checker.validate_statement(node, names, allow_predeclared_func) {
  VTime kind = parser.node_kind(node)

  if kind == parser.NODE_ERROR {
    return checker.fail(names, get(node, 1))
  }

  if kind == parser.NODE_DECL {
    VTime init_state = checker.validate_expr(get(node, 3), names)

    if get(init_state, 0) != checker.STATUS_OK {
      return init_state
    }
  }

  if kind == parser.NODE_LIST_DECL {
    VTime list_state = checker.validate_expr(get(node, 2), names)

    if get(list_state, 0) != checker.STATUS_OK {
      return list_state
    }
  }

  if kind == parser.NODE_VTIME_DECL {
    VTime vtime_state = checker.validate_expr(get(node, 2), names)

    if get(vtime_state, 0) != checker.STATUS_OK {
      return vtime_state
    }
  }

  VTime name_state = checker.validate_decl_name(node, names, allow_predeclared_func)
  names = get(name_state, 1)

  if get(name_state, 0) != checker.STATUS_OK {
    return name_state
  }

  if kind == parser.NODE_ASSIGN {
    VTime target_state = checker.validate_assignment_target(node, names)

    if get(target_state, 0) != checker.STATUS_OK {
      return target_state
    }

    return checker.validate_expr(get(node, 3), names)
  }

  if kind == parser.NODE_SECRETUP {
    return checker.validate_secretup_target(node, names)
  }

  if kind == parser.NODE_INFO_ASSIGN {
    return checker.validate_info_assignment(node, names)
  }

  if kind == parser.NODE_EXPR {
    return checker.validate_expr(get(node, 1), names)
  }

  if kind == parser.NODE_OUT {
    return checker.validate_expr(get(node, 1), names)
  }

  if kind == parser.NODE_STOP {
    return checker.validate_optional_expr(get(node, 1), names)
  }

  if kind == parser.NODE_FAIL {
    return checker.validate_expr(get(node, 1), names)
  }

  if kind == parser.NODE_RETURN {
    return checker.validate_expr(get(node, 1), names)
  }

  if kind == parser.NODE_FUNC {
    VTime local_names = names[:]

    pick(get(node, 2)): param {
      local_names = checker.add_name(local_names, param, "VTime", "VTime")
    }

    VTime body_state = checker.validate_block(get(node, 3), local_names, false)

    if get(body_state, 0) != checker.STATUS_OK {
      return body_state
    }

    return checker.ok(checker.merge_global_names(names, get(body_state, 1)))
  }

  if kind == parser.NODE_IF {
    VTime condition_state = checker.validate_expr(get(node, 1), names)

    if get(condition_state, 0) != checker.STATUS_OK {
      return condition_state
    }

    VTime body_state = checker.validate_block(get(node, 2), names[:], false)

    if get(body_state, 0) != checker.STATUS_OK {
      return body_state
    }

    names = checker.merge_global_names(names, get(body_state, 1))
    VTime else_state = checker.validate_block(get(node, 3), names[:], false)

    if get(else_state, 0) != checker.STATUS_OK {
      return else_state
    }

    return checker.ok(checker.merge_global_names(names, get(else_state, 1)))
  }

  if kind == parser.NODE_WHILE {
    VTime condition_state = checker.validate_expr(get(node, 1), names)

    if get(condition_state, 0) != checker.STATUS_OK {
      return condition_state
    }

    VTime body_state = checker.validate_block(get(node, 3), names[:], false)

    if get(body_state, 0) != checker.STATUS_OK {
      return body_state
    }

    return checker.ok(checker.merge_global_names(names, get(body_state, 1)))
  }

  if kind == parser.NODE_PICK {
    VTime value_state = checker.validate_expr(get(node, 1), names)

    if get(value_state, 0) != checker.STATUS_OK {
      return value_state
    }

    VTime local_names = checker.add_name(names[:], get(node, 2), "VTime", "VTime")
    VTime body_state = checker.validate_block(get(node, 3), local_names, false)

    if get(body_state, 0) != checker.STATUS_OK {
      return body_state
    }

    return checker.ok(checker.merge_global_names(names, get(body_state, 1)))
  }

  return checker.ok(names)
}

func checker.validate_block(statements, names, allow_predeclared_func) {
  pick(statements): statement {
    VTime state = checker.validate_statement(statement, names, allow_predeclared_func)
    names = get(state, 1)

    if get(state, 0) != checker.STATUS_OK {
      return state
    }
  }

  return checker.ok(names)
}

func checker.collect_function_names(statements, names) {
  pick(statements): statement {
    if parser.node_kind(statement) == parser.NODE_FUNC {
      VTime name = get(statement, 1)

      if checker.name_exists(names, name) {
        return checker.fail(names, join(["duplicate name `", name, "`"], ""))
      }

      names = checker.add_name(names, name, "Func", len(get(statement, 2)))
    }
  }

  return checker.ok(names)
}

func checker.add_external_symbols(names, symbols) {
  pick(symbols): symbol {
    VTime name = get(symbol, 0)

    if checker.name_exists(names, name) {
      return checker.fail(names, join(["duplicate name `", name, "`"], ""))
    }

    names = checker.add_name(names, name, get(symbol, 1), get(symbol, 2))
  }

  return checker.ok(names)
}

func checker.validate_report_with_symbols(statements, symbols) {
  VTime external_state = checker.add_external_symbols([], symbols)

  if get(external_state, 0) != checker.STATUS_OK {
    return [get(external_state, 0), get(external_state, 2)]
  }

  VTime collect_state = checker.collect_function_names(statements, get(external_state, 1))

  if get(collect_state, 0) != checker.STATUS_OK {
    return [get(collect_state, 0), get(collect_state, 2)]
  }

  VTime state = checker.validate_block(statements, get(collect_state, 1), true)

  if get(state, 0) != checker.STATUS_OK {
    return [get(state, 0), get(state, 2)]
  }

  return [checker.STATUS_OK, statements]
}

func checker.validate_report_with_functions(statements, function_names, arities) {
  if len(function_names) != len(arities) {
    return [checker.STATUS_FAIL, "external function table shape mismatch"]
  }

  VTime symbols = []
  VTime index = 0

  while (index < len(function_names)) (-1) {
    add(symbols, [get(function_names, index), "Func", get(arities, index)])
    index += 1
  }

  return checker.validate_report_with_symbols(statements, symbols)
}

func checker.validate_report(statements) {
  return checker.validate_report_with_symbols(statements, [])
}
