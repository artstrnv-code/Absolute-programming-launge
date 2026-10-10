# Test strict names and local parameter scope in the APL checker.

AVStr valid_names = "AVInt alpha.beta_2 = 1 func echo(value) { return value } VTime result = echo(alpha.beta_2)"
AVStr valid_shadow = join(["AVStr value = ", char(34), "global", char(34), " func increment(value) { return value + 1 } func apply(increment) { return increment(increment) } out apply(1) out value"], "")
List invalid_names = [
  "AVInt bad?name = 1",
  "AVInt .hidden = 1",
  "AVInt len = 1",
  "func bad?name(value) { return value }",
  "func duplicate(value, value) { return value }",
  "func identity(bad?name) { return bad?name }",
  "AVInt item = 1 pick([1]): item { out item }",
  "pick([1]): bad?name { out bad?name }",
  "func go() { return 1 } out go"
]

VTime valid_names_report = bootstrap.compile_report(valid_names)
VTime valid_shadow_report = bootstrap.run_report(valid_shadow)
out get(valid_names_report, 0)
out get(valid_shadow_report, 0)

pick(get(valid_shadow_report, 1)): line {
  out line
}

pick(invalid_names): invalid_name_source {
  VTime report = bootstrap.compile_report(invalid_name_source)
  out get(report, 0)

  if get(report, 0) == checker.STATUS_FAIL {
    out get(report, 1)
  }
}

out "Done!"
