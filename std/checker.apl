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

func checker.decl_protection(typ) {
  if contains(typ, "SASV") {
    return "SASV"
  }

  if contains(typ, "ASV") {
    return "ASV"
  }

  return "AV"
}

func checker.entry_protection(entry) {
  if len(entry) >= 4 {
    return get(entry, 3)
  }

  if checker.entry_role(entry) == "Absolute" {
    return checker.decl_protection(checker.entry_type(entry))
  }

  return "AV"
}

func checker.protection_rank(protection) {
  if protection == "SASV" {
    return 2
  }

  if protection == "ASV" {
    return 1
  }

  return 0
}

func checker.max_protection(left, right) {
  if checker.protection_rank(left) >= checker.protection_rank(right) {
    return left
  }

  return right
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
  VTime index = len(names) - 1

  while (index >= 0) (-1) {
    VTime existing = get(names, index)

    if checker.entry_name(existing) == name {
      return existing
    }

    index -= 1
  }

  return NONE
}

func checker.find_function(names, name) {
  pick(names): existing {
    if checker.entry_name(existing) == name {
      if checker.entry_role(existing) == "Func" {
        return existing
      }
    }
  }

  return NONE
}

func checker.value_exists(values, value) {
  pick(values): existing {
    if existing == value {
      return true
    }
  }

  return false
}

func checker.is_ascii_letter(ch) {
  VTime code = ord(ch)

  if (code >= 65) and (code <= 90) {
    return true
  }

  return (code >= 97) and (code <= 122)
}

func checker.name_part_is_valid(part) {
  if len(part) == 0 {
    return false
  }

  VTime first = part[0]

  if first != "_" {
    if checker.is_ascii_letter(first) != true {
      return false
    }
  }

  VTime index = 1

  while (index < len(part)) (-1) {
    VTime ch = part[index]

    if ch != "_" {
      if checker.is_ascii_letter(ch) != true {
        if lexer.is_digit(ch) != true {
          return false
        }
      }
    }

    index += 1
  }

  return true
}

func checker.name_is_valid(name) {
  if len(name) == 0 {
    return false
  }

  if lexer.is_keyword(name) {
    return false
  }

  if checker.builtin_arity(name) >= 0 {
    return false
  }

  if name[0] == "." {
    return false
  }

  if name[len(name) - 1] == "." {
    return false
  }

  if contains(name, "..") {
    return false
  }

  pick(split(name, ".")): part {
    if checker.name_part_is_valid(part) != true {
      return false
    }
  }

  return true
}

func checker.validate_name(names, name) {
  if checker.name_is_valid(name) {
    return checker.ok(names)
  }

  return checker.fail(names, join(["invalid name `", name, "`"], ""))
}

func checker.add_name(names, name, role, typ) {
  VTime protection = "AV"

  if role == "Absolute" {
    protection = checker.decl_protection(typ)
  }

  add(names, [name, role, typ, protection])
  return names
}

func checker.add_name_with_protection(names, name, role, typ, protection) {
  add(names, [name, role, typ, protection])
  return names
}

