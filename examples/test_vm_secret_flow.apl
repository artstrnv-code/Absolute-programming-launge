AVStr source = join(["ASVStr token = ", char(34), "root", char(34), " VTime tmp = token List items = [token] out tmp out token == token out items out get(items, 0)"], "")
VTime vm_output = vm.run_source(source)

out "VM secret flow output:"
pick(vm_output): line {
  out line
}
out "Done!"
