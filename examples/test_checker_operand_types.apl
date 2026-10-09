# Test APL checker expression operand and control-value type validation.

List operand_type_sources = [
  "AVInt integer = 1 AVFloat decimal = 1.0 AVBool flag = true AVInt sum = integer + 2 AVFloat quotient = decimal / 2.0 AVBool equal = integer == 1 AVBool ordered = decimal >= 1.0 AVBool logical = flag and true if flag { out integer } while (integer < 2) (1) { integer += 1 } pick([integer]): item { out item }",
  "VTime value = 1 VTime other = value + true VTime index = true VTime item = value[index] VTime part = value[index:index:index] if value { out other } while (value) (1) { value = false } pick(value): picked { out picked }",
  "VTime value = -true",
  "VTime value = not 1",
  "VTime value = 1 + 1.0",
  "VTime value = bytes(1) + bytes(2)",
  "VTime value = 1 == bytes(1)",
  "VTime value = 1 < 1.0",
  "VTime value = true and 1",
  "if 1 { out 1 }",
  "while (bytes(1)) (1) { out 1 }",
  "pick(1): item { out item }",
  "VTime value = 1[0]",
  "VTime value = [1][true]",
  "VTime value = true[:]",
  "VTime value = [1][1.0:]"
]

out "Checker operand type reports:"

pick(operand_type_sources): operand_type_source {
  VTime operand_type_report = bootstrap.compile_report(operand_type_source)
  out get(operand_type_report, 0)

  if get(operand_type_report, 0) == checker.STATUS_FAIL {
    out get(operand_type_report, 1)
  }
}

out "Done!"
