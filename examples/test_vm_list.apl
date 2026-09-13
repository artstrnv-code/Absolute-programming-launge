AVStr source = "List items = [1, 2] add(items, 3) VTime first = get(items, 0) VTime last = pop(items) out first out last out len(items)"
VTime vm_output = vm.run_source(source)

out "VM list output:"
pick(vm_output): line {
  out line
}
out "Done!"
