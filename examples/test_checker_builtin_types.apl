# Test APL checker builtin argument-type validation.

List builtin_type_sources = [
  "List values = [1] out len(values) out get(values, 0) out char(65) out pow(2, 3)",
  "VTime value = 1 out len(value) out split(value, value) out pow(value, value)",
  "out len(1)",
  "out get(1, 0)",
  "out split(1, 2)",
  "out join(1, 2)",
  "out contains(1, 2)",
  "out ord(1)",
  "out char(true)",
  "out pow(true, 2)",
  "VTime value = pop(1)",
  "AVInt value = 1 add(value, 2)"
]

out "Checker builtin type reports:"

pick(builtin_type_sources): builtin_type_source {
  VTime builtin_type_report = bootstrap.compile_report(builtin_type_source)
  out get(builtin_type_report, 0)

  if get(builtin_type_report, 0) == checker.STATUS_FAIL {
    out get(builtin_type_report, 1)
  }
}

out "Done!"
