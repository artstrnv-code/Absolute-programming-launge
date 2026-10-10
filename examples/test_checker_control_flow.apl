# Test APL checker control-flow context validation.

List control_flow_sources = [
  "func choose(value) { if true { return value } return NONE } VTime result = choose(1)",
  "func work() { while (true) (1) { continue } pick([1]): item { break } return NONE }",
  "while (true) (-1) { break }",
  "return NONE",
  "if true { return NONE }",
  "break",
  "continue",
  "if true { break }",
  "func work() { break }",
  "while (true) (-2) { break }",
  "while (true) (1) { func nested() { break } }"
]

out "Checker control-flow reports:"

pick(control_flow_sources): control_flow_source {
  VTime control_flow_report = bootstrap.compile_report(control_flow_source)
  out get(control_flow_report, 0)

  if get(control_flow_report, 0) == checker.STATUS_FAIL {
    out get(control_flow_report, 1)
  }
}

out "Done!"
