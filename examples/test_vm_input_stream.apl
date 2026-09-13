AVStr source = "AVStr public = input ASVStr secret = secret input AVStr missing = input out public out secret out missing"
VTime output = vm.run_source_with_input(source, ["hello", "token"])

out "VM input stream output:"

pick(output): line {
  out line
}

out "Done!"
