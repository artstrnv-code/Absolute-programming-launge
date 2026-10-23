# APL-owned textual x86-64 assembly emitter.
# It consumes verified APLABI1 and calls a small native runtime ABI.

AVStr assembly.MAGIC = "APLASM1"
AVStr assembly.ENTRY_SYMBOL = "apl_program_entry"
AVInt assembly.RUNTIME_ABI_VERSION = 1

AVInt assembly.ARG_IMMEDIATE = 0
AVInt assembly.ARG_POINTER = 1
AVInt assembly.ARG_NULL = 2

func assembly.arg_immediate(value) {
  return [assembly.ARG_IMMEDIATE, value]
}

func assembly.arg_pointer(label) {
  return [assembly.ARG_POINTER, label]
}

func assembly.arg_null() {
  return [assembly.ARG_NULL, 0]
}

func assembly.state(code, tables) {
  return [code, tables]
}

func assembly.emit(state, text) {
  VTime code = get(state, 0)
  add(code, text)
  return assembly.state(code, get(state, 1))
}

func assembly.emit_table(state, text) {
  VTime tables = get(state, 1)
  add(tables, text)
  return assembly.state(get(state, 0), tables)
}

func assembly.is_windows(target) {
  return get(target, 0) == 0
}

func assembly.argument_register(target, index) {
  return get(get(target, 8), index)
}

func assembly.pool_label(ref) {
  return join([".Lapl_pool_", str(ref)], "")
}

func assembly.block_label(tag, label) {
  return join([".Lapl_block_", tag, "_", str(label)], "")
}

func assembly.function_symbol(slot) {
  return join(["apl_fn_", str(slot)], "")
}

func assembly.function_epilogue(slot) {
  return join([".Lapl_epilogue_fn_", str(slot)], "")
}

func assembly.target_table_label(tag, index) {
  return join([".Lapl_targets_", tag, "_", str(index)], "")
}

func assembly.param_table_label(slot) {
  return join([".Lapl_params_", str(slot)], "")
}

func assembly.render_arg(target, index, arg) {
  VTime register = assembly.argument_register(target, index)
  VTime kind = get(arg, 0)

  if kind == assembly.ARG_IMMEDIATE {
    return join(["  mov ", register, ", ", str(get(arg, 1)), "\n"], "")
  }

  if kind == assembly.ARG_POINTER {
    return join(["  lea ", register, ", [rip + ", get(arg, 1), "]\n"], "")
  }

  return join(["  xor ", register, ", ", register, "\n"], "")
}

func assembly.call_prefix(target, symbol, args) {
  VTime parts = [join(["  mov ", assembly.argument_register(target, 0), ", r12\n"], "")]
  VTime index = 0

  pick(args): arg {
    add(parts, assembly.render_arg(target, index + 1, arg))
    index += 1
  }

  add(parts, join(["  call ", symbol, "\n"], ""))
  return join(parts, "")
}

func assembly.checked_call(target, symbol, args, epilogue) {
  return join([
    assembly.call_prefix(target, symbol, args),
    "  test eax, eax\n",
    "  jne ", epilogue, "\n"
  ], "")
}

func assembly.branch_call(target, symbol, args, branch_label, epilogue) {
  return join([
    assembly.call_prefix(target, symbol, args),
    "  test eax, eax\n",
    "  js ", epilogue, "\n",
    "  jne ", branch_label, "\n"
  ], "")
}

func assembly.prologue(target, symbol, public_symbol) {
  VTime parts = []

  if public_symbol {
    add(parts, join([".globl ", symbol, "\n"], ""))

    if assembly.is_windows(target) {
      add(parts, join([".def ", symbol, "; .scl 2; .type 32; .endef\n"], ""))
    }
  }

  if assembly.is_windows(target) != true {
    add(parts, join([".type ", symbol, ", @function\n"], ""))
  }

  add(parts, ".p2align 4\n")
  add(parts, join([symbol, ":\n"], ""))
  add(parts, "  push rbp\n")
  add(parts, "  mov rbp, rsp\n")
  add(parts, "  push r12\n")

  if assembly.is_windows(target) {
    add(parts, "  sub rsp, 40\n")
  } else {
    add(parts, "  sub rsp, 8\n")
  }

  add(parts, join(["  mov r12, ", assembly.argument_register(target, 0), "\n"], ""))
  return join(parts, "")
}

