# APL parser bootstrap helpers.
# This module consumes tokens from std/lexer.apl and builds a small AST shape
# in APL lists. It is not the full parser yet; it is the first self-hosted
# parser layer.

AVStr parser.NODE_DECL = "Decl"
AVStr parser.NODE_LIST_DECL = "ListDecl"
AVStr parser.NODE_VTIME_DECL = "VTimeDecl"
AVStr parser.NODE_INFO_ASSIGN = "InfoAssign"
AVStr parser.NODE_ASSIGN = "Assign"
AVStr parser.NODE_EXPR = "Expr"
AVStr parser.NODE_OUT = "Out"
AVStr parser.NODE_STOP = "Stop"
AVStr parser.NODE_FAIL = "Fail"
AVStr parser.NODE_IF = "If"
AVStr parser.NODE_WHILE = "While"
AVStr parser.NODE_PICK = "Pick"
AVStr parser.NODE_BREAK = "Break"
AVStr parser.NODE_CONTINUE = "Continue"
AVStr parser.NODE_FUNC = "Func"
AVStr parser.NODE_RETURN = "Return"
AVStr parser.NODE_SECRETUP = "SecretUp"
AVStr parser.NODE_ERROR = "Error"

AVStr parser.EXPR_INT = "Int"
AVStr parser.EXPR_FLOAT = "Float"
AVStr parser.EXPR_STR = "Str"
AVStr parser.EXPR_BOOL = "Bool"
AVStr parser.EXPR_NONE = "None"
AVStr parser.EXPR_VAR = "Var"
AVStr parser.EXPR_INPUT = "Input"
AVStr parser.EXPR_SECRET_INPUT = "SecretInput"
AVStr parser.EXPR_UNARY = "Unary"
AVStr parser.EXPR_BINARY = "Binary"
AVStr parser.EXPR_CALL = "Call"
AVStr parser.EXPR_LIST = "List"
AVStr parser.EXPR_INDEX = "Index"
AVStr parser.EXPR_SLICE = "Slice"
AVStr parser.EXPR_SELF = "Self"
AVStr parser.EXPR_TAG = "Tag"
AVStr parser.EXPR_ERROR = "Error"

List parser.DECL_KEYWORDS = ["AVInt", "AVFloat", "AVBool", "AVStr", "AVBytes", "AVJson",
    "ASVInt", "ASVFloat", "ASVBool", "ASVStr", "ASVBytes", "ASVJson",
    "SASVInt", "SASVFloat", "SASVBool", "SASVStr", "SASVBytes", "SASVJson"]

List parser.ASSIGN_OPS = ["=", "+=", "-=", "*=", "/="]
List parser.BINARY_OPS = ["==", "!=", ">", "<", ">=", "<=", "and", "or", "+", "-", "*", "/"]

func parser.node_kind(node) {
  return get(node, 0)
}

func parser.expr_kind(expression) {
  return get(expression, 0)
}

func parser.expr_value(expression) {
  return get(expression, 1)
}

func parser.is_symbol_token(token, value) {
  if lexer.token_kind(token) != lexer.TOKEN_SYM {
    return false
  }

  return lexer.token_value(token) == value
}

func parser.is_decl_keyword(value) {
  pick(parser.DECL_KEYWORDS): keyword {
    if keyword == value {
      return true
    }
  }

  return false
}

func parser.is_assign_op(value) {
  pick(parser.ASSIGN_OPS): op {
    if op == value {
      return true
    }
  }

  return false
}

func parser.is_binary_op(value) {
  pick(parser.BINARY_OPS): op {
    if op == value {
      return true
    }
  }

  return false
}

func parser.is_factor_op(value) {
  if value == "*" {
    return true
  }

  if value == "/" {
    return true
  }

  return false
}

func parser.is_term_op(value) {
  if value == "+" {
    return true
  }

  if value == "-" {
    return true
  }

  return false
}

func parser.is_comparison_op(value) {
  if value == "==" {
    return true
  }

  if value == "!=" {
    return true
  }

  if value == ">" {
    return true
  }

  if value == "<" {
    return true
  }

  if value == ">=" {
    return true
  }

  if value == "<=" {
    return true
  }

  return false
}

