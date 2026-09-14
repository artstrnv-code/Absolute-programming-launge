# APL VM bootstrap helpers.
# This module executes the small list-based IR from std/ir.apl. It is not the
# final VM, but it proves runtime behavior can move into compiled APL code.

AVStr vm.FLOW_OK = "OK"
AVStr vm.FLOW_STOP = "STOP"
AVStr vm.FLOW_FAIL = "FAIL"
AVStr vm.FLOW_BREAK = "BREAK"
AVStr vm.FLOW_CONTINUE = "CONTINUE"
AVStr vm.FLOW_RETURN = "RETURN"
AVStr vm.KIND_LIST = "LIST_KIND"

func vm.new_env() {
  return vm.new_env_with_input([])
}

func vm.new_env_with_input(inputs) {
  return [[], [], [], [], [], inputs, 0]
}

func vm.env_names(env) {
  return get(env, 0)
}

func vm.env_values(env) {
  return get(env, 1)
}

func vm.env_kinds(env) {
  return get(env, 2)
}

func vm.env_types(env) {
  return get(env, 3)
}

func vm.env_initials(env) {
  return get(env, 4)
}

func vm.env_inputs(env) {
  return get(env, 5)
}

func vm.env_input_index(env) {
  return get(env, 6)
}

func vm.env_with_input_index(env, input_index) {
  return [vm.env_names(env), vm.env_values(env), vm.env_kinds(env), vm.env_types(env), vm.env_initials(env), vm.env_inputs(env), input_index]
}

func vm.env_read_input(env) {
  VTime inputs = vm.env_inputs(env)
  VTime input_index = vm.env_input_index(env)

  if input_index >= len(inputs) {
    return [NONE, env]
  }

  return [get(inputs, input_index), vm.env_with_input_index(env, input_index + 1)]
}

func vm.env_put(env, name, value) {
  VTime names = vm.env_names(env)
  VTime values = vm.env_values(env)
  VTime kinds = vm.env_kinds(env)
  VTime types = vm.env_types(env)
  VTime initials = vm.env_initials(env)
  VTime inputs = vm.env_inputs(env)
  VTime input_index = vm.env_input_index(env)
  add(names, name)
  add(values, value)
  add(kinds, vm.env_kind(env, name))
  add(types, vm.env_type(env, name))
  add(initials, vm.env_initial(env, name))
  return [names, values, kinds, types, initials, inputs, input_index]
}

func vm.env_put_meta(env, name, value, kind, value_type) {
  VTime names = vm.env_names(env)
  VTime values = vm.env_values(env)
  VTime kinds = vm.env_kinds(env)
  VTime types = vm.env_types(env)
  VTime initials = vm.env_initials(env)
  VTime inputs = vm.env_inputs(env)
  VTime input_index = vm.env_input_index(env)
  add(names, name)
  add(values, value)
  add(kinds, kind)
  add(types, value_type)
  add(initials, vm.env_initial_or_value(env, name, value))
  return [names, values, kinds, types, initials, inputs, input_index]
}

func vm.env_get(env, name) {
  VTime names = vm.env_names(env)
  VTime values = vm.env_values(env)
  VTime index = len(names) - 1

  while (index >= 0) (-1) {
    if get(names, index) == name {
      return get(values, index)
    }

    index -= 1
  }

  return NONE
}

func vm.env_has(env, name) {
  VTime names = vm.env_names(env)
  VTime index = len(names) - 1

  while (index >= 0) (-1) {
    if get(names, index) == name {
      return true
    }

    index -= 1
  }

  return false
}

func vm.env_kind(env, name) {
  VTime names = vm.env_names(env)
  VTime kinds = vm.env_kinds(env)
  VTime index = len(names) - 1

  while (index >= 0) (-1) {
    if get(names, index) == name {
      return get(kinds, index)
    }

    index -= 1
  }

  return "VTime"
}

func vm.env_type(env, name) {
  VTime names = vm.env_names(env)
  VTime types = vm.env_types(env)
  VTime index = len(names) - 1

  while (index >= 0) (-1) {
    if get(names, index) == name {
      return get(types, index)
    }

    index -= 1
  }

  return "VTime"
}

func vm.env_initial(env, name) {
  VTime names = vm.env_names(env)
  VTime initials = vm.env_initials(env)
  VTime index = len(names) - 1

  while (index >= 0) (-1) {
    if get(names, index) == name {
      return get(initials, index)
    }

    index -= 1
  }

  return NONE
}

func vm.env_initial_or_value(env, name, value) {
  VTime initial = vm.env_initial(env, name)

  if initial == NONE {
    return value
  }

  return initial
}

func vm.env_is_public(env, name) {
  return vm.kind_is_public(vm.env_kind(env, name))
}

func vm.is_list_kind(kind) {
  return get(kind, 0) == vm.KIND_LIST
}