func assembly.epilogue(target) {
  if assembly.is_windows(target) {
    return "  add rsp, 40\n  pop r12\n  pop rbp\n  ret\n"
  }

  return "  add rsp, 8\n  pop r12\n  pop rbp\n  ret\n"
}

func assembly.utf8_bytes(text) {
  VTime encoded = []

  pick(text): ch {
    VTime code = ord(ch)

    if code <= 127 {
      add(encoded, code)
    } else if code <= 2047 {
      VTime high = int(code / 64)
      add(encoded, 192 + high)
      add(encoded, 128 + code - high * 64)
    } else if code <= 65535 {
      VTime high = int(code / 4096)
      VTime middle = int(code / 64) - high * 64
      add(encoded, 224 + high)
      add(encoded, 128 + middle)
      add(encoded, 128 + code - int(code / 64) * 64)
    } else {
      VTime high = int(code / 262144)
      VTime next = int(code / 4096) - high * 64
      VTime middle = int(code / 64) - int(code / 4096) * 64
      add(encoded, 240 + high)
      add(encoded, 128 + next)
      add(encoded, 128 + middle)
      add(encoded, 128 + code - int(code / 64) * 64)
    }
  }

  return encoded
}

func assembly.byte_line(encoded) {
  if len(encoded) == 0 {
    return ""
  }

  VTime values = []

  pick(encoded): byte {
    add(values, str(byte))
  }

  return join(["  .byte ", join(values, ", "), "\n"], "")
}

func assembly.pool_record(entry, index) {
  VTime kind = get(entry, 0)
  VTime value = get(entry, 1)
  VTime parts = [assembly.pool_label(index), ":\n", "  .quad ", str(kind), "\n"]

  if (kind == abi.POOL_NAME) or (kind == abi.POOL_STR) {
    VTime encoded = assembly.utf8_bytes(value)
    add(parts, join(["  .quad ", str(len(encoded)), "\n"], ""))
    add(parts, assembly.byte_line(encoded))
  } else {
    add(parts, "  .quad 8\n")

    if kind == abi.POOL_FLOAT {
      add(parts, join(["  .double ", str(value), "\n"], ""))
    } else if kind == abi.POOL_BOOL {
      if value {
        add(parts, "  .quad 1\n")
      } else {
        add(parts, "  .quad 0\n")
      }
    } else {
      add(parts, join(["  .quad ", str(value), "\n"], ""))
    }
  }

  add(parts, "  .balign 8\n")
  return join(parts, "")
}

func assembly.ref_table(label, refs) {
  VTime parts = [label, ":\n"]

  if len(refs) == 0 {
    add(parts, "  .quad 0\n")
  } else {
    pick(refs): ref {
      if ref < 0 {
        add(parts, "  .quad 0\n")
      } else {
        add(parts, join(["  .quad ", assembly.pool_label(ref), "\n"], ""))
      }
    }
  }

  return join(parts, "")
}

func assembly.emit_checked(state, target, symbol, args, epilogue) {
  return assembly.emit(state, assembly.checked_call(target, symbol, args, epilogue))
}

func assembly.builtin_symbol(builtin_id) {
  return join(["apl_rt_builtin_", get(abi.BUILTINS, builtin_id)], "")
}

