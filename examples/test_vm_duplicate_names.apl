AVStr duplicate_var = join(["AVInt x = 1 out x AVStr x = ", char(34), "bad", char(34), " out x"], "")
AVStr duplicate_func = "func same() { return 1 } func same() { return 2 } out same()"
VTime var_output = vm.run_source(duplicate_var)
VTime func_output = vm.run_source(duplicate_func)

out "VM duplicate names output:"
out len(var_output)
out get(var_output, 0)
out get(var_output, 1)
out len(func_output)
out get(func_output, 0)
out "Done!"