func vm.list_kind(kinds) {
  return [vm.KIND_LIST, kinds]
}

func vm.list_kind_values(kind) {
  return get(kind, 1)
}

func vm.list_item_kind(kind, index) {
  if vm.is_list_kind(kind) {
    return get(vm.list_kind_values(kind), index)
  }

  return kind
}

func vm.list_kind_after_add(kind, item_kind) {
  if vm.is_list_kind(kind) {
    VTime kinds = vm.list_kind_values(kind)
    add(kinds, item_kind)
    return vm.list_kind(kinds)
  }

  return vm.max_kind(kind, item_kind)
}

func vm.list_kind_after_pop(kind) {
  if vm.is_list_kind(kind) {
    VTime kinds = vm.list_kind_values(kind)
    VTime ignored = pop(kinds)
    return vm.list_kind(kinds)
  }

  return kind
}

func vm.slice_kind_with_bounds(kind, start_kind, end_kind, step_kind) {
  return vm.max_kind(vm.max_kind(vm.max_kind(kind, start_kind), end_kind), step_kind)
}

func vm.slice_kind(kind, start, end, step, start_kind, end_kind, step_kind) {
  if vm.is_list_kind(kind) != true {
    return vm.slice_kind_with_bounds(kind, start_kind, end_kind, step_kind)
  }

  VTime kinds = vm.list_kind_values(kind)

  if start == NONE {
    if end == NONE {
      if step == NONE {
        VTime sliced = kinds[:]
        return vm.slice_kind_with_bounds(vm.list_kind(sliced), start_kind, end_kind, step_kind)
      }

      VTime sliced = kinds[::step]
      return vm.slice_kind_with_bounds(vm.list_kind(sliced), start_kind, end_kind, step_kind)
    }

    if step == NONE {
      VTime sliced = kinds[:end]
      return vm.slice_kind_with_bounds(vm.list_kind(sliced), start_kind, end_kind, step_kind)
    }

    VTime sliced = kinds[:end:step]
    return vm.slice_kind_with_bounds(vm.list_kind(sliced), start_kind, end_kind, step_kind)
  }

  if end == NONE {
    if step == NONE {
      VTime sliced = kinds[start:]
      return vm.slice_kind_with_bounds(vm.list_kind(sliced), start_kind, end_kind, step_kind)
    }

    VTime sliced = kinds[start::step]
    return vm.slice_kind_with_bounds(vm.list_kind(sliced), start_kind, end_kind, step_kind)
  }

  if step == NONE {
    VTime sliced = kinds[start:end]
    return vm.slice_kind_with_bounds(vm.list_kind(sliced), start_kind, end_kind, step_kind)
  }

  VTime sliced = kinds[start:end:step]
  return vm.slice_kind_with_bounds(vm.list_kind(sliced), start_kind, end_kind, step_kind)
}

func vm.aggregate_kind(kind) {
  if vm.is_list_kind(kind) {
    VTime kinds = vm.list_kind_values(kind)
    VTime result = "AV"
    VTime index = 0

    while (index < len(kinds)) (-1) {
      result = vm.max_kind(result, get(kinds, index))
      index += 1
    }

    return result
  }

  return kind
}

func vm.kind_is_public(kind) {
  VTime plain_kind = vm.aggregate_kind(kind)

  if plain_kind == "ASV" {
    return false
  }

  if plain_kind == "SASV" {
    return false
  }

  return true
}

func vm.kind_rank(kind) {
  VTime plain_kind = vm.aggregate_kind(kind)

  if plain_kind == "SASV" {
    return 2
  }

  if plain_kind == "ASV" {
    return 1
  }

  return 0
}

func vm.max_kind(left, right) {
  if vm.kind_rank(left) >= vm.kind_rank(right) {
    return left
  }

  return right
}

func vm.kind_allows(target, source) {
  return vm.kind_rank(target) >= vm.kind_rank(source)
}

func vm.up_kind(kind) {
  if kind == "AV" {
    return "ASV"
  }

  return "SASV"
}

func vm.decl_kind(decl_keyword) {
  if decl_keyword[0:4] == "SASV" {
    return "SASV"
  }

  if decl_keyword[0:3] == "ASV" {
    return "ASV"
  }

  return "AV"
}

func vm.decl_type(decl_keyword) {
  if decl_keyword[0:4] == "SASV" {
    return decl_keyword[4:]
  }

  if decl_keyword[0:3] == "ASV" {
    return decl_keyword[3:]
  }

  return decl_keyword[2:]
}