func assembly.emit_instruction(instruction, index, tag, target, epilogue, state) {
  VTime opcode = get(instruction, 0)

  if opcode == abi.OP_CONST {
    return assembly.emit_checked(state, target, "apl_rt_push_const", [assembly.arg_pointer(assembly.pool_label(get(instruction, 1)))], epilogue)
  }

  if opcode == abi.OP_NONE {
    return assembly.emit_checked(state, target, "apl_rt_push_none", [], epilogue)
  }

  if opcode == abi.OP_INPUT {
    return assembly.emit_checked(state, target, "apl_rt_input", [], epilogue)
  }

  if opcode == abi.OP_SECRET_INPUT {
    return assembly.emit_checked(state, target, "apl_rt_secret_input", [], epilogue)
  }

  if opcode == abi.OP_LOAD {
    return assembly.emit_checked(state, target, "apl_rt_load", [assembly.arg_pointer(assembly.pool_label(get(instruction, 1)))], epilogue)
  }

  if opcode == abi.OP_UNARY {
    return assembly.emit_checked(state, target, "apl_rt_unary", [assembly.arg_immediate(get(instruction, 1))], epilogue)
  }

  if opcode == abi.OP_BINARY {
    return assembly.emit_checked(state, target, "apl_rt_binary", [assembly.arg_immediate(get(instruction, 1))], epilogue)
  }

  if opcode == abi.OP_CALL_BUILTIN {
    VTime table_label = assembly.target_table_label(tag, index)
    state = assembly.emit_table(state, assembly.ref_table(table_label, get(instruction, 3)))
    return assembly.emit_checked(state, target, assembly.builtin_symbol(get(instruction, 1)), [assembly.arg_pointer(table_label)], epilogue)
  }

  if opcode == abi.OP_CALL_SLOT {
    return assembly.emit_checked(state, target, assembly.function_symbol(get(instruction, 1)), [], epilogue)
  }

  if opcode == abi.OP_LIST {
    return assembly.emit_checked(state, target, "apl_rt_list", [assembly.arg_immediate(get(instruction, 1))], epilogue)
  }

  if opcode == abi.OP_INDEX {
    return assembly.emit_checked(state, target, "apl_rt_index", [], epilogue)
  }

  if opcode == abi.OP_SLICE {
    return assembly.emit_checked(state, target, "apl_rt_slice", [], epilogue)
  }

  if opcode == abi.OP_SELF {
    return assembly.emit_checked(state, target, "apl_rt_self", [assembly.arg_pointer(assembly.pool_label(get(instruction, 1)))], epilogue)
  }

  if opcode == abi.OP_TAG {
    return assembly.emit_checked(state, target, "apl_rt_tag", [assembly.arg_immediate(get(instruction, 1))], epilogue)
  }

  if opcode == abi.OP_DECLARE {
    return assembly.emit_checked(state, target, "apl_rt_declare", [assembly.arg_immediate(get(instruction, 1)), assembly.arg_pointer(assembly.pool_label(get(instruction, 2)))], epilogue)
  }

  if opcode == abi.OP_DECLARE_LIST {
    return assembly.emit_checked(state, target, "apl_rt_declare_list", [assembly.arg_pointer(assembly.pool_label(get(instruction, 1)))], epilogue)
  }

  if opcode == abi.OP_DECLARE_VTIME {
    return assembly.emit_checked(state, target, "apl_rt_declare_vtime", [assembly.arg_pointer(assembly.pool_label(get(instruction, 1)))], epilogue)
  }

  if opcode == abi.OP_ASSIGN {
    return assembly.emit_checked(state, target, "apl_rt_assign", [assembly.arg_pointer(assembly.pool_label(get(instruction, 1))), assembly.arg_immediate(get(instruction, 2))], epilogue)
  }

  if opcode == abi.OP_INFO_ASSIGN {
    return assembly.emit_checked(state, target, "apl_rt_info_assign", [
      assembly.arg_pointer(assembly.pool_label(get(instruction, 1))),
      assembly.arg_pointer(assembly.pool_label(get(instruction, 2))),
      assembly.arg_pointer(assembly.pool_label(get(instruction, 3)))
    ], epilogue)
  }

  if opcode == abi.OP_DROP {
    return assembly.emit_checked(state, target, "apl_rt_drop", [], epilogue)
  }

  if opcode == abi.OP_SECRETUP {
    return assembly.emit_checked(state, target, "apl_rt_secretup", [assembly.arg_pointer(assembly.pool_label(get(instruction, 1)))], epilogue)
  }

  if opcode == abi.OP_OUT {
    return assembly.emit_checked(state, target, "apl_rt_out", [], epilogue)
  }

  if opcode == abi.OP_STOP {
    state = assembly.emit(state, assembly.call_prefix(target, "apl_rt_stop", []))
    return assembly.emit(state, join(["  jmp ", epilogue, "\n"], ""))
  }

  if opcode == abi.OP_FAIL {
    state = assembly.emit(state, assembly.call_prefix(target, "apl_rt_fail", []))
    return assembly.emit(state, join(["  jmp ", epilogue, "\n"], ""))
  }

  if opcode == abi.OP_RETURN {
    state = assembly.emit(state, assembly.call_prefix(target, "apl_rt_return", [assembly.arg_immediate(get(instruction, 1))]))
    return assembly.emit(state, join(["  jmp ", epilogue, "\n"], ""))
  }

  if opcode == abi.OP_LABEL {
    return assembly.emit(state, join([assembly.block_label(tag, get(instruction, 1)), ":\n"], ""))
  }

  if opcode == abi.OP_JUMP {
    return assembly.emit(state, join(["  jmp ", assembly.block_label(tag, get(instruction, 1)), "\n"], ""))
  }

  if opcode == abi.OP_JUMP_IF_FALSE {
    return assembly.emit(state, assembly.branch_call(target, "apl_rt_branch_false", [], assembly.block_label(tag, get(instruction, 1)), epilogue))
  }

  if opcode == abi.OP_SCOPE_ENTER {
    return assembly.emit_checked(state, target, "apl_rt_scope_enter", [], epilogue)
  }

  if opcode == abi.OP_SCOPE_EXIT {
    return assembly.emit_checked(state, target, "apl_rt_scope_exit", [], epilogue)
  }

  if opcode == abi.OP_UNWIND_JUMP {
    state = assembly.emit_checked(state, target, "apl_rt_unwind", [assembly.arg_immediate(get(instruction, 1))], epilogue)
    return assembly.emit(state, join(["  jmp ", assembly.block_label(tag, get(instruction, 2)), "\n"], ""))
  }

  if opcode == abi.OP_LOOP_ENTER {
    return assembly.emit_checked(state, target, "apl_rt_loop_enter", [assembly.arg_immediate(get(instruction, 1)), assembly.arg_immediate(get(instruction, 2))], epilogue)
  }

  if opcode == abi.OP_LOOP_CHECK {
    return assembly.emit(state, assembly.branch_call(target, "apl_rt_loop_check", [assembly.arg_immediate(get(instruction, 1))], assembly.block_label(tag, get(instruction, 2)), epilogue))
  }

  if opcode == abi.OP_LOOP_EXIT {
    return assembly.emit_checked(state, target, "apl_rt_loop_exit", [assembly.arg_immediate(get(instruction, 1))], epilogue)
  }

  if opcode == abi.OP_PICK_ENTER {
    return assembly.emit_checked(state, target, "apl_rt_pick_enter", [assembly.arg_immediate(get(instruction, 1))], epilogue)
  }

  if opcode == abi.OP_PICK_NEXT {
    return assembly.emit(state, assembly.branch_call(target, "apl_rt_pick_next", [assembly.arg_immediate(get(instruction, 1)), assembly.arg_pointer(assembly.pool_label(get(instruction, 2)))], assembly.block_label(tag, get(instruction, 3)), epilogue))
  }

  if opcode == abi.OP_PICK_EXIT {
    return assembly.emit_checked(state, target, "apl_rt_pick_exit", [assembly.arg_immediate(get(instruction, 1))], epilogue)
  }

  return state
}