func parser.parse_expr_token(token) {
  VTime kind = lexer.token_kind(token)
  VTime value = lexer.token_value(token)

  if kind == lexer.TOKEN_INT {
    return [parser.EXPR_INT, int(value)]
  }

  if kind == lexer.TOKEN_FLOAT {
    return [parser.EXPR_FLOAT, float(value)]
  }

  if kind == lexer.TOKEN_STR {
    return [parser.EXPR_STR, value]
  }

  if kind == lexer.TOKEN_IDENT {
    if (lexer.is_digit(value[0])) and (contains(value, ".")) {
      return [parser.EXPR_FLOAT, NONE]
    }
  }

  if value == "true" {
    return [parser.EXPR_BOOL, true]
  }

  if value == "false" {
    return [parser.EXPR_BOOL, false]
  }

  if value == "NONE" {
    return [parser.EXPR_NONE, NONE]
  }

  return [parser.EXPR_VAR, value]
}

func parser.parse_call_args(tokens, index) {
  VTime args = []

  while (index < len(tokens)) (-1) {
    VTime token = get(tokens, index)

    if parser.is_symbol_token(token, ")") {
      return [args, index + 1, NONE]
    }

    VTime parsed_arg = parser.parse_expression(tokens, index)
    VTime arg = get(parsed_arg, 0)

    if parser.expr_kind(arg) == parser.EXPR_ERROR {
      return [args, get(parsed_arg, 1), parser.expr_value(arg)]
    }

    add(args, arg)
    index = get(parsed_arg, 1)

    if index >= len(tokens) {
      return [args, index, "call expects )"]
    }

    VTime separator = get(tokens, index)

    if parser.is_symbol_token(separator, ")") {
      return [args, index + 1, NONE]
    }

    if parser.is_symbol_token(separator, ",") {
      index += 1
      VTime next_token = get(tokens, index)

      if (index >= len(tokens)) or (parser.is_symbol_token(next_token, ")")) {
        return [args, index, "expected expression after ,"]
      }

      continue
    }

    return [args, index, "call expects , or )"]
  }

  return [args, index, "call expects )"]
}

func parser.parse_list_items(tokens, index) {
  VTime items = []

  while (index < len(tokens)) (-1) {
    VTime token = get(tokens, index)

    if parser.is_symbol_token(token, "]") {
      return [items, index + 1, NONE]
    }

    VTime parsed_item = parser.parse_expression(tokens, index)
    VTime item = get(parsed_item, 0)

    if parser.expr_kind(item) == parser.EXPR_ERROR {
      return [items, get(parsed_item, 1), parser.expr_value(item)]
    }

    add(items, item)
    index = get(parsed_item, 1)

    if index >= len(tokens) {
      return [items, index, "list expects ]"]
    }

    VTime separator = get(tokens, index)

    if parser.is_symbol_token(separator, "]") {
      return [items, index + 1, NONE]
    }

    if parser.is_symbol_token(separator, ",") {
      index += 1
      VTime next_token = get(tokens, index)

      if (index >= len(tokens)) or (parser.is_symbol_token(next_token, "]")) {
        return [items, index, "expected expression after ,"]
      }

      continue
    }

    return [items, index, "list expects , or ]"]
  }

  return [items, index, "list expects ]"]
}

func parser.parse_expr_primary(tokens, index) {
  if index >= len(tokens) {
    return [[parser.EXPR_ERROR, "expected expression"], index]
  }

  VTime token = get(tokens, index)
  VTime kind = lexer.token_kind(token)
  VTime value = lexer.token_value(token)
  VTime next_token = get(tokens, index + 1)

  if (kind == lexer.TOKEN_SYM) and (value == "[") {
    VTime parsed_items = parser.parse_list_items(tokens, index + 1)

    if get(parsed_items, 2) != NONE {
      return [[parser.EXPR_ERROR, get(parsed_items, 2)], get(parsed_items, 1)]
    }

    return [[parser.EXPR_LIST, get(parsed_items, 0)], get(parsed_items, 1)]
  }

  if (kind == lexer.TOKEN_KW) and (value == "secret") {
    if lexer.token_value(next_token) == "input" {
      return [[parser.EXPR_SECRET_INPUT, NONE], index + 2]
    }
  }

  if (kind == lexer.TOKEN_KW) and (value == "input") {
    return [[parser.EXPR_INPUT, NONE], index + 1]
  }

  if (kind == lexer.TOKEN_SYM) and (value == "(") {
    VTime parsed_group = parser.parse_expression(tokens, index + 1)
    VTime close_index = get(parsed_group, 1)
    VTime close_token = get(tokens, close_index)

    if parser.is_symbol_token(close_token, ")") {
      return [get(parsed_group, 0), close_index + 1]
    }

    return [[parser.EXPR_ERROR, "group expects )"], close_index]
  }

  if (kind == lexer.TOKEN_IDENT) and (lexer.token_value(next_token) == "(") {
    VTime parsed_args = parser.parse_call_args(tokens, index + 2)

    if get(parsed_args, 2) != NONE {
      return [[parser.EXPR_ERROR, get(parsed_args, 2)], get(parsed_args, 1)]
    }

    return [[parser.EXPR_CALL, value, get(parsed_args, 0)], get(parsed_args, 1)]
  }

  if kind == lexer.TOKEN_SYM {
    return [[parser.EXPR_ERROR, "expected expression"], index + 1]
  }

  return [parser.parse_expr_token(token), index + 1]
}