func vm.env_without(env, name) {
  VTime old_names = vm.env_names(env)
  VTime old_values = vm.env_values(env)
  VTime old_kinds = vm.env_kinds(env)
  VTime old_types = vm.env_types(env)
  VTime old_initials = vm.env_initials(env)
  VTime inputs = vm.env_inputs(env)
  VTime input_index = vm.env_input_index(env)
  VTime names = []
  VTime values = []
  VTime kinds = []
  VTime types = []
  VTime initials = []
  VTime index = 0

  while (index < len(old_names)) (-1) {
    if get(old_names, index) != name {
      add(names, get(old_names, index))
      add(values, get(old_values, index))
      add(kinds, get(old_kinds, index))
      add(types, get(old_types, index))
      add(initials, get(old_initials, index))
    }

    index += 1
  }

  return [names, values, kinds, types, initials, inputs, input_index]
}

func vm.new_functions() {
  return [[], [], [], true]
}

func vm.func_names(functions) {
  return get(functions, 0)
}

func vm.func_params(functions) {
  return get(functions, 1)
}

func vm.func_bodies(functions) {
  return get(functions, 2)
}

func vm.functions_ok(functions) {
  return get(functions, 3)
}

func vm.functions_fail(functions) {
  return [vm.func_names(functions), vm.func_params(functions), vm.func_bodies(functions), false]
}

func vm.func_put(functions, name, params, body) {
  VTime names = vm.func_names(functions)
  VTime params_list = vm.func_params(functions)
  VTime bodies = vm.func_bodies(functions)
  add(names, name)
  add(params_list, params)
  add(bodies, body)
  return [names, params_list, bodies, vm.functions_ok(functions)]
}

func vm.func_index(functions, name) {
  VTime names = vm.func_names(functions)
  VTime index = len(names) - 1

  while (index >= 0) (-1) {
    if get(names, index) == name {
      return index
    }

    index -= 1
  }

  return NONE
}

func vm.func_has(functions, name) {
  return vm.func_index(functions, name) != NONE
}

func vm.collect_functions(program) {
  VTime functions = vm.new_functions()

  pick(program): instruction {
    if ir.opcode(instruction) == ir.OP_FUNC {
      if vm.func_has(functions, get(instruction, 1)) {
        return vm.functions_fail(functions)
      }

      functions = vm.func_put(functions, get(instruction, 1), get(instruction, 2), get(instruction, 3))
    }
  }

  return functions
}

func vm.eval_expr(expression, env, output, functions) {
  VTime opcode = ir.expr_opcode(expression)

  if opcode == ir.EXPR_UNARY {
    return vm.eval_unary(get(expression, 1), get(expression, 2), env, output, functions)
  }

  if opcode == ir.EXPR_BINARY {
    return vm.eval_binary(get(expression, 1), get(expression, 2), get(expression, 3), env, output, functions)
  }

  if opcode == ir.EXPR_LITERAL {
    return [ir.expr_value(expression), env, output, "AV"]
  }

  if opcode == ir.EXPR_LOAD {
    return [vm.env_get(env, ir.expr_value(expression)), env, output, vm.env_kind(env, ir.expr_value(expression))]
  }

  if opcode == ir.EXPR_SELF {
    VTime name = ir.expr_value(expression)
    return [vm.env_get(env, name) == vm.env_initial(env, name), env, output, vm.env_kind(env, name)]
  }

  if opcode == ir.EXPR_TAG {
    VTime value_state = vm.eval_expr(get(expression, 1), env, output, functions)
    return [get(value_state, 0), get(value_state, 1), get(value_state, 2), vm.max_kind(get(value_state, 3), get(expression, 2))]
  }

  if opcode == ir.EXPR_NONE {
    return [NONE, env, output, "AV"]
  }

  if opcode == ir.EXPR_INPUT {
    VTime input_state = vm.env_read_input(env)
    return [get(input_state, 0), get(input_state, 1), output, "AV"]
  }

  if opcode == ir.EXPR_SECRET_INPUT {
    VTime input_state = vm.env_read_input(env)
    return [get(input_state, 0), get(input_state, 1), output, "ASV"]
  }

  if opcode == ir.EXPR_CALL {
    if vm.is_builtin_call(get(expression, 1)) {
      return vm.call_builtin(get(expression, 1), get(expression, 2), env, output, functions)
    }

    return vm.call_func(get(expression, 1), get(expression, 2), env, output, functions)
  }

  if opcode == ir.EXPR_LIST {
    VTime values = []
    VTime list_kinds = []

    pick(get(expression, 1)): item_expr {
      VTime item_state = vm.eval_expr(item_expr, env, output, functions)
      add(values, get(item_state, 0))
      add(list_kinds, get(item_state, 3))
      env = get(item_state, 1)
      output = get(item_state, 2)
    }

    return [values, env, output, vm.list_kind(list_kinds)]
  }

  if opcode == ir.EXPR_INDEX {
    VTime target_state = vm.eval_expr(get(expression, 1), env, output, functions)
    VTime target = get(target_state, 0)
    env = get(target_state, 1)
    output = get(target_state, 2)
    VTime index_state = vm.eval_expr(get(expression, 2), env, output, functions)
    VTime index = get(index_state, 0)
    env = get(index_state, 1)
    output = get(index_state, 2)
    return [get(target, index), env, output, vm.max_kind(vm.list_item_kind(get(target_state, 3), index), get(index_state, 3))]
  }

  if opcode == ir.EXPR_SLICE {
    return vm.eval_slice_expr(expression, env, output, functions)
  }

  return [NONE, env, output, "AV"]
}

