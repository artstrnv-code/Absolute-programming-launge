# Test VM bootstrap with while.

AVStr source = "AVInt x = 0 while ( x < 3 ) ( 10 ) { x += 1 out x }"
VTime output = vm.run_source(source)

out "VM while output:"
out len(output)
out get(output, 0)
out get(output, 1)
out get(output, 2)
out "Done!"
