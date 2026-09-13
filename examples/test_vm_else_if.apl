AVStr source = "AVInt x = 2 if x == 1 { out 1 } else if x == 2 { out 42 } else { out 0 }"
VTime output = vm.run_source(source)

out "VM else-if output:"
out len(output)
out get(output, 0)
out "Done!"
