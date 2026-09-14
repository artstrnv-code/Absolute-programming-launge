# Test APL bootstrap facade.

AVStr source = "AVInt x = input x += 1 out x"
VTime tokens = bootstrap.tokens(source)
VTime ast = bootstrap.ast(source)
VTime program = bootstrap.ir(source)
VTime output = bootstrap.run_with_input(source, ["4"])

out "Bootstrap facade:"
out len(tokens)
out parser.node_kind(get(ast, 0))
out ir.opcode(get(program, 0))

pick(output): line {
  out line
}

out "Done!"
