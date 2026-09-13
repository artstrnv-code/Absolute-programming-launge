# Test parser bootstrap.

AVStr source = "AVInt x = 42 out x x += 1"
VTime statements = parser.parse_source(source)

out "Statements:"
out len(statements)

VTime i = 0
while (i < len(statements)) (-1) {
  VTime statement = get(statements, i)
  out parser.node_kind(statement)
  i += 1
}

VTime decl = get(statements, 0)
VTime literal = get(decl, 3)

out get(decl, 1)
out get(decl, 2)
out parser.expr_kind(literal)
out parser.expr_value(literal)
out "Done!"
