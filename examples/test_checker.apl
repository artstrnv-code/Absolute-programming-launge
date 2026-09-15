# Test APL checker bootstrap.

AVStr ok_source = "AVInt x = 1 List items = [x] func done() { return x } out done()"
AVStr duplicate_var = "AVInt x = 1 AVStr x = 2 out x"
AVStr duplicate_func = "func go() { return 1 } func go() { return 2 } out go()"
AVStr nested_duplicate = "AVInt x = 1 if true { AVInt x = 2 } out x"
AVStr nested_func_duplicate = "func go() { return 1 } if true { func go() { return 2 } } out go()"

VTime ok_report = bootstrap.compile_report(ok_source)
VTime var_report = bootstrap.compile_report(duplicate_var)
VTime func_report = bootstrap.compile_report(duplicate_func)
VTime nested_report = bootstrap.compile_report(nested_duplicate)
VTime nested_func_report = bootstrap.compile_report(nested_func_duplicate)

out "Checker reports:"
out get(ok_report, 0)
out ir.opcode(get(get(ok_report, 1), 0))
out get(var_report, 0)
out get(var_report, 1)
out get(func_report, 0)
out get(func_report, 1)
out get(nested_report, 0)
out get(nested_report, 1)
out get(nested_func_report, 0)
out get(nested_func_report, 1)
out "Done!"