func parser.parse_index_or_slice(tokens, target, index) {
  VTime token = get(tokens, index)

  if parser.is_symbol_token(token, ":") {
    VTime end = NONE
    VTime step = NONE
    index += 1
    token = get(tokens, index)

    if (parser.is_symbol_token(token, ":") == false) and (parser.is_symbol_token(token, "]") == false) {
      VTime parsed_end = parser.parse_expression(tokens, index)
      end = get(parsed_end, 0)
      index = get(parsed_end, 1)
      token = get(tokens, index)
    }

    if parser.is_symbol_token(token, ":") {
      index += 1
      token = get(tokens, index)

      if parser.is_symbol_token(token, "]") == false {
        VTime parsed_step = parser.parse_expression(tokens, index)
        step = get(parsed_step, 0)
        index = get(parsed_step, 1)
      }
    }

    token = get(tokens, index)
    if parser.is_symbol_token(token, "]") == false {
      return [[parser.EXPR_ERROR, "slice expects ]"], index]
    }

    return [[parser.EXPR_SLICE, target, NONE, end, step], index + 1]
  }

  VTime parsed_first = parser.parse_expression(tokens, index)
  VTime first = get(parsed_first, 0)

  if parser.expr_kind(first) == parser.EXPR_ERROR {
    return parsed_first
  }

  index = get(parsed_first, 1)
  token = get(tokens, index)

  if parser.is_symbol_token(token, ":") == false {
    if parser.is_symbol_token(token, "]") == false {
      return [[parser.EXPR_ERROR, "index expects ]"], index]
    }

    return [[parser.EXPR_INDEX, target, first], index + 1]
  }

  VTime end = NONE
  VTime step = NONE
  index += 1
  token = get(tokens, index)

  if (parser.is_symbol_token(token, ":") == false) and (parser.is_symbol_token(token, "]") == false) {
    VTime parsed_end = parser.parse_expression(tokens, index)
    end = get(parsed_end, 0)
    index = get(parsed_end, 1)
    token = get(tokens, index)
  }

  if parser.is_symbol_token(token, ":") {
    index += 1
    token = get(tokens, index)

    if parser.is_symbol_token(token, "]") == false {
      VTime parsed_step = parser.parse_expression(tokens, index)
      step = get(parsed_step, 0)
      index = get(parsed_step, 1)
    }
  }

  token = get(tokens, index)
  if parser.is_symbol_token(token, "]") == false {
    return [[parser.EXPR_ERROR, "slice expects ]"], index]
  }

  return [[parser.EXPR_SLICE, target, first, end, step], index + 1]
}

func parser.parse_expr_postfix(tokens, index) {
  VTime parsed = parser.parse_expr_primary(tokens, index)
  VTime expression = get(parsed, 0)
  index = get(parsed, 1)

  while (index < len(tokens)) (-1) {
    VTime token = get(tokens, index)

    if lexer.token_kind(token) != lexer.TOKEN_SYM {
      return [expression, index]
    }

    if lexer.token_value(token) != "[" {
      return [expression, index]
    }

    parsed = parser.parse_index_or_slice(tokens, expression, index + 1)
    expression = get(parsed, 0)
    index = get(parsed, 1)
  }

  return [expression, index]
}

