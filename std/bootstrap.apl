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

func bootstrap.module_report(source) {
  VTime compiled = bootstrap.compile_report(source)

  if get(compiled, 0) != vm.FLOW_OK {
    return compiled
  }

  return artifact.encode_module_report(get(compiled, 1))
}

func bootstrap.module(source) {
  return get(bootstrap.module_report(source), 1)
}

func bootstrap.modules_report(sources) {
  VTime combined = []
  VTime symbols = []

  pick(sources): source {
    VTime ast = bootstrap.ast(source)
    VTime checked = checker.validate_report_with_symbols(ast, symbols)

    if get(checked, 0) != checker.STATUS_OK {
      return [vm.FLOW_FAIL, get(checked, 1)]
    }

    VTime extension = ir.compile_ast(get(checked, 1))

    if bootstrap.ir_has_error(extension) {
      return [vm.FLOW_FAIL, bootstrap.ir_error(extension)]
    }

    pick(extension): instruction {
      add(combined, instruction)
    }

    symbols = artifact.program_symbols(combined)
  }

  return artifact.encode_module_report(combined)
}

func bootstrap.modules(sources) {
  return get(bootstrap.modules_report(sources), 1)
}

func bootstrap.load_module_report(encoded) {
  return artifact.decode_module_report(encoded)
}

func bootstrap.load_module(encoded) {
  return get(bootstrap.load_module_report(encoded), 1)
}

func bootstrap.extend_module_report(encoded_module, source) {
  VTime module_report = bootstrap.load_module_report(encoded_module)

  if get(module_report, 0) != vm.FLOW_OK {
    return module_report
  }

  VTime module = get(module_report, 1)
  VTime ast = bootstrap.ast(source)
  VTime checked = checker.validate_report_with_symbols(ast, artifact.module_symbols(module))

  if get(checked, 0) != checker.STATUS_OK {
    return [vm.FLOW_FAIL, get(checked, 1)]
  }

  VTime extension = ir.compile_ast(get(checked, 1))

  if bootstrap.ir_has_error(extension) {
    return [vm.FLOW_FAIL, bootstrap.ir_error(extension)]
  }

  VTime combined = artifact.module_program(module)[:]

  pick(extension): instruction {
    add(combined, instruction)
  }

  return artifact.encode_module_report(combined)
}

func bootstrap.extend_module(encoded_module, source) {
  return get(bootstrap.extend_module_report(encoded_module, source), 1)
}

func bootstrap.linked_artifact_with_module_report(encoded_module, source) {
  VTime module_report = bootstrap.load_module_report(encoded_module)

  if get(module_report, 0) != vm.FLOW_OK {
    return module_report
  }

  VTime module = get(module_report, 1)
  VTime ast = bootstrap.ast(source)
  VTime checked = checker.validate_report_with_symbols(ast, artifact.module_symbols(module))

  if get(checked, 0) != checker.STATUS_OK {
    return [vm.FLOW_FAIL, get(checked, 1)]
  }

  VTime user_program = ir.compile_ast(get(checked, 1))

  if bootstrap.ir_has_error(user_program) {
    return [vm.FLOW_FAIL, bootstrap.ir_error(user_program)]
  }

  VTime combined = artifact.module_program(module)[:]

  pick(user_program): instruction {
    add(combined, instruction)
  }

  VTime loaded = vm.load_ir_report(combined)

  if get(loaded, 0) != vm.FLOW_OK {
    return loaded
  }

  return artifact.encode_loaded_report(get(loaded, 1))
}

