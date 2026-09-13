AVStr source = join(["AVStr public = input ASVStr secret = secret input AVStr typ = ", char(34), char(34), " AVStr level = ", char(34), char(34), " typ, level = info(public) out typ out level out public typ, level = info(secret) out typ out level out secret"], "")
VTime output = vm.run_source(source)

out "VM input output:"

pick(output): line {
  out line
}

out "Done!"