func parser.parse_expr_unary(tokens, index) {
  VTime token = get(tokens, index)
  VTime kind = lexer.token_kind(token)
  VTime op = lexer.token_value(token)

  if (kind == lexer.TOKEN_SYM) and (op == "-") {
    VTime next_token = get(tokens, index + 1)
    VTime next_kind = lexer.token_kind(next_token)

    if next_kind == lexer.TOKEN_INT {
      VTime signed_value = join(["-", lexer.token_value(next_token)], "")
      return [[parser.EXPR_INT, int(signed_value)], index + 2]
    }

    if next_kind == lexer.TOKEN_FLOAT {
      VTime signed_value = join(["-", lexer.token_value(next_token)], "")
      return [[parser.EXPR_FLOAT, float(signed_value)], index + 2]
    }

    VTime parsed_value = parser.parse_expr_unary(tokens, index + 1)
    return [[parser.EXPR_UNARY, op, get(parsed_value, 0)], get(parsed_value, 1)]
  }

  if (kind == lexer.TOKEN_KW) and (op == "not") {
    VTime parsed_value = parser.parse_expr_unary(tokens, index + 1)
    return [[parser.EXPR_UNARY, op, get(parsed_value, 0)], get(parsed_value, 1)]
  }

  return parser.parse_expr_postfix(tokens, index)
}

func parser.parse_expr_self(tokens, index) {
  VTime parsed_left = parser.parse_expr_unary(tokens, index)
  VTime left = get(parsed_left, 0)
  VTime next_index = get(parsed_left, 1)
  VTime op_token = get(tokens, next_index)
  VTime op = lexer.token_value(op_token)

  if op == "=self=" {
    if parser.expr_kind(left) == parser.EXPR_VAR {
      return [[parser.EXPR_SELF, parser.expr_value(left)], next_index + 1]
    }
  }

  return [left, next_index]
}

func parser.parse_expr_factor(tokens, index) {
  VTime parsed_left = parser.parse_expr_self(tokens, index)
  VTime left = get(parsed_left, 0)
  VTime next_index = get(parsed_left, 1)
  VTime token = get(tokens, next_index)
  VTime op = lexer.token_value(token)

  while (parser.is_factor_op(op)) (-1) {
    VTime parsed_right = parser.parse_expr_self(tokens, next_index + 1)
    left = [parser.EXPR_BINARY, op, left, get(parsed_right, 0)]
    next_index = get(parsed_right, 1)
    token = get(tokens, next_index)
    op = lexer.token_value(token)
  }

  return [left, next_index]
}

func parser.parse_expr_term(tokens, index) {
  VTime parsed_left = parser.parse_expr_factor(tokens, index)
  VTime left = get(parsed_left, 0)
  VTime next_index = get(parsed_left, 1)
  VTime token = get(tokens, next_index)
  VTime op = lexer.token_value(token)

  while (parser.is_term_op(op)) (-1) {
    VTime parsed_right = parser.parse_expr_factor(tokens, next_index + 1)
    left = [parser.EXPR_BINARY, op, left, get(parsed_right, 0)]
    next_index = get(parsed_right, 1)
    token = get(tokens, next_index)
    op = lexer.token_value(token)
  }

  return [left, next_index]
}

func parser.parse_expr_comparison(tokens, index) {
  VTime parsed_left = parser.parse_expr_term(tokens, index)
  VTime left = get(parsed_left, 0)
  VTime next_index = get(parsed_left, 1)
  VTime token = get(tokens, next_index)
  VTime op = lexer.token_value(token)

  if parser.is_comparison_op(op) {
    VTime parsed_right = parser.parse_expr_term(tokens, next_index + 1)
    return [[parser.EXPR_BINARY, op, left, get(parsed_right, 0)], get(parsed_right, 1)]
  }

  return [left, next_index]
}

func parser.parse_expr_and(tokens, index) {
  VTime parsed_left = parser.parse_expr_comparison(tokens, index)
  VTime left = get(parsed_left, 0)
  VTime next_index = get(parsed_left, 1)
  VTime token = get(tokens, next_index)
  VTime op = lexer.token_value(token)

  while (op == "and") (-1) {
    VTime parsed_right = parser.parse_expr_comparison(tokens, next_index + 1)
    left = [parser.EXPR_BINARY, op, left, get(parsed_right, 0)]
    next_index = get(parsed_right, 1)
    token = get(tokens, next_index)
    op = lexer.token_value(token)
  }

  return [left, next_index]
}