func vm.eval_slice_expr(expression, env, output, functions) {
  VTime target_state = vm.eval_expr(get(expression, 1), env, output, functions)
  VTime target = get(target_state, 0)
  env = get(target_state, 1)
  output = get(target_state, 2)
  VTime start_state = vm.eval_expr(get(expression, 2), env, output, functions)
  VTime start = get(start_state, 0)
  env = get(start_state, 1)
  output = get(start_state, 2)
  VTime end_state = vm.eval_expr(get(expression, 3), env, output, functions)
  VTime end = get(end_state, 0)
  env = get(end_state, 1)
  output = get(end_state, 2)
  VTime step_state = vm.eval_expr(get(expression, 4), env, output, functions)
  VTime step = get(step_state, 0)
  env = get(step_state, 1)
  output = get(step_state, 2)

  if start == NONE {
    if end == NONE {
      if step == NONE {
        return [target[:], env, output, vm.slice_kind(get(target_state, 3), start, end, step, get(start_state, 3), get(end_state, 3), get(step_state, 3))]
      }

      return [target[::step], env, output, vm.slice_kind(get(target_state, 3), start, end, step, get(start_state, 3), get(end_state, 3), get(step_state, 3))]
    }

    if step == NONE {
      return [target[:end], env, output, vm.slice_kind(get(target_state, 3), start, end, step, get(start_state, 3), get(end_state, 3), get(step_state, 3))]
    }

    return [target[:end:step], env, output, vm.slice_kind(get(target_state, 3), start, end, step, get(start_state, 3), get(end_state, 3), get(step_state, 3))]
  }

  if end == NONE {
    if step == NONE {
      return [target[start:], env, output, vm.slice_kind(get(target_state, 3), start, end, step, get(start_state, 3), get(end_state, 3), get(step_state, 3))]
    }

    return [target[start::step], env, output, vm.slice_kind(get(target_state, 3), start, end, step, get(start_state, 3), get(end_state, 3), get(step_state, 3))]
  }

  if step == NONE {
    return [target[start:end], env, output, vm.slice_kind(get(target_state, 3), start, end, step, get(start_state, 3), get(end_state, 3), get(step_state, 3))]
  }

  return [target[start:end:step], env, output, vm.slice_kind(get(target_state, 3), start, end, step, get(start_state, 3), get(end_state, 3), get(step_state, 3))]
}

func vm.is_builtin_call(name) {
  if name == "get" {
    return true
  }

  if name == "len" {
    return true
  }

  if name == "split" {
    return true
  }

  if name == "join" {
    return true
  }

  if name == "contains" {
    return true
  }

  if name == "ord" {
    return true
  }

  if name == "char" {
    return true
  }

  if name == "pow" {
    return true
  }

  if name == "pop" {
    return true
  }

  if name == "add" {
    return true
  }

  if name == "int" {
    return true
  }

  if name == "float" {
    return true
  }

  if name == "bool" {
    return true
  }

  if name == "str" {
    return true
  }

  if name == "bytes" {
    return true
  }

  if name == "json" {
    return true
  }

  return false
}

func vm.eval_arg_values(arg_exprs, env, output, functions) {
  VTime args = []
  VTime kinds = []

  pick(arg_exprs): arg_expr {
    VTime arg_state = vm.eval_expr(arg_expr, env, output, functions)
    add(args, get(arg_state, 0))
    add(kinds, get(arg_state, 3))
    env = get(arg_state, 1)
    output = get(arg_state, 2)
  }

  return [args, env, output, kinds]
}

