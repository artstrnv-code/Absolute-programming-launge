AVStr source = join(["AVStr public = input ASVStr hidden = secret input AVStr typ = ", char(34), char(34), " AVStr level = ", char(34), char(34), " typ, level = info(public) out typ out level out public typ, level = info(hidden) out typ out level out hidden"], "")
VTime output = vm.run_source(source)

out "VM input output:"

pick(output): line {
  out line
}

out "Done!"
