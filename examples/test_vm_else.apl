# Test VM bootstrap with else.

AVStr source = "AVInt x = 1 if x == 2 { out 0 } else { x += 41 out x }"
VTime output = vm.run_source(source)

out "VM else output:"
out len(output)
out get(output, 0)
out "Done!"
