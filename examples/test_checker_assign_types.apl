# Test APL checker declaration and assignment type validation.

List assignment_type_sources = [
  "AVInt integer = 1 AVFloat decimal = 1.0 AVBool flag = true AVStr text = str(1) AVBytes raw = bytes(text) AVJson document = json(text) integer = 2 decimal = 2.0 flag = false text = str(2) raw = bytes(text) document = json(text)",
  "AVInt value = input value = NONE VTime dynamic = 1 dynamic = str(dynamic) value = dynamic",
  "AVInt value = str(1)",
  "AVFloat value = 1",
  "AVBool value = 1",
  "AVStr value = 1",
  "AVBytes value = str(1)",
  "AVJson value = str(1)",
  "AVInt value = 1 value = str(2)",
  "AVInt value = 1 value += 1.0"
]

out "Checker assignment type reports:"

pick(assignment_type_sources): assignment_type_source {
  VTime assignment_type_report = bootstrap.compile_report(assignment_type_source)
  out get(assignment_type_report, 0)

  if get(assignment_type_report, 0) == checker.STATUS_FAIL {
    out get(assignment_type_report, 1)
  }
}

out "Done!"