func vm.call_builtin(name, arg_exprs, env, output, functions) {
  if name == "add" {
    VTime list_expr = get(arg_exprs, 0)

    if ir.expr_opcode(list_expr) == ir.EXPR_LOAD {
      VTime add_name = ir.expr_value(list_expr)

      if vm.target_allows_list_mutation(env, add_name) != true {
        return [NONE, env, output, "AV"]
      }

      VTime value_state = vm.eval_expr(get(arg_exprs, 1), env, output, functions)
      VTime value = get(value_state, 0)
      env = get(value_state, 1)
      output = get(value_state, 2)
      VTime target = vm.env_get(env, add_name)
      add(target, value)
      return [NONE, vm.env_put_meta(env, add_name, target, vm.list_kind_after_add(vm.env_kind(env, add_name), get(value_state, 3)), vm.env_type(env, add_name)), output, "AV"]
    }

    return [NONE, env, output, "AV"]
  }

  if name == "pop" {
    VTime list_expr = get(arg_exprs, 0)

    if ir.expr_opcode(list_expr) == ir.EXPR_LOAD {
      VTime pop_name = ir.expr_value(list_expr)

      if vm.target_allows_list_mutation(env, pop_name) != true {
        return [NONE, env, output, "AV"]
      }

      VTime target = vm.env_get(env, pop_name)
      VTime value = pop(target)
      VTime item_kind = vm.list_item_kind(vm.env_kind(env, pop_name), len(target))
      return [value, vm.env_put_meta(env, pop_name, target, vm.list_kind_after_pop(vm.env_kind(env, pop_name)), vm.env_type(env, pop_name)), output, item_kind]
    }

    return [NONE, env, output, "AV"]
  }

  VTime args_state = vm.eval_arg_values(arg_exprs, env, output, functions)
  VTime args = get(args_state, 0)
  env = get(args_state, 1)
  output = get(args_state, 2)

  if name == "get" {
    return [get(get(args, 0), get(args, 1)), env, output, vm.max_kind(vm.list_item_kind(get(get(args_state, 3), 0), get(args, 1)), get(get(args_state, 3), 1))]
  }

  if name == "len" {
    return [len(get(args, 0)), env, output, get(get(args_state, 3), 0)]
  }

  if name == "split" {
    return [split(get(args, 0), get(args, 1)), env, output, vm.max_kind(get(get(args_state, 3), 0), get(get(args_state, 3), 1))]
  }

  if name == "join" {
    return [join(get(args, 0), get(args, 1)), env, output, vm.max_kind(get(get(args_state, 3), 0), get(get(args_state, 3), 1))]
  }

  if name == "contains" {
    return [contains(get(args, 0), get(args, 1)), env, output, vm.max_kind(get(get(args_state, 3), 0), get(get(args_state, 3), 1))]
  }

  if name == "ord" {
    return [ord(get(args, 0)), env, output, get(get(args_state, 3), 0)]
  }

  if name == "char" {
    return [char(get(args, 0)), env, output, get(get(args_state, 3), 0)]
  }

  if name == "pow" {
    VTime base = get(args, 0)
    VTime exponent = get(args, 1)
    VTime kind = vm.max_kind(get(get(args_state, 3), 0), get(get(args_state, 3), 1))

    if exponent < 0 {
      return [NONE, env, output, kind]
    }

    VTime result = 1
    VTime index = 0

    while (index < exponent) (-1) {
      result *= base
      index += 1
    }

    return [result, env, output, kind]
  }

  if name == "int" {
    return [int(get(args, 0)), env, output, get(get(args_state, 3), 0)]
  }

  if name == "float" {
    return [float(get(args, 0)), env, output, get(get(args_state, 3), 0)]
  }

  if name == "bool" {
    return [bool(get(args, 0)), env, output, get(get(args_state, 3), 0)]
  }

  if name == "str" {
    return [str(get(args, 0)), env, output, get(get(args_state, 3), 0)]
  }

  if name == "bytes" {
    return [bytes(get(args, 0)), env, output, get(get(args_state, 3), 0)]
  }

  if name == "json" {
    return [json(get(args, 0)), env, output, get(get(args_state, 3), 0)]
  }

  return [NONE, env, output, "AV"]
}

func vm.eval_unary(op, value_expr, env, output, functions) {
  VTime value_state = vm.eval_expr(value_expr, env, output, functions)
  VTime value = get(value_state, 0)
  env = get(value_state, 1)
  output = get(value_state, 2)

  if op == "-" {
    return [value - value - value, env, output, get(value_state, 3)]
  }

  if op == "not" {
    return [not value, env, output, get(value_state, 3)]
  }

  return [NONE, env, output, "AV"]
}

func vm.eval_binary(op, left_expr, right_expr, env, output, functions) {
  VTime left_state = vm.eval_expr(left_expr, env, output, functions)
  VTime left = get(left_state, 0)
  env = get(left_state, 1)
  output = get(left_state, 2)
  VTime right_state = vm.eval_expr(right_expr, env, output, functions)
  VTime right = get(right_state, 0)
  env = get(right_state, 1)
  output = get(right_state, 2)

  if op == "==" {
    return [left == right, env, output, vm.max_kind(get(left_state, 3), get(right_state, 3))]
  }

  if op == "!=" {
    return [left != right, env, output, vm.max_kind(get(left_state, 3), get(right_state, 3))]
  }

  if op == ">" {
    return [left > right, env, output, vm.max_kind(get(left_state, 3), get(right_state, 3))]
  }

  if op == "<" {
    return [left < right, env, output, vm.max_kind(get(left_state, 3), get(right_state, 3))]
  }

  if op == ">=" {
    return [left >= right, env, output, vm.max_kind(get(left_state, 3), get(right_state, 3))]
  }

  if op == "<=" {
    return [left <= right, env, output, vm.max_kind(get(left_state, 3), get(right_state, 3))]
  }

  if op == "and" {
    return [left and right, env, output, vm.max_kind(get(left_state, 3), get(right_state, 3))]
  }

  if op == "or" {
    return [left or right, env, output, vm.max_kind(get(left_state, 3), get(right_state, 3))]
  }

  if op == "+" {
    return [left + right, env, output, vm.max_kind(get(left_state, 3), get(right_state, 3))]
  }

  if op == "-" {
    return [left - right, env, output, vm.max_kind(get(left_state, 3), get(right_state, 3))]
  }

  if op == "*" {
    return [left * right, env, output, vm.max_kind(get(left_state, 3), get(right_state, 3))]
  }

  if op == "/" {
    return [left / right, env, output, vm.max_kind(get(left_state, 3), get(right_state, 3))]
  }

  return [NONE, env, output, "AV"]
}

