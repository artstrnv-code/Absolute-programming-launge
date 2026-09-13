AVStr source = join(["AVStr typ = ", char(34), char(34), " AVStr level = ", char(34), char(34), " SASVStr master = ", char(34), "root", char(34), " typ, level = info(master) out typ out level out master"], "")
VTime vm_output = vm.run_source(source)

out "VM security output:"
pick(vm_output): line {
  out line
}
out "Done!"
