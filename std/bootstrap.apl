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

func bootstrap.compile_with_symbols_report(source) {
  VTime ast = bootstrap.ast(source)
  VTime checked = checker.validate_report(ast)

  if get(checked, 0) != checker.STATUS_OK {
    return [vm.FLOW_FAIL, get(checked, 1)]
  }

  VTime program = ir.compile_ast(get(checked, 1))

  if bootstrap.ir_has_error(program) {
    return [vm.FLOW_FAIL, bootstrap.ir_error(program)]
  }

  return [vm.FLOW_OK, program, get(checked, 2), get(checked, 3), get(checked, 4)]
}

func bootstrap.compile_report(source) {
  VTime compiled = bootstrap.compile_with_symbols_report(source)

  if get(compiled, 0) != vm.FLOW_OK {
    return compiled
  }

  return [vm.FLOW_OK, get(compiled, 1)]
}

func bootstrap.module_report(source) {
  VTime compiled = bootstrap.compile_with_symbols_report(source)

  if get(compiled, 0) != vm.FLOW_OK {
    return compiled
  }

  return artifact.encode_module_with_symbols_report(get(compiled, 1), get(compiled, 2))
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

    symbols = get(checked, 2)
  }

  return artifact.encode_module_with_symbols_report(combined, symbols)
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

  return artifact.encode_module_with_symbols_report(combined, get(checked, 2))
}

func bootstrap.extend_module(encoded_module, source) {
  return get(bootstrap.extend_module_report(encoded_module, source), 1)
}

func bootstrap.library_exclusions(library, exclusions) {
  VTime symbols = []

  pick(exclusions): exclusion {
    if get(exclusion, 0) == library {
      add(symbols, get(exclusion, 1))
    }
  }

  return symbols
}

func bootstrap.library_manifest(includes, exclusions) {
  VTime libraries = [["base", bootstrap.library_exclusions("base", exclusions)]]

  pick(includes): library {
    if library != "base" {
      add(libraries, [library, bootstrap.library_exclusions(library, exclusions)])
    }
  }

  return [linker.MANIFEST_MAGIC, libraries]
}

func bootstrap.encode_loaded_with_composition_report(loaded, includes, exclusions) {
  VTime manifested = artifact.with_library_manifest_report(loaded, bootstrap.library_manifest(includes, exclusions))

  if get(manifested, 0) != vm.FLOW_OK {
    return manifested
  }

  return artifact.encode_loaded_report(get(manifested, 1))
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

  return bootstrap.encode_loaded_with_composition_report(get(loaded, 1), get(checked, 3), get(checked, 4))
}

func bootstrap.linked_artifact_with_module(encoded_module, source) {
  return get(bootstrap.linked_artifact_with_module_report(encoded_module, source), 1)
}

func bootstrap.named_library_index(libraries, name) {
  VTime index = 0

  while (index < len(libraries)) (-1) {
    if get(get(libraries, index), 0) == name {
      return index
    }

    index += 1
  }

  return NONE
}

func bootstrap.decode_named_libraries_report(encoded_libraries) {
  VTime libraries = []
  VTime catalog = []

  pick(encoded_libraries): encoded_library {
    if len(encoded_library) != 2 {
      return [vm.FLOW_FAIL, "invalid named library entry"]
    }

    VTime name = get(encoded_library, 0)

    if verifier.is_str(name) != true {
      return [vm.FLOW_FAIL, "invalid library name"]
    }

    if checker.composition_name_is_valid(name) != true {
      return [vm.FLOW_FAIL, join(["invalid library name `", name, "`"], "")]
    }

    if name == "base" {
      return [vm.FLOW_FAIL, "named library cannot replace `base`"]
    }

    if bootstrap.named_library_index(libraries, name) != NONE {
      return [vm.FLOW_FAIL, join(["duplicate library `", name, "`"], "")]
    }

    VTime module_report = bootstrap.load_module_report(get(encoded_library, 1))

    if get(module_report, 0) != vm.FLOW_OK {
      return [vm.FLOW_FAIL, join(["invalid library `", name, "`: ", get(module_report, 1)], "")]
    }

    VTime module = get(module_report, 1)
    add(libraries, [name, module])
    add(catalog, [name, artifact.module_symbols(module)])
  }

  return [vm.FLOW_OK, libraries, catalog]
}