func vm.call_func(name, arg_exprs, env, output, functions) {
  VTime function_index = vm.func_index(functions, name)

  if function_index == NONE {
    return [NONE, env, output, "AV"]
  }

  VTime params = get(vm.func_params(functions), function_index)
  VTime body = get(vm.func_bodies(functions), function_index)
  VTime args_state = vm.eval_arg_values(arg_exprs, env, output, functions)
  VTime args = get(args_state, 0)
  VTime arg_kinds = get(args_state, 3)
  env = get(args_state, 1)
  output = get(args_state, 2)

  VTime call_env = env
  VTime index = 0

  while (index < len(params)) (-1) {
    call_env = vm.env_put_meta(call_env, get(params, index), get(args, index), get(arg_kinds, index), "VTime")
    index += 1
  }

  VTime state = vm.run_ir_state(body, call_env, output, functions)
  output = get(state, 2)

  if get(state, 0) == vm.FLOW_RETURN {
    return [get(state, 3), env, output, get(state, 4)]
  }

  return [NONE, env, output, "AV"]
}

func vm.apply_assign(current, op, value) {
  if op == "=" {
    return value
  }

  if op == "+=" {
    return current + value
  }

  if op == "-=" {
    return current - value
  }

  if op == "*=" {
    return current * value
  }

  if op == "/=" {
    return current / value
  }

  return NONE
}

func vm.is_compound_assign(op) {
  return op != "="
}

func vm.type_allows_compound(value_type) {
  if value_type == "Int" {
    return true
  }

  if value_type == "Float" {
    return true
  }

  return false
}

func vm.target_allows_assignment(env, name, op) {
  VTime value_type = vm.env_type(env, name)

  if value_type == "List" {
    return false
  }

  if value_type == "VTime" {
    return true
  }

  if vm.is_compound_assign(op) {
    if vm.env_kind(env, name) != "AV" {
      return false
    }

    return vm.type_allows_compound(value_type)
  }

  return true
}

func vm.target_allows_list_mutation(env, name) {
  if vm.env_has(env, name) != true {
    return false
  }

  if vm.env_type(env, name) == "List" {
    return true
  }

  if vm.env_type(env, name) == "VTime" {
    return true
  }

  return false
}

func vm.is_av_str_target(env, name) {
  if vm.env_has(env, name) != true {
    return false
  }

  if vm.env_kind(env, name) != "AV" {
    return false
  }

  return vm.env_type(env, name) == "Str"
}

func vm.coerce_value(value, value_type) {
  if value == NONE {
    return NONE
  }

  if value_type == "Int" {
    return int(value)
  }

  if value_type == "Float" {
    return float(value)
  }

  if value_type == "Bool" {
    return bool(value)
  }

  if value_type == "Str" {
    return str(value)
  }

  if value_type == "Bytes" {
    return bytes(value)
  }

  if value_type == "Json" {
    return json(value)
  }

  return value
}

func vm.coerce_state(value_state, value_type) {
  return [vm.coerce_value(get(value_state, 0), value_type), get(value_state, 1), get(value_state, 2), get(value_state, 3)]
}

func vm.add_public_reason(output, value_state, fallback) {
  VTime next_output = get(value_state, 2)

  if vm.kind_is_public(get(value_state, 3)) != true {
    add(next_output, "DENIED")
    return next_output
  }

  VTime value = get(value_state, 0)

  if value == NONE {
    if fallback != NONE {
      add(next_output, fallback)
    }

    return next_output
  }

  add(next_output, str(value))
  return next_output
}

