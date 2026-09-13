# Test VM bootstrap with if.

AVStr source = "AVInt x = 1 if x == 1 { x += 41 out x } if x != 42 { out 0 }"
VTime output = vm.run_source(source)

out "VM if output:"
out len(output)
out get(output, 0)
out "Done!"
