# Test lexer - simple test

AVStr source = join(["AVInt x += 42 out ", char(34), "ok", char(34), " # ignored"], "")
VTime tokens = lexer.tokenize(source)

out "Tokens:"
out len(tokens)

VTime i = 0
while (i < len(tokens)) (-1) {
  VTime token = get(tokens, i)
  out lexer.token_kind(token)
  out lexer.token_value(token)
  i = i + 1
}

out "Done!"
