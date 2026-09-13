AVStr source = "AVInt good = input AVInt bad = input AVFloat ratio = input AVBool flag = input AVStr text = input AVInt assigned = 0 assigned = input out good out bad out ratio out flag out text out assigned"
VTime output = vm.run_source_with_input(source, ["42", "oops", "2.5", "true", "abc", "7"])

out "VM typed input output:"

pick(output): line {
  out line
}

out "Done!"