func parser.parse_expr_or(tokens, index) {
  VTime parsed_left = parser.parse_expr_and(tokens, index)
  VTime left = get(parsed_left, 0)
  VTime next_index = get(parsed_left, 1)
  VTime token = get(tokens, next_index)
  VTime op = lexer.token_value(token)

  while (op == "or") (-1) {
    VTime parsed_right = parser.parse_expr_and(tokens, next_index + 1)
    left = [parser.EXPR_BINARY, op, left, get(parsed_right, 0)]
    next_index = get(parsed_right, 1)
    token = get(tokens, next_index)
    op = lexer.token_value(token)
  }

  return [left, next_index]
}

func parser.parse_expression(tokens, index) {
  VTime parsed_left = parser.parse_expr_or(tokens, index)
  VTime left = get(parsed_left, 0)
  VTime next_index = get(parsed_left, 1)
  VTime op_token = get(tokens, next_index)
  VTime op = lexer.token_value(op_token)

  if op == ":" {
    VTime tag_token = get(tokens, next_index + 1)
    VTime tag = lexer.token_value(tag_token)

    if tag == "AV" {
      return [[parser.EXPR_TAG, left, tag], next_index + 2]
    }

    if tag == "ASV" {
      return [[parser.EXPR_TAG, left, tag], next_index + 2]
    }

    if tag == "SASV" {
      return [[parser.EXPR_TAG, left, tag], next_index + 2]
    }
  }

  return [left, next_index]
}

func parser.error(message, next_index) {
  return [[parser.NODE_ERROR, message], next_index]
}