func bootstrap.linked_artifact_with_module(encoded_module, source) {
  return get(bootstrap.linked_artifact_with_module_report(encoded_module, source), 1)
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

func bootstrap.artifact_image_report(source) {
  VTime compiled = bootstrap.compile_report(source)

  if get(compiled, 0) != vm.FLOW_OK {
    return compiled
  }

  return [vm.FLOW_OK, ["APLIR1", get(compiled, 1)]]
}

func bootstrap.artifact_image(source) {
  return get(bootstrap.artifact_image_report(source), 1)
}

func bootstrap.artifact_image_is_valid(image) {
  if len(image) < 2 {
    return false
  }

  return get(image, 0) == "APLIR1"
}

func bootstrap.run_artifact_image(image) {
  return get(bootstrap.run_artifact_image_report(image), 1)
}

func bootstrap.run_artifact_image_report(image) {
  VTime loaded = bootstrap.load_artifact_image_report(image)

  if get(loaded, 0) != vm.FLOW_OK {
    return loaded
  }

  return bootstrap.run_loaded_image_report(get(loaded, 1))
}

func bootstrap.run_artifact_image_with_input(image, inputs) {
  return get(bootstrap.run_artifact_image_with_input_report(image, inputs), 1)
}

func bootstrap.run_artifact_image_with_input_report(image, inputs) {
  VTime loaded = bootstrap.load_artifact_image_report(image)

  if get(loaded, 0) != vm.FLOW_OK {
    return loaded
  }

  return bootstrap.run_loaded_image_with_input_report(get(loaded, 1), inputs)
}

func bootstrap.load_artifact_image_report(image) {
  if bootstrap.artifact_image_is_valid(image) != true {
    return [vm.FLOW_FAIL, ["invalid artifact image"]]
  }

  return vm.load_ir_report(get(image, 1))
}

func bootstrap.load_artifact_image(image) {
  return get(bootstrap.load_artifact_image_report(image), 1)
}

func bootstrap.loaded_image_report(source) {
  VTime image_report = bootstrap.artifact_image_report(source)

  if get(image_report, 0) != vm.FLOW_OK {
    return image_report
  }

  return bootstrap.load_artifact_image_report(get(image_report, 1))
}

func bootstrap.loaded_image(source) {
  return get(bootstrap.loaded_image_report(source), 1)
}

func bootstrap.linked_artifact_report(source) {
  VTime loaded = bootstrap.loaded_image_report(source)

  if get(loaded, 0) != vm.FLOW_OK {
    return loaded
  }

  return artifact.encode_loaded_report(get(loaded, 1))
}

func bootstrap.linked_artifact(source) {
  return get(bootstrap.linked_artifact_report(source), 1)
}

func bootstrap.load_linked_artifact_report(encoded) {
  return artifact.decode_loaded_report(encoded)
}

func bootstrap.load_linked_artifact(encoded) {
  return get(bootstrap.load_linked_artifact_report(encoded), 1)
}

func bootstrap.run_linked_artifact_report(encoded) {
  VTime loaded = bootstrap.load_linked_artifact_report(encoded)

  if get(loaded, 0) != vm.FLOW_OK {
    return loaded
  }

  return bootstrap.run_loaded_image_report(get(loaded, 1))
}

func bootstrap.run_linked_artifact(encoded) {
  return get(bootstrap.run_linked_artifact_report(encoded), 1)
}

func bootstrap.run_linked_artifact_with_input_report(encoded, inputs) {
  VTime loaded = bootstrap.load_linked_artifact_report(encoded)

  if get(loaded, 0) != vm.FLOW_OK {
    return loaded
  }

  return bootstrap.run_loaded_image_with_input_report(get(loaded, 1), inputs)
}

func bootstrap.run_linked_artifact_with_input(encoded, inputs) {
  return get(bootstrap.run_linked_artifact_with_input_report(encoded, inputs), 1)
}

func bootstrap.run_loaded_image(loaded) {
  return get(bootstrap.run_loaded_image_report(loaded), 1)
}

func bootstrap.run_loaded_image_report(loaded) {
  return vm.run_loaded_report(loaded)
}

func bootstrap.run_loaded_image_with_input(loaded, inputs) {
  return get(bootstrap.run_loaded_image_with_input_report(loaded, inputs), 1)
}

func bootstrap.run_loaded_image_with_input_report(loaded, inputs) {
  return vm.run_loaded_with_input_report(loaded, inputs)
}

func bootstrap.run(source) {
  return get(bootstrap.run_report(source), 1)
}

func bootstrap.run_report(source) {
  VTime loaded = bootstrap.loaded_image_report(source)

  if get(loaded, 0) != vm.FLOW_OK {
    return [get(loaded, 0), [get(loaded, 1)]]
  }

  return bootstrap.run_loaded_image_report(get(loaded, 1))
}

func bootstrap.run_with_input(source, inputs) {
  return get(bootstrap.run_with_input_report(source, inputs), 1)
}

func bootstrap.run_with_input_report(source, inputs) {
  VTime loaded = bootstrap.loaded_image_report(source)

  if get(loaded, 0) != vm.FLOW_OK {
    return [get(loaded, 0), [get(loaded, 1)]]
  }

  return bootstrap.run_loaded_image_with_input_report(get(loaded, 1), inputs)
}