func checker.update_name_protection(names, name, protection) {
  VTime updated = []
  VTime target_index = -1
  VTime index = 0

  pick(names): entry {
    if checker.entry_name(entry) == name {
      target_index = index
    }

    index += 1
  }

  index = 0

  pick(names): entry {
    if index == target_index {
      add(updated, [checker.entry_name(entry), checker.entry_role(entry), checker.entry_type(entry), protection])
    } else {
      add(updated, entry)
    }

    index += 1
  }

  return updated
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

func checker.declared_protection(node, names) {
  VTime kind = parser.node_kind(node)

  if kind == parser.NODE_DECL {
    return checker.decl_protection(get(node, 1))
  }

  if kind == parser.NODE_LIST_DECL {
    return checker.expr_protection(get(node, 2), names)
  }

  if kind == parser.NODE_VTIME_DECL {
    return checker.expr_protection(get(node, 2), names)
  }

  return "AV"
}

func checker.fail(names, message) {
  return [checker.STATUS_FAIL, names, message]
}

func checker.ok(names) {
  return [checker.STATUS_OK, names, NONE]
}

func checker.merge_global_names(names, scoped_names) {
  VTime merged = []
  VTime index = 0

  pick(names): entry {
    if index >= len(scoped_names) {
      add(merged, entry)
    } else {
      add(merged, get(scoped_names, index))
    }

    index += 1
  }

  index = len(names)

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

  VTime valid_state = checker.validate_name(names, name)

  if get(valid_state, 0) != checker.STATUS_OK {
    return valid_state
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

  names = checker.add_name_with_protection(names, name, checker.declared_role(node), checker.declared_type(node), checker.declared_protection(node, names))
  return checker.ok(names)
}

func checker.validate_decl_collision(node, names, allow_predeclared_func) {
  VTime name = checker.declared_name(node)
  VTime kind = parser.node_kind(node)

  if name == NONE {
    return checker.ok(names)
  }

  VTime valid_state = checker.validate_name(names, name)

  if get(valid_state, 0) != checker.STATUS_OK {
    return valid_state
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

func checker.max_expr_protection(expressions, names) {
  VTime protection = "AV"

  pick(expressions): expression {
    protection = checker.max_protection(protection, checker.expr_protection(expression, names))
  }

  return protection
}

func checker.call_result_protection(name, args, names) {
  if name == "add" {
    return "AV"
  }

  if name == "get" {
    return checker.expr_protection(get(args, 0), names)
  }

  if name == "pop" {
    return checker.expr_protection(get(args, 0), names)
  }

  if name == "len" {
    return checker.expr_protection(get(args, 0), names)
  }

  if name == "ord" {
    return checker.expr_protection(get(args, 0), names)
  }

  if name == "char" {
    return checker.expr_protection(get(args, 0), names)
  }

  if name == "int" {
    return checker.expr_protection(get(args, 0), names)
  }

  if name == "float" {
    return checker.expr_protection(get(args, 0), names)
  }

  if name == "bool" {
    return checker.expr_protection(get(args, 0), names)
  }

  if name == "str" {
    return checker.expr_protection(get(args, 0), names)
  }

  if name == "bytes" {
    return checker.expr_protection(get(args, 0), names)
  }

  if name == "json" {
    return checker.expr_protection(get(args, 0), names)
  }

  return checker.max_expr_protection(args, names)
}

func checker.expr_protection(expression, names) {
  VTime kind = parser.expr_kind(expression)

  if kind == parser.EXPR_SECRET_INPUT {
    return "ASV"
  }

  if kind == parser.EXPR_VAR {
    VTime entry = checker.find_name(names, parser.expr_value(expression))

    if entry == NONE {
      return "AV"
    }

    return checker.entry_protection(entry)
  }

  if kind == parser.EXPR_TAG {
    return get(expression, 2)
  }

  if kind == parser.EXPR_LIST {
    return checker.max_expr_protection(get(expression, 1), names)
  }

  if kind == parser.EXPR_INDEX {
    return checker.max_protection(checker.expr_protection(get(expression, 1), names), checker.expr_protection(get(expression, 2), names))
  }

  if kind == parser.EXPR_SLICE {
    VTime protection = checker.expr_protection(get(expression, 1), names)
    VTime start = get(expression, 2)
    VTime end = get(expression, 3)
    VTime step = get(expression, 4)

    if start != NONE {
      protection = checker.max_protection(protection, checker.expr_protection(start, names))
    }

    if end != NONE {
      protection = checker.max_protection(protection, checker.expr_protection(end, names))
    }

    if step != NONE {
      protection = checker.max_protection(protection, checker.expr_protection(step, names))
    }

    return protection
  }

  if kind == parser.EXPR_UNARY {
    return checker.expr_protection(get(expression, 2), names)
  }

  if kind == parser.EXPR_BINARY {
    return checker.max_protection(checker.expr_protection(get(expression, 2), names), checker.expr_protection(get(expression, 3), names))
  }

  if kind == parser.EXPR_CALL {
    return checker.call_result_protection(get(expression, 1), get(expression, 2), names)
  }

  return "AV"
}

func checker.protection_downgrade(names, name, source, target) {
  return checker.fail(names, join(["protection downgrade `", name, "`: source ", source, ", target ", target], ""))
}

func checker.validate_tag_protection(expression, names) {
  VTime source = checker.expr_protection(get(expression, 1), names)
  VTime target = get(expression, 2)

  if checker.protection_rank(source) <= checker.protection_rank(target) {
    return checker.ok(names)
  }

  return checker.protection_downgrade(names, "list element", source, target)
}

func checker.validate_decl_protection(node, names) {
  VTime name = get(node, 2)
  VTime expression = get(node, 3)
  VTime kind = parser.expr_kind(expression)
  VTime target = checker.decl_protection(get(node, 1))

  if kind == parser.EXPR_SECRET_INPUT {
    if target != "ASV" {
      return checker.fail(names, join(["secret expression denied `", name, "`"], ""))
    }
  }

  if target == "SASV" {
    if kind == parser.EXPR_INPUT {
      return checker.fail(names, join(["secret expression denied `", name, "`"], ""))
    }

    if kind == parser.EXPR_SECRET_INPUT {
      return checker.fail(names, join(["secret expression denied `", name, "`"], ""))
    }
  }

  VTime source = checker.expr_protection(expression, names)

  if checker.protection_rank(source) > checker.protection_rank(target) {
    return checker.protection_downgrade(names, name, source, target)
  }

  return checker.ok(names)
}

func checker.validate_assignment_protection(node, names) {
  VTime name = get(node, 1)
  VTime entry = checker.find_name(names, name)
  VTime source = checker.expr_protection(get(node, 3), names)

  if checker.entry_role(entry) == "VTime" {
    return checker.ok(checker.update_name_protection(names, name, source))
  }

  VTime target = checker.entry_protection(entry)

  if checker.protection_rank(source) > checker.protection_rank(target) {
    return checker.protection_downgrade(names, name, source, target)
  }

  return checker.ok(names)
}

func checker.validate_public_expression(expression, names, channel) {
  if expression == NONE {
    return checker.ok(names)
  }

  if checker.expr_protection(expression, names) == "AV" {
    return checker.ok(names)
  }

  return checker.fail(names, join(["secret expression denied `", channel, "`"], ""))
}

func checker.type_allows(actual, expected) {
  if actual == "VTime" {
    return true
  }

  return actual == expected
}

func checker.type_is_dynamic(actual) {
  if actual == "VTime" {
    return true
  }

  if actual == "None" {
    return true
  }

  if actual == "Input" {
    return true
  }

  return actual == "SecretInput"
}

func checker.type_is_assignable(actual, expected) {
  if checker.type_is_dynamic(actual) {
    return true
  }

  return actual == expected
}

func checker.type_mismatch(names, name, expected, actual) {
  return checker.fail(names, join(["type mismatch `", name, "`: expected ", expected, ", got ", actual], ""))
}

func checker.validate_decl_type(node, names) {
  VTime name = get(node, 2)
  VTime expected = checker.decl_value_type(get(node, 1))
  VTime actual = checker.expr_type(get(node, 3), names)

  if checker.type_is_assignable(actual, expected) {
    return checker.ok(names)
  }

  return checker.type_mismatch(names, name, expected, actual)
}

func checker.validate_assignment_type(node, names) {
  VTime name = get(node, 1)
  VTime entry = checker.find_name(names, name)

  if checker.entry_role(entry) == "VTime" {
    return checker.ok(names)
  }

  VTime expected = checker.entry_value_type(entry)
  VTime actual = checker.expr_type(get(node, 3), names)

  if checker.type_is_assignable(actual, expected) {
    return checker.ok(names)
  }

  return checker.type_mismatch(names, name, expected, actual)
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

func checker.binary_is_arithmetic(op) {
  if op == "+" {
    return true
  }

  if op == "-" {
    return true
  }

  if op == "*" {
    return true
  }

  return op == "/"
}

func checker.binary_is_equality(op) {
  if op == "==" {
    return true
  }

  return op == "!="
}

func checker.binary_is_ordered(op) {
  if op == ">" {
    return true
  }

  if op == ">=" {
    return true
  }

  if op == "<" {
    return true
  }

  return op == "<="
}

func checker.binary_is_logical(op) {
  if op == "and" {
    return true
  }

  return op == "or"
}

func checker.validate_unary_operand_types(expression, names) {
  VTime op = get(expression, 1)
  VTime actual = checker.expr_type(get(expression, 2), names)

  if op == "-" {
    if checker.type_is_numeric(actual) {
      return checker.ok(names)
    }
  }

  if op == "not" {
    if checker.type_allows(actual, "Bool") {
      return checker.ok(names)
    }
  }

  return checker.fail(names, join(["invalid unary operation for ", actual], ""))
}

func checker.validate_binary_operand_types(expression, names) {
  VTime op = get(expression, 1)
  VTime left = checker.expr_type(get(expression, 2), names)
  VTime right = checker.expr_type(get(expression, 3), names)

  if left == "VTime" {
    return checker.ok(names)
  }

  if right == "VTime" {
    return checker.ok(names)
  }

  if checker.binary_is_arithmetic(op) {
    if left == right {
      if left == "Int" {
        return checker.ok(names)
      }

      if left == "Float" {
        return checker.ok(names)
      }
    }

    return checker.fail(names, join(["arithmetic requires matching Int or Float values, got ", left, " and ", right], ""))
  }

  if checker.binary_is_equality(op) {
    if left == "None" {
      return checker.ok(names)
    }

    if right == "None" {
      return checker.ok(names)
    }

    if left == right {
      return checker.ok(names)
    }

    return checker.fail(names, join(["comparison requires matching types, got ", left, " and ", right], ""))
  }

  if checker.binary_is_ordered(op) {
    if left == right {
      if left == "Int" {
        return checker.ok(names)
      }

      if left == "Float" {
        return checker.ok(names)
      }
    }

    return checker.fail(names, join(["ordered comparison requires matching Int or Float values, got ", left, " and ", right], ""))
  }

  if checker.binary_is_logical(op) {
    if left == "Bool" {
      if right == "Bool" {
        return checker.ok(names)
      }
    }

    return checker.fail(names, join(["logical operation requires Bool values, got ", left, " and ", right], ""))
  }

  return checker.fail(names, join(["unknown binary operator `", op, "`"], ""))
}

func checker.validate_index_operand_types(expression, names) {
  VTime target = checker.expr_type(get(expression, 1), names)

  if checker.type_is_pickable(target) != true {
    return checker.fail(names, join(["index access requires Str, Bytes, List, or VTime, got ", target], ""))
  }

  VTime index = checker.expr_type(get(expression, 2), names)

  if checker.type_allows(index, "Int") != true {
    return checker.fail(names, join(["index must be Int or VTime, got ", index], ""))
  }

  return checker.ok(names)
}

func checker.validate_slice_target_type(expression, names) {
  VTime target = checker.expr_type(get(expression, 1), names)

  if checker.type_is_pickable(target) {
    return checker.ok(names)
  }

  return checker.fail(names, join(["slice requires Str, Bytes, List, or VTime, got ", target], ""))
}

func checker.validate_slice_bound_type(bound, names) {
  VTime actual = checker.expr_type(bound, names)

  if checker.type_allows(actual, "Int") {
    return checker.ok(names)
  }

  return checker.fail(names, join(["slice bounds must be Int or VTime, got ", actual], ""))
}

func checker.validate_condition_type(expression, names) {
  VTime actual = checker.expr_type(expression, names)

  if checker.type_allows(actual, "Bool") {
    return checker.ok(names)
  }

  return checker.type_mismatch(names, "condition", "Bool", actual)
}

func checker.validate_pick_type(expression, names) {
  VTime actual = checker.expr_type(expression, names)

  if checker.type_is_pickable(actual) {
    return checker.ok(names)
  }

  return checker.fail(names, join(["pick requires Str, Bytes, List, or VTime, got ", actual], ""))
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

  VTime entry = checker.find_function(names, name)

  if entry == NONE {
    if checker.find_name(names, name) != NONE {
      return checker.fail(names, join(["invalid function `", name, "`"], ""))
    }

    return checker.fail(names, join(["unknown function `", name, "`"], ""))
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

  if checker.entry_protection(entry) == "AV" {
    VTime value_type = checker.entry_value_type(entry)

    if value_type == "Int" {
      return checker.ok(names)
    }

    if value_type == "Float" {
      return checker.ok(names)
    }
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

  VTime protection = checker.entry_protection(entry)

  if protection == "SASV" {
    return checker.fail(names, join(["invalid secretup target `", name, "`"], ""))
  }

  if protection == "ASV" {
    return checker.ok(checker.update_name_protection(names, name, "SASV"))
  }

  return checker.ok(checker.update_name_protection(names, name, "ASV"))
}

func checker.validate_info_target(names, name) {
  VTime entry = checker.find_name(names, name)

  if entry == NONE {
    return checker.fail(names, join(["unknown info target `", name, "`"], ""))
  }

  if checker.entry_value_type(entry) != "Str" {
    return checker.fail(names, join(["invalid info target `", name, "`"], ""))
  }

  if checker.entry_protection(entry) != "AV" {
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

func checker.apply_mutating_call_protection(expression, names) {
  if parser.expr_kind(expression) != parser.EXPR_CALL {
    return checker.ok(names)
  }

  if get(expression, 1) != "add" {
    return checker.ok(names)
  }

  VTime args = get(expression, 2)
  VTime target = get(args, 0)

  if parser.expr_kind(target) != parser.EXPR_VAR {
    return checker.ok(names)
  }

  VTime name = parser.expr_value(target)
  VTime entry = checker.find_name(names, name)

  if entry == NONE {
    return checker.ok(names)
  }

  VTime protection = checker.max_protection(checker.entry_protection(entry), checker.expr_protection(get(args, 1), names))
  return checker.ok(checker.update_name_protection(names, name, protection))
}

func checker.validate_expr(expression, names) {
  VTime kind = parser.expr_kind(expression)

  if kind == parser.EXPR_VAR {
    VTime name = parser.expr_value(expression)
    VTime entry = checker.find_name(names, name)

    if entry != NONE {
      if checker.entry_role(entry) != "Func" {
        return checker.ok(names)
      }
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

    if checker.entry_protection(entry) != "AV" {
      return checker.fail(names, join(["secret expression denied `", name, "`"], ""))
    }

    return checker.ok(names)
  }

  if kind == parser.EXPR_UNARY {
    VTime inner_state = checker.validate_expr(get(expression, 2), names)

    if get(inner_state, 0) != checker.STATUS_OK {
      return inner_state
    }

    return checker.validate_unary_operand_types(expression, names)
  }

  if kind == parser.EXPR_BINARY {
    VTime left_state = checker.validate_expr(get(expression, 2), names)

    if get(left_state, 0) != checker.STATUS_OK {
      return left_state
    }

    VTime right_state = checker.validate_expr(get(expression, 3), names)

    if get(right_state, 0) != checker.STATUS_OK {
      return right_state
    }

    return checker.validate_binary_operand_types(expression, names)
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

    VTime index_state = checker.validate_expr(get(expression, 2), names)

    if get(index_state, 0) != checker.STATUS_OK {
      return index_state
    }

    return checker.validate_index_operand_types(expression, names)
  }

  if kind == parser.EXPR_SLICE {
    VTime target_state = checker.validate_expr(get(expression, 1), names)

    if get(target_state, 0) != checker.STATUS_OK {
      return target_state
    }

    VTime target_type_state = checker.validate_slice_target_type(expression, names)

    if get(target_type_state, 0) != checker.STATUS_OK {
      return target_type_state
    }

    VTime start = get(expression, 2)
    VTime end = get(expression, 3)
    VTime step = get(expression, 4)

    if start != NONE {
      VTime start_state = checker.validate_expr(start, names)

      if get(start_state, 0) != checker.STATUS_OK {
        return start_state
      }

      VTime start_type_state = checker.validate_slice_bound_type(start, names)

      if get(start_type_state, 0) != checker.STATUS_OK {
        return start_type_state
      }
    }

    if end != NONE {
      VTime end_state = checker.validate_expr(end, names)

      if get(end_state, 0) != checker.STATUS_OK {
        return end_state
      }

      VTime end_type_state = checker.validate_slice_bound_type(end, names)

      if get(end_type_state, 0) != checker.STATUS_OK {
        return end_type_state
      }
    }

    if step != NONE {
      VTime step_state = checker.validate_expr(step, names)

      if get(step_state, 0) != checker.STATUS_OK {
        return step_state
      }

      VTime step_type_state = checker.validate_slice_bound_type(step, names)

      if get(step_type_state, 0) != checker.STATUS_OK {
        return step_type_state
      }
    }

    return checker.ok(names)
  }

  if kind == parser.EXPR_TAG {
    VTime value_state = checker.validate_expr(get(expression, 1), names)

    if get(value_state, 0) != checker.STATUS_OK {
      return value_state
    }

    return checker.validate_tag_protection(expression, names)
  }

  return checker.ok(names)
}

func checker.validate_optional_expr(expression, names) {
  if expression == NONE {
    return checker.ok(names)
  }

  return checker.validate_expr(expression, names)
}

func checker.validate_statement(node, names, allow_predeclared_func, in_function, loop_depth) {
  VTime kind = parser.node_kind(node)

  if kind == parser.NODE_ERROR {
    return checker.fail(names, get(node, 1))
  }

  VTime collision_state = checker.validate_decl_collision(node, names, allow_predeclared_func)

  if get(collision_state, 0) != checker.STATUS_OK {
    return collision_state
  }

  if kind == parser.NODE_DECL {
    VTime init_state = checker.validate_expr(get(node, 3), names)

    if get(init_state, 0) != checker.STATUS_OK {
      return init_state
    }

    VTime type_state = checker.validate_decl_type(node, names)

    if get(type_state, 0) != checker.STATUS_OK {
      return type_state
    }

    VTime protection_state = checker.validate_decl_protection(node, names)

    if get(protection_state, 0) != checker.STATUS_OK {
      return protection_state
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

    VTime value_state = checker.validate_expr(get(node, 3), names)

    if get(value_state, 0) != checker.STATUS_OK {
      return value_state
    }

    VTime type_state = checker.validate_assignment_type(node, names)

    if get(type_state, 0) != checker.STATUS_OK {
      return type_state
    }

    return checker.validate_assignment_protection(node, names)
  }

  if kind == parser.NODE_SECRETUP {
    return checker.validate_secretup_target(node, names)
  }

  if kind == parser.NODE_INFO_ASSIGN {
    return checker.validate_info_assignment(node, names)
  }

  if kind == parser.NODE_EXPR {
    VTime expression = get(node, 1)
    VTime expression_state = checker.validate_expr(expression, names)

    if get(expression_state, 0) != checker.STATUS_OK {
      return expression_state
    }

    return checker.apply_mutating_call_protection(expression, names)
  }

  if kind == parser.NODE_OUT {
    VTime expression = get(node, 1)
    VTime expression_state = checker.validate_expr(expression, names)

    if get(expression_state, 0) != checker.STATUS_OK {
      return expression_state
    }

    return checker.validate_public_expression(expression, names, "out")
  }

  if kind == parser.NODE_STOP {
    VTime expression = get(node, 1)
    VTime expression_state = checker.validate_optional_expr(expression, names)

    if get(expression_state, 0) != checker.STATUS_OK {
      return expression_state
    }

    return checker.validate_public_expression(expression, names, "stop")
  }

  if kind == parser.NODE_FAIL {
    VTime expression = get(node, 1)
    VTime expression_state = checker.validate_expr(expression, names)

    if get(expression_state, 0) != checker.STATUS_OK {
      return expression_state
    }

    return checker.validate_public_expression(expression, names, "fail")
  }

  if kind == parser.NODE_BREAK {
    if loop_depth <= 0 {
      return checker.fail(names, "break outside loop")
    }

    return checker.ok(names)
  }

  if kind == parser.NODE_CONTINUE {
    if loop_depth <= 0 {
      return checker.fail(names, "continue outside loop")
    }

    return checker.ok(names)
  }

  if kind == parser.NODE_RETURN {
    if in_function != true {
      return checker.fail(names, "return outside function")
    }

    return checker.validate_expr(get(node, 1), names)
  }

  if kind == parser.NODE_FUNC {
    VTime local_names = names[:]
    VTime params = []

    pick(get(node, 2)): param {
      VTime param_state = checker.validate_name(names, param)

      if get(param_state, 0) != checker.STATUS_OK {
        return param_state
      }

      if checker.value_exists(params, param) {
        return checker.fail(names, join(["duplicate parameter `", param, "`"], ""))
      }

      add(params, param)
      local_names = checker.add_name(local_names, param, "VTime", "VTime")
    }

    VTime body_state = checker.validate_block(get(node, 3), local_names, false, true, 0)

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

    VTime condition_type_state = checker.validate_condition_type(get(node, 1), names)

    if get(condition_type_state, 0) != checker.STATUS_OK {
      return condition_type_state
    }

    VTime body_state = checker.validate_block(get(node, 2), names[:], false, in_function, loop_depth)

    if get(body_state, 0) != checker.STATUS_OK {
      return body_state
    }

    names = checker.merge_global_names(names, get(body_state, 1))
    VTime else_state = checker.validate_block(get(node, 3), names[:], false, in_function, loop_depth)

    if get(else_state, 0) != checker.STATUS_OK {
      return else_state
    }

    return checker.ok(checker.merge_global_names(names, get(else_state, 1)))
  }

  if kind == parser.NODE_WHILE {
    VTime limit = get(node, 2)

    if limit < -1 {
      return checker.fail(names, join(["invalid loop limit `", str(limit), "`"], ""))
    }

    VTime condition_state = checker.validate_expr(get(node, 1), names)

    if get(condition_state, 0) != checker.STATUS_OK {
      return condition_state
    }

    VTime condition_type_state = checker.validate_condition_type(get(node, 1), names)

    if get(condition_type_state, 0) != checker.STATUS_OK {
      return condition_type_state
    }

    VTime body_state = checker.validate_block(get(node, 3), names[:], false, in_function, loop_depth + 1)

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

    VTime value_type_state = checker.validate_pick_type(get(node, 1), names)

    if get(value_type_state, 0) != checker.STATUS_OK {
      return value_type_state
    }

    VTime item_name = get(node, 2)
    VTime item_name_state = checker.validate_name(names, item_name)

    if get(item_name_state, 0) != checker.STATUS_OK {
      return item_name_state
    }

    if checker.name_exists(names, item_name) {
      return checker.fail(names, join(["duplicate name `", item_name, "`"], ""))
    }

    VTime local_names = checker.add_name_with_protection(names[:], item_name, "VTime", "VTime", checker.expr_protection(get(node, 1), names))
    VTime body_state = checker.validate_block(get(node, 3), local_names, false, in_function, loop_depth + 1)

    if get(body_state, 0) != checker.STATUS_OK {
      return body_state
    }

    return checker.ok(checker.merge_global_names(names, get(body_state, 1)))
  }

  return checker.ok(names)
}

func checker.validate_block(statements, names, allow_predeclared_func, in_function, loop_depth) {
  pick(statements): statement {
    VTime state = checker.validate_statement(statement, names, allow_predeclared_func, in_function, loop_depth)
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
      VTime name_state = checker.validate_name(names, name)

      if get(name_state, 0) != checker.STATUS_OK {
        return name_state
      }

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
    VTime name_state = checker.validate_name(names, name)

    if get(name_state, 0) != checker.STATUS_OK {
      return name_state
    }

    if checker.name_exists(names, name) {
      return checker.fail(names, join(["duplicate name `", name, "`"], ""))
    }

    if len(symbol) >= 4 {
      names = checker.add_name_with_protection(names, name, get(symbol, 1), get(symbol, 2), get(symbol, 3))
    } else {
      names = checker.add_name(names, name, get(symbol, 1), get(symbol, 2))
    }
  }

  return checker.ok(names)
}

func checker.export_symbols(names) {
  VTime symbols = []

  pick(names): entry {
    if checker.entry_role(entry) != "VTime" {
      add(symbols, [checker.entry_name(entry), checker.entry_role(entry), checker.entry_type(entry), checker.entry_protection(entry)])
    }
  }

  return symbols
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

  VTime state = checker.validate_block(statements, get(collect_state, 1), true, false, 0)

  if get(state, 0) != checker.STATUS_OK {
    return [get(state, 0), get(state, 2)]
  }

  return [checker.STATUS_OK, statements, checker.export_symbols(get(state, 1))]
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