func assembly.emit_code(code, tag, target, epilogue, state) {
  VTime index = 0

  pick(code): instruction {
    state = assembly.emit_instruction(instruction, index, tag, target, epilogue, state)
    index += 1
  }

  return state
}

func assembly.function_footer(target, symbol) {
  if assembly.is_windows(target) {
    return ""
  }

  return join([".size ", symbol, ", .-", symbol, "\n"], "")
}

func assembly.emit_function(function, target, state) {
  VTime slot = get(function, 0)
  VTime symbol = assembly.function_symbol(slot)
  VTime epilogue = assembly.function_epilogue(slot)
  VTime params = get(function, 2)
  VTime param_table = assembly.param_table_label(slot)
  state = assembly.emit_table(state, assembly.ref_table(param_table, params))
  state = assembly.emit(state, assembly.prologue(target, symbol, false))
  state = assembly.emit_checked(state, target, "apl_rt_function_enter", [assembly.arg_pointer(param_table), assembly.arg_immediate(len(params))], epilogue)
  state = assembly.emit_code(get(function, 3), join(["fn", str(slot)], ""), target, epilogue, state)
  state = assembly.emit(state, join([epilogue, ":\n"], ""))
  state = assembly.emit(state, assembly.epilogue(target))
  return assembly.emit(state, assembly.function_footer(target, symbol))
}

