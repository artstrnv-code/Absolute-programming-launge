AVStr source = "AVStr public = input ASVStr hidden = secret input AVStr missing = input out public out hidden out missing"
VTime output = vm.run_source_with_input(source, ["hello", "token"])

out "VM input stream output:"

pick(output): line {
  out line
}

out "Done!"
