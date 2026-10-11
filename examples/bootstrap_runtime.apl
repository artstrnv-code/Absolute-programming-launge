AVStr source = "AVStr public = input ASVStr hidden = secret input AVInt age = input List values = [age, 2, [3, 4]] func inc(x) { return x + 1 } VTime next = inc(age) VTime first = values[0] out public out \"\\n\" out next out \"\\n\" out first out \"\\n\" AVStr missing = input out missing out \"\\n\""
VTime output = bootstrap.run_with_input(source, ["hello", "token", "41"])

out "Bootstrap runtime output:\n"

pick(output): line {
  out line
}

out "Done!\n"
