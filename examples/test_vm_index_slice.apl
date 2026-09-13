AVStr source = join(["List items = [1, 2, 3, 4] VTime first = items[0] VTime tail = items[1:] AVStr text = ", char(34), "APL", char(34), " out first out get(tail, 0) out get(tail, 2) out text[::-1]"], "")
VTime vm_output = vm.run_source(source)

out "VM index/slice output:"
pick(vm_output): line {
  out line
}
out "Done!"
