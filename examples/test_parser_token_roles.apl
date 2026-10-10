# Test strict token roles in the APL-written lexer and parser.

VTime quote = char(34)
List invalid_sources = [
  join(["out ", quote, "hello"], ""),
  join(["AVInt ", quote, "x", quote, " = 1"], ""),
  join(["List ", quote, "items", quote, " = []"], ""),
  join(["VTime ", quote, "temp", quote, " = 1"], ""),
  join(["func ", quote, "f", quote, "() { return 1 }"], ""),
  join(["func f(", quote, "x", quote, ") { return 1 }"], ""),
  "func f(a b) { return a }",
  "func f(a,) { return a }",
  join(["pick([1]): ", quote, "item", quote, " { out 1 }"], ""),
  join(["secretup(", quote, "value", quote, ")"], ""),
  join(["type, ", quote, "level", quote, " = info(source)"], ""),
  join(["type, level = info(", quote, "source", quote, ")"], ""),
  join([quote, "if", quote, " true { out 1 }"], ""),
  join(["VTime value = 1 ", quote, "+", quote, " 2"], ""),
  join(["VTime value = 1 ", quote, "==", quote, " 1"], ""),
  join(["VTime value = true ", quote, "and", quote, " false"], ""),
  join(["AVInt x = 1 VTime value = x ", quote, "=self=", quote], ""),
  join(["List values = [1 ", quote, ":", quote, " ASV]"], ""),
  join(["VTime value = secret ", quote, "input", quote], "")
]

out "Parser token-role reports:"

pick(invalid_sources): source {
  VTime report = bootstrap.compile_report(source)
  out get(report, 0)
  out get(report, 1)
}

out "Done!"