func assembly.emit_functions(functions, target, state) {
  pick(functions): function {
    state = assembly.emit_function(function, target, state)
  }

  return state
}

func assembly.data_section(target, pool, tables) {
  VTime parts = []

  if assembly.is_windows(target) {
    add(parts, ".section .rdata,\"dr\"\n")
  } else {
    add(parts, ".section .rodata\n")
  }

  add(parts, ".balign 8\n")
  VTime index = 0

  pick(pool): entry {
    add(parts, assembly.pool_record(entry, index))
    index += 1
  }

  pick(tables): table {
    add(parts, table)
    add(parts, "  .balign 8\n")
  }

  return join(parts, "")
}

func assembly.emit_image_report(image) {
  if abi.image_is_valid(image) != true {
    return [vm.FLOW_FAIL, "invalid native ABI image"]
  }

  VTime target = get(image, 1)
  VTime entry_epilogue = ".Lapl_epilogue_entry"
  VTime state = assembly.state([], [])
  state = assembly.emit(state, join(["# ", assembly.MAGIC, " runtime-abi=", str(assembly.RUNTIME_ABI_VERSION), " target=", get(target, 1), "\n.intel_syntax noprefix\n.text\n"], ""))
  state = assembly.emit(state, assembly.prologue(target, assembly.ENTRY_SYMBOL, true))
  state = assembly.emit_code(get(image, 3), "entry", target, entry_epilogue, state)
  state = assembly.emit(state, join([entry_epilogue, ":\n"], ""))
  state = assembly.emit(state, assembly.call_prefix(target, "apl_rt_finish", []))
  state = assembly.emit(state, assembly.epilogue(target))
  state = assembly.emit(state, assembly.function_footer(target, assembly.ENTRY_SYMBOL))
  state = assembly.emit_functions(get(image, 4), target, state)

  VTime parts = [join(get(state, 0), ""), assembly.data_section(target, get(image, 2), get(state, 1))]

  if assembly.is_windows(target) != true {
    add(parts, ".section .note.GNU-stack,\"\",@progbits\n")
  }

  return [vm.FLOW_OK, join(parts, "")]
}

func assembly.emit_encoded_report(encoded) {
  VTime decoded = abi.decode_image_report(encoded)

  if get(decoded, 0) != vm.FLOW_OK {
    return decoded
  }

  return assembly.emit_image_report(get(decoded, 1))
}
