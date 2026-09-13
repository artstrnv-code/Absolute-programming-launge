# Test IR bootstrap.

AVStr source = "AVInt x = 42 out x x += 1"
VTime program = ir.compile_source(source)

out "Instructions:"
out len(program)

VTime i = 0
while (i < len(program)) (-1) {
  VTime instruction = get(program, i)
  out ir.opcode(instruction)
  i += 1
}

VTime decl = get(program, 0)
VTime literal = get(decl, 3)

out get(decl, 1)
out get(decl, 2)
out ir.expr_opcode(literal)
out ir.expr_value(literal)
out "Done!"
