AVStr source = join(["AVInt x = 4 out x =self= x += 1 out x =self= ASVStr token = ", char(34), "root", char(34), " out token =self="], "")
VTime output = vm.run_source(source)

out "VM self output:"

pick(output): line {
  out line
}

out "Done!"
