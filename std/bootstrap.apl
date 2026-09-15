# APL bootstrap facade.
# This module exposes the APL-owned source -> tokens -> AST -> IR -> VM pipeline
# as one stable namespace for compiled APL programs.

func bootstrap.tokens(source) {
  return lexer.tokenize(source)
}

func bootstrap.ast(source) {
  return parser.parse_source(source)
}

func bootstrap.ir(source) {
  return ir.compile_ast(bootstrap.ast(source))
}

func bootstrap.ir_has_error(program) {
  pick(program): instruction {
    if ir.opcode(instruction) == ir.OP_ERROR {
      return true
    }
  }

  return false
}

func bootstrap.ir_error(program) {
  pick(program): instruction {
    if ir.opcode(instruction) == ir.OP_ERROR {
      return get(instruction, 1)
    }
  }

  return NONE
}

func bootstrap.compile_report(source) {
  VTime ast = bootstrap.ast(source)
  VTime checked = checker.validate_report(ast)

  if get(checked, 0) != checker.STATUS_OK {
    return [vm.FLOW_FAIL, get(checked, 1)]
  }

  VTime program = ir.compile_ast(get(checked, 1))

  if bootstrap.ir_has_error(program) {
    return [vm.FLOW_FAIL, bootstrap.ir_error(program)]
  }

  return [vm.FLOW_OK, program]
}

func bootstrap.artifact_report(source) {
  VTime compiled = bootstrap.compile_report(source)

  if get(compiled, 0) != vm.FLOW_OK {
    return compiled
  }

  return [vm.FLOW_OK, join(["APLIR1:", str(get(compiled, 1))], "")]
}

func bootstrap.artifact(source) {
  return get(bootstrap.artifact_report(source), 1)
}

func bootstrap.run(source) {
  return get(bootstrap.run_report(source), 1)
}

func bootstrap.run_report(source) {
  VTime compiled = bootstrap.compile_report(source)

  if get(compiled, 0) != vm.FLOW_OK {
    return [get(compiled, 0), [get(compiled, 1)]]
  }

  return vm.run_ir_report(get(compiled, 1))
}

func bootstrap.run_with_input(source, inputs) {
  return get(bootstrap.run_with_input_report(source, inputs), 1)
}

func bootstrap.run_with_input_report(source, inputs) {
  VTime compiled = bootstrap.compile_report(source)

  if get(compiled, 0) != vm.FLOW_OK {
    return [get(compiled, 0), [get(compiled, 1)]]
  }

  return vm.run_ir_with_input_report(get(compiled, 1), inputs)
}