func vm.exec_instruction(instruction, env, output, functions) {
  VTime opcode = ir.opcode(instruction)

  if opcode == ir.OP_DECL {
    VTime decl_keyword = get(instruction, 1)
    VTime name = get(instruction, 2)

    if vm.env_has(env, name) {
      return [vm.FLOW_FAIL, env, output]
    }

    VTime value_state = vm.eval_expr(get(instruction, 3), env, output, functions)
    value_state = vm.coerce_state(value_state, vm.decl_type(decl_keyword))
    VTime value = get(value_state, 0)
    env = get(value_state, 1)
    output = get(value_state, 2)
    VTime target_kind = vm.decl_kind(decl_keyword)

    if vm.kind_allows(target_kind, get(value_state, 3)) != true {
      return [vm.FLOW_FAIL, env, output]
    }

    return [vm.FLOW_OK, vm.env_put_meta(env, name, value, target_kind, vm.decl_type(decl_keyword)), output]
  }

  if opcode == ir.OP_LIST_DECL {
    VTime name = get(instruction, 1)

    if vm.env_has(env, name) {
      return [vm.FLOW_FAIL, env, output]
    }

    VTime value_state = vm.eval_expr(get(instruction, 2), env, output, functions)
    VTime value = get(value_state, 0)
    env = get(value_state, 1)
    output = get(value_state, 2)
    return [vm.FLOW_OK, vm.env_put_meta(env, name, value, get(value_state, 3), "List"), output]
  }

  if opcode == ir.OP_VTIME_DECL {
    VTime name = get(instruction, 1)

    if vm.env_has(env, name) {
      return [vm.FLOW_FAIL, env, output]
    }

    VTime value_state = vm.eval_expr(get(instruction, 2), env, output, functions)
    VTime value = get(value_state, 0)
    env = get(value_state, 1)
    output = get(value_state, 2)
    return [vm.FLOW_OK, vm.env_put_meta(env, name, value, get(value_state, 3), "VTime"), output]
  }

  if opcode == ir.OP_INFO_ASSIGN {
    VTime type_target = get(instruction, 1)
    VTime level_target = get(instruction, 2)
    VTime source = get(instruction, 3)

    if vm.is_av_str_target(env, type_target) != true {
      return [vm.FLOW_FAIL, env, output]
    }

    if vm.is_av_str_target(env, level_target) != true {
      return [vm.FLOW_FAIL, env, output]
    }

    if vm.env_has(env, source) != true {
      return [vm.FLOW_FAIL, env, output]
    }

    if vm.env_type(env, source) == "VTime" {
      return [vm.FLOW_FAIL, env, output]
    }

    env = vm.env_put(env, type_target, vm.env_type(env, source))
    env = vm.env_put(env, level_target, vm.aggregate_kind(vm.env_kind(env, source)))
    return [vm.FLOW_OK, env, output]
  }

  if opcode == ir.OP_ASSIGN {
    VTime name = get(instruction, 1)
    VTime op = get(instruction, 2)

    if vm.env_has(env, name) != true {
      return [vm.FLOW_FAIL, env, output]
    }

    if vm.target_allows_assignment(env, name, op) != true {
      return [vm.FLOW_FAIL, env, output]
    }

    VTime value_state = vm.eval_expr(get(instruction, 3), env, output, functions)
    VTime value = get(value_state, 0)
    env = get(value_state, 1)
    output = get(value_state, 2)
    VTime current = vm.env_get(env, name)

    if op == "=" {
      VTime coerced_state = vm.coerce_state(value_state, vm.env_type(env, name))
      value = get(coerced_state, 0)
      env = get(coerced_state, 1)
      output = get(coerced_state, 2)
    }

    if vm.kind_allows(vm.env_kind(env, name), get(value_state, 3)) != true {
      return [vm.FLOW_FAIL, env, output]
    }

    return [vm.FLOW_OK, vm.env_put_meta(env, name, vm.apply_assign(current, op, value), vm.env_kind(env, name), vm.env_type(env, name)), output]
  }

  if opcode == ir.OP_EXPR {
    VTime value_state = vm.eval_expr(get(instruction, 1), env, output, functions)
    return [vm.FLOW_OK, get(value_state, 1), get(value_state, 2)]
  }

  if opcode == ir.OP_OUT {
    VTime value_state = vm.eval_expr(get(instruction, 1), env, output, functions)
    env = get(value_state, 1)
    VTime next_output = get(value_state, 2)

    if vm.kind_is_public(get(value_state, 3)) != true {
      add(next_output, "DENIED")
      return [vm.FLOW_OK, env, next_output]
    }

    add(next_output, str(get(value_state, 0)))
    return [vm.FLOW_OK, env, next_output]
  }

  if opcode == ir.OP_STOP {
    VTime value_state = vm.eval_expr(get(instruction, 1), env, output, functions)
    env = get(value_state, 1)
    return [vm.FLOW_STOP, env, vm.add_public_reason(output, value_state, NONE)]
  }

  if opcode == ir.OP_FAIL {
    VTime value_state = vm.eval_expr(get(instruction, 1), env, output, functions)
    env = get(value_state, 1)
    return [vm.FLOW_FAIL, env, vm.add_public_reason(output, value_state, "FAIL")]
  }

  if opcode == ir.OP_BREAK {
    return [vm.FLOW_BREAK, env, output]
  }

  if opcode == ir.OP_CONTINUE {
    return [vm.FLOW_CONTINUE, env, output]
  }

  if opcode == ir.OP_RETURN {
    VTime value_state = vm.eval_expr(get(instruction, 1), env, output, functions)
    return [vm.FLOW_RETURN, get(value_state, 1), get(value_state, 2), get(value_state, 0), get(value_state, 3)]
  }

  if opcode == ir.OP_SECRETUP {
    VTime name = get(instruction, 1)

    if vm.env_has(env, name) != true {
      return [vm.FLOW_FAIL, env, output]
    }

    if vm.env_type(env, name) == "VTime" {
      return [vm.FLOW_FAIL, env, output]
    }

    if vm.env_type(env, name) == "List" {
      return [vm.FLOW_FAIL, env, output]
    }

    return [vm.FLOW_OK, vm.env_put_meta(env, name, vm.env_get(env, name), vm.up_kind(vm.env_kind(env, name)), vm.env_type(env, name)), output]
  }

  if opcode == ir.OP_FUNC {
    return [vm.FLOW_OK, env, output]
  }

  if opcode == ir.OP_IF {
    VTime condition_state = vm.eval_expr(get(instruction, 1), env, output, functions)
    env = get(condition_state, 1)
    output = get(condition_state, 2)

    if get(condition_state, 0) {
      VTime state = vm.run_ir_state(get(instruction, 2), env, output, functions)
      return state
    }

    VTime state = vm.run_ir_state(get(instruction, 3), env, output, functions)
    return state
  }

  if opcode == ir.OP_WHILE {
    VTime condition = get(instruction, 1)
    VTime limit = get(instruction, 2)
    VTime body = get(instruction, 3)
    VTime iterations = 0

    while (true) (-1) {
      VTime condition_state = vm.eval_expr(condition, env, output, functions)
      env = get(condition_state, 1)
      output = get(condition_state, 2)

      if get(condition_state, 0) != true {
        break
      }

      if limit < -1 {
        return [vm.FLOW_OK, env, output]
      }

      if (limit != -1) and (iterations >= limit) {
        return [vm.FLOW_OK, env, output]
      }

      VTime state = vm.run_ir_state(body, env, output, functions)
      env = get(state, 1)
      output = get(state, 2)
      iterations += 1

      if get(state, 0) == vm.FLOW_BREAK {
        return [vm.FLOW_OK, env, output]
      }

      if get(state, 0) == vm.FLOW_CONTINUE {
        continue
      }

      if get(state, 0) != vm.FLOW_OK {
        return [get(state, 0), env, output]
      }
    }

    return [vm.FLOW_OK, env, output]
  }

  if opcode == ir.OP_PICK {
    VTime value_state = vm.eval_expr(get(instruction, 1), env, output, functions)
    VTime items = get(value_state, 0)
    env = get(value_state, 1)
    output = get(value_state, 2)
    VTime item_name = get(instruction, 2)
    VTime body = get(instruction, 3)

    pick(items): item {
      env = vm.env_put_meta(env, item_name, item, get(value_state, 3), "VTime")
      VTime state = vm.run_ir_state(body, env, output, functions)
      env = vm.env_without(get(state, 1), item_name)
      output = get(state, 2)

      if get(state, 0) == vm.FLOW_CONTINUE {
        continue
      }

      if get(state, 0) == vm.FLOW_BREAK {
        return [vm.FLOW_OK, env, output]
      }

      if get(state, 0) != vm.FLOW_OK {
        if get(state, 0) == vm.FLOW_RETURN {
          return [vm.FLOW_RETURN, env, output, get(state, 3), get(state, 4)]
        }

        return [get(state, 0), env, output]
      }
    }

    return [vm.FLOW_OK, env, output]
  }

  return [vm.FLOW_FAIL, env, output]
}

func vm.run_ir_state(program, env, output, functions) {
  pick(program): instruction {
    VTime state = vm.exec_instruction(instruction, env, output, functions)
    env = get(state, 1)
    output = get(state, 2)

    if get(state, 0) != vm.FLOW_OK {
      return state
    }
  }

  return [vm.FLOW_OK, env, output]
}

func vm.run_ir(program) {
  VTime functions = vm.collect_functions(program)

  if vm.functions_ok(functions) != true {
    return []
  }

  VTime state = vm.run_ir_state(program, vm.new_env(), [], functions)
  return get(state, 2)
}

func vm.run_ir_with_input(program, inputs) {
  VTime functions = vm.collect_functions(program)

  if vm.functions_ok(functions) != true {
    return []
  }

  VTime state = vm.run_ir_state(program, vm.new_env_with_input(inputs), [], functions)
  return get(state, 2)
}

func vm.run_source(source) {
  return vm.run_ir(ir.compile_source(source))
}

func vm.run_source_with_input(source, inputs) {
  return vm.run_ir_with_input(ir.compile_source(source), inputs)
}