func bootstrap.linked_artifact_from_library_catalog_report(base_program, base_symbols, libraries, catalog, source) {
  VTime ast = bootstrap.ast(source)
  VTime checked = checker.validate_report_with_library_symbols(ast, base_symbols, catalog)

  if get(checked, 0) != checker.STATUS_OK {
    return [vm.FLOW_FAIL, get(checked, 1)]
  }

  VTime user_program = ir.compile_ast(get(checked, 1))

  if bootstrap.ir_has_error(user_program) {
    return [vm.FLOW_FAIL, bootstrap.ir_error(user_program)]
  }

  VTime combined = base_program[:]

  pick(get(checked, 3)): library_name {
    if library_name != "base" {
      VTime library = get(libraries, bootstrap.named_library_index(libraries, library_name))

      pick(artifact.module_program(get(library, 1))): instruction {
        add(combined, instruction)
      }
    }
  }

  pick(user_program): instruction {
    add(combined, instruction)
  }

  VTime loaded = vm.load_ir_report(combined)

  if get(loaded, 0) != vm.FLOW_OK {
    return loaded
  }

  return bootstrap.encode_loaded_with_composition_report(get(loaded, 1), get(checked, 3), get(checked, 4))
}

func bootstrap.linked_artifact_with_libraries_report(base_source, encoded_libraries, source) {
  VTime base = bootstrap.compile_with_symbols_report(base_source)

  if get(base, 0) != vm.FLOW_OK {
    return base
  }

  VTime decoded = bootstrap.decode_named_libraries_report(encoded_libraries)

  if get(decoded, 0) != vm.FLOW_OK {
    return decoded
  }

  return bootstrap.linked_artifact_from_library_catalog_report(get(base, 1), get(base, 2), get(decoded, 1), get(decoded, 2), source)
}

func bootstrap.linked_artifact_with_libraries(base_source, encoded_libraries, source) {
  return get(bootstrap.linked_artifact_with_libraries_report(base_source, encoded_libraries, source), 1)
}

func bootstrap.linked_artifact_with_module_and_libraries_report(encoded_base_module, encoded_libraries, source) {
  VTime base_report = bootstrap.load_module_report(encoded_base_module)

  if get(base_report, 0) != vm.FLOW_OK {
    return base_report
  }

  VTime decoded = bootstrap.decode_named_libraries_report(encoded_libraries)

  if get(decoded, 0) != vm.FLOW_OK {
    return decoded
  }

  VTime base = get(base_report, 1)
  return bootstrap.linked_artifact_from_library_catalog_report(artifact.module_program(base), artifact.module_symbols(base), get(decoded, 1), get(decoded, 2), source)
}

func bootstrap.linked_artifact_with_module_and_libraries(encoded_base_module, encoded_libraries, source) {
  return get(bootstrap.linked_artifact_with_module_and_libraries_report(encoded_base_module, encoded_libraries, source), 1)
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

func bootstrap.loaded_image_with_composition_report(source) {
  VTime compiled = bootstrap.compile_with_symbols_report(source)

  if get(compiled, 0) != vm.FLOW_OK {
    return compiled
  }

  VTime loaded = vm.load_ir_report(get(compiled, 1))

  if get(loaded, 0) != vm.FLOW_OK {
    return loaded
  }

  return [vm.FLOW_OK, get(loaded, 1), get(compiled, 3), get(compiled, 4)]
}

func bootstrap.loaded_image_report(source) {
  VTime loaded = bootstrap.loaded_image_with_composition_report(source)

  if get(loaded, 0) != vm.FLOW_OK {
    return loaded
  }

  return [vm.FLOW_OK, get(loaded, 1)]
}

func bootstrap.loaded_image(source) {
  return get(bootstrap.loaded_image_report(source), 1)
}

func bootstrap.linked_artifact_report(source) {
  VTime loaded = bootstrap.loaded_image_with_composition_report(source)

  if get(loaded, 0) != vm.FLOW_OK {
    return loaded
  }

  return bootstrap.encode_loaded_with_composition_report(get(loaded, 1), get(loaded, 2), get(loaded, 3))
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

func bootstrap.linked_artifact_manifest_report(encoded) {
  VTime loaded = bootstrap.load_linked_artifact_report(encoded)

  if get(loaded, 0) != vm.FLOW_OK {
    return loaded
  }

  return [vm.FLOW_OK, artifact.loaded_library_manifest(get(loaded, 1))]
}

func bootstrap.linked_artifact_manifest(encoded) {
  return get(bootstrap.linked_artifact_manifest_report(encoded), 1)
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
