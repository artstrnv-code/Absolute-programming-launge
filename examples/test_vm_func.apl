AVStr source = "func sum_two(left, right) { return left + right } AVInt x = sum_two(20, 22) out x"
VTime vm_output = vm.run_source(source)

out "VM function output:"
pick(vm_output): line {
  out line
}
out "Done!"