func parser.parse_statement(tokens, index) {
  if index >= len(tokens) {
    return parser.error("unexpected end", index)
  }

  VTime token = get(tokens, index)
  VTime kind = lexer.token_kind(token)
  VTime value = lexer.token_value(token)

  if value == "func" {
    VTime name_token = get(tokens, index + 1)
    VTime params_open = get(tokens, index + 2)

    if lexer.token_value(params_open) != "(" {
      return parser.error("func expects (", index + 2)
    }

    VTime params = []
    VTime param_index = index + 3

    while (param_index < len(tokens)) (-1) {
      VTime param_token = get(tokens, param_index)

      if lexer.token_value(param_token) == ")" {
        break
      }

      add(params, lexer.token_value(param_token))
      param_index += 1

      VTime comma = get(tokens, param_index)
      if lexer.token_value(comma) == "," {
        param_index += 1
      }
    }

    VTime body_open_index = param_index + 1
    VTime body_open = get(tokens, body_open_index)

    if lexer.token_value(body_open) != "{" {
      return parser.error("func expects {", body_open_index)
    }

    VTime parsed_body = parser.parse_block(tokens, body_open_index + 1, true)
    return [[parser.NODE_FUNC, lexer.token_value(name_token), params, get(parsed_body, 0)], get(parsed_body, 1)]
  }

  if value == "List" {
    VTime name_token = get(tokens, index + 1)
    VTime assign_token = get(tokens, index + 2)

    if lexer.token_value(assign_token) != "=" {
      return parser.error("list declaration expects =", index + 2)
    }

    VTime parsed_value = parser.parse_expression(tokens, index + 3)
    return [[parser.NODE_LIST_DECL, lexer.token_value(name_token), get(parsed_value, 0)], get(parsed_value, 1)]
  }

  if value == "VTime" {
    VTime name_token = get(tokens, index + 1)
    VTime assign_token = get(tokens, index + 2)

    if lexer.token_value(assign_token) != "=" {
      return parser.error("vtime declaration expects =", index + 2)
    }

    VTime parsed_value = parser.parse_expression(tokens, index + 3)
    return [[parser.NODE_VTIME_DECL, lexer.token_value(name_token), get(parsed_value, 0)], get(parsed_value, 1)]
  }

  if value == "if" {
    VTime parsed_condition = parser.parse_expression(tokens, index + 1)
    VTime open_index = get(parsed_condition, 1)
    VTime open_token = get(tokens, open_index)

    if lexer.token_value(open_token) != "{" {
      return parser.error("if expects {", open_index)
    }

    VTime parsed_body = parser.parse_block(tokens, open_index + 1, true)
    VTime next_index = get(parsed_body, 1)
    VTime else_body = []
    VTime next_token = get(tokens, next_index)

    if lexer.token_value(next_token) == "else" {
      VTime else_open = get(tokens, next_index + 1)

      if lexer.token_value(else_open) == "if" {
        VTime parsed_else_if = parser.parse_statement(tokens, next_index + 1)
        else_body = [get(parsed_else_if, 0)]
        next_index = get(parsed_else_if, 1)
      } else {

        if lexer.token_value(else_open) != "{" {
          return parser.error("else expects {", next_index + 1)
        }

        VTime parsed_else = parser.parse_block(tokens, next_index + 2, true)
        else_body = get(parsed_else, 0)
        next_index = get(parsed_else, 1)
      }
    }

    return [[parser.NODE_IF, get(parsed_condition, 0), get(parsed_body, 0), else_body], next_index]
  }

  if value == "while" {
    VTime condition_open = get(tokens, index + 1)

    if lexer.token_value(condition_open) != "(" {
      return parser.error("while expects condition (", index + 1)
    }

    VTime parsed_condition = parser.parse_expression(tokens, index + 2)
    VTime condition_close_index = get(parsed_condition, 1)
    VTime condition_close = get(tokens, condition_close_index)

    if lexer.token_value(condition_close) != ")" {
      return parser.error("while expects condition )", condition_close_index)
    }

    VTime limit_open_index = condition_close_index + 1
    VTime limit_open = get(tokens, limit_open_index)

    if lexer.token_value(limit_open) != "(" {
      return parser.error("while expects limit (", limit_open_index)
    }

    VTime limit_token = get(tokens, limit_open_index + 1)
    VTime limit_close_index = limit_open_index + 2
    VTime limit = int(lexer.token_value(limit_token))

    if lexer.token_value(limit_token) == "-" {
      VTime limit_value_token = get(tokens, limit_open_index + 2)
      limit = int(lexer.token_value(limit_value_token)) * -1
      limit_close_index = limit_open_index + 3
    }

    VTime limit_close = get(tokens, limit_close_index)

    if lexer.token_value(limit_close) != ")" {
      return parser.error("while expects limit )", limit_close_index)
    }

    VTime body_open_index = limit_close_index + 1
    VTime body_open = get(tokens, body_open_index)

    if lexer.token_value(body_open) != "{" {
      return parser.error("while expects {", body_open_index)
    }

    VTime parsed_body = parser.parse_block(tokens, body_open_index + 1, true)
    return [[parser.NODE_WHILE, get(parsed_condition, 0), limit, get(parsed_body, 0)], get(parsed_body, 1)]
  }

  if value == "pick" {
    VTime value_open = get(tokens, index + 1)

    if lexer.token_value(value_open) != "(" {
      return parser.error("pick expects (", index + 1)
    }

    VTime parsed_value = parser.parse_expression(tokens, index + 2)
    VTime value_close_index = get(parsed_value, 1)
    VTime value_close = get(tokens, value_close_index)

    if lexer.token_value(value_close) != ")" {
      return parser.error("pick expects )", value_close_index)
    }

    VTime colon_index = value_close_index + 1
    VTime colon = get(tokens, colon_index)

    if lexer.token_value(colon) != ":" {
      return parser.error("pick expects :", colon_index)
    }

    VTime item_token = get(tokens, colon_index + 1)
    VTime body_open_index = colon_index + 2
    VTime body_open = get(tokens, body_open_index)

    if lexer.token_value(body_open) != "{" {
      return parser.error("pick expects {", body_open_index)
    }

    VTime parsed_body = parser.parse_block(tokens, body_open_index + 1, true)
    return [[parser.NODE_PICK, get(parsed_value, 0), lexer.token_value(item_token), get(parsed_body, 0)], get(parsed_body, 1)]
  }

  if (kind == lexer.TOKEN_KW) and (parser.is_decl_keyword(value)) {
    VTime name_token = get(tokens, index + 1)
    VTime assign_token = get(tokens, index + 2)
    VTime parsed_value = parser.parse_expression(tokens, index + 3)

    if lexer.token_value(assign_token) != "=" {
      return parser.error("declaration expects =", index + 2)
    }

    return [[parser.NODE_DECL, value, lexer.token_value(name_token), get(parsed_value, 0)], get(parsed_value, 1)]
  }

  if value == "out" {
    VTime parsed_value = parser.parse_expression(tokens, index + 1)
    return [[parser.NODE_OUT, get(parsed_value, 0)], get(parsed_value, 1)]
  }

  if value == "stop" {
    if index + 1 >= len(tokens) {
      return [[parser.NODE_STOP, NONE], index + 1]
    }

    VTime parsed_value = parser.parse_expression(tokens, index + 1)
    return [[parser.NODE_STOP, get(parsed_value, 0)], get(parsed_value, 1)]
  }

  if value == "fail" {
    VTime parsed_value = parser.parse_expression(tokens, index + 1)
    return [[parser.NODE_FAIL, get(parsed_value, 0)], get(parsed_value, 1)]
  }

  if value == "return" {
    VTime parsed_value = parser.parse_expression(tokens, index + 1)
    return [[parser.NODE_RETURN, get(parsed_value, 0)], get(parsed_value, 1)]
  }

  if value == "secretup" {
    VTime open_token = get(tokens, index + 1)
    VTime name_token = get(tokens, index + 2)
    VTime close_token = get(tokens, index + 3)

    if lexer.token_value(open_token) != "(" {
      return parser.error("secretup expects (", index + 1)
    }

    if lexer.token_value(close_token) != ")" {
      return parser.error("secretup expects )", index + 3)
    }

    return [[parser.NODE_SECRETUP, lexer.token_value(name_token)], index + 4]
  }

  if value == "break" {
    return [[parser.NODE_BREAK], index + 1]
  }

  if value == "continue" {
    return [[parser.NODE_CONTINUE], index + 1]
  }

  if kind == lexer.TOKEN_IDENT {
    VTime op_token = get(tokens, index + 1)
    VTime op = lexer.token_value(op_token)

    if op == "," {
      VTime level_token = get(tokens, index + 2)
      VTime assign_token = get(tokens, index + 3)
      VTime info_token = get(tokens, index + 4)
      VTime open_token = get(tokens, index + 5)
      VTime source_token = get(tokens, index + 6)
      VTime close_token = get(tokens, index + 7)

      if lexer.token_value(assign_token) != "=" {
        return parser.error("info assignment expects =", index + 3)
      }

      if lexer.token_value(info_token) != "info" {
        return parser.error("info assignment expects info", index + 4)
      }

      if lexer.token_value(open_token) != "(" {
        return parser.error("info assignment expects (", index + 5)
      }

      if lexer.token_value(close_token) != ")" {
        return parser.error("info assignment expects )", index + 7)
      }

      return [[parser.NODE_INFO_ASSIGN, value, lexer.token_value(level_token), lexer.token_value(source_token)], index + 8]
    }

    if op == "(" {
      VTime parsed_expr = parser.parse_expression(tokens, index)
      return [[parser.NODE_EXPR, get(parsed_expr, 0)], get(parsed_expr, 1)]
    }

    if parser.is_assign_op(op) {
      VTime parsed_value = parser.parse_expression(tokens, index + 2)
      return [[parser.NODE_ASSIGN, value, op, get(parsed_value, 0)], get(parsed_value, 1)]
    }
  }

  return parser.error("unknown statement", index + 1)
}

func parser.parse_block(tokens, index, expect_close) {
  VTime statements = []

  while (index < len(tokens)) (-1) {
    VTime token = get(tokens, index)

    if parser.is_symbol_token(token, "}") {
      if expect_close {
        return [statements, index + 1, true]
      }

      add(statements, [parser.NODE_ERROR, "unexpected }"])
      return [statements, index + 1, false]
    }

    VTime parsed = parser.parse_statement(tokens, index)
    VTime statement = get(parsed, 0)
    add(statements, statement)
    index = get(parsed, 1)

    if parser.node_kind(statement) == parser.NODE_ERROR {
      return [statements, index]
    }
  }

  if expect_close {
    add(statements, [parser.NODE_ERROR, "expected }"])
  }

  return [statements, index, false]
}

func parser.parse_tokens(tokens) {
  VTime parsed = parser.parse_block(tokens, 0, false)
  VTime statements = get(parsed, 0)

  return statements
}

func parser.parse_source(source) {
  return parser.parse_tokens(lexer.tokenize(source))
}
