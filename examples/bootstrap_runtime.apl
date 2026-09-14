AVStr source = "AVStr public = input ASVStr secret = secret input AVInt age = input List values = [age, 2, [3, 4], secret:ASV] func inc(x) { return x + 1 } VTime next = inc(age) VTime first = values[0] out public out next out first out secret out values out len(values) AVStr missing = input out missing"
VTime output = vm.run_source_with_input(source, ["hello", "token", "41"])

out "Bootstrap runtime output:"

pick(output): line {
  out line
}

out "Done!"
