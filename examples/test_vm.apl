# Test VM bootstrap.

AVStr source = "AVInt x = 41 x += 1 out x"
VTime output = vm.run_source(source)

out "VM output:"
out len(output)
out get(output, 0)
out "Done!"
