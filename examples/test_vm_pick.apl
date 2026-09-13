AVStr source = "List items = [1, 2, 3, 4] AVInt total = 0 pick(items): item { if item == 2 { continue } if item == 4 { break } total += item } out total out item"
VTime vm_output = vm.run_source(source)

out "VM pick output:"
pick(vm_output): line {
  out line
}
out "Done!"
