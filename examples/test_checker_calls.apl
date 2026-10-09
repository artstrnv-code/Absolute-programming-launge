# Test APL checker function-call validation.

AVStr valid_forward = "out go() func go() { return 7 }"
AVStr builtin_ok = "out len([1, 2])"
AVStr missing_func = "out missing()"
AVStr var_as_func = "AVInt x = 1 out x()"
AVStr bad_arity = "func go(a) { return a } out go()"
AVStr missing_arg = "out len(missing)"
AVStr builtin_missing = "out len()"
AVStr builtin_extra = "out pow(2, 3, 4)"
AVStr mutating_builtin_missing = "add()"

VTime valid_report = bootstrap.run_report(valid_forward)
VTime builtin_report = bootstrap.run_report(builtin_ok)
VTime missing_func_report = bootstrap.compile_report(missing_func)
VTime var_func_report = bootstrap.compile_report(var_as_func)
VTime arity_report = bootstrap.compile_report(bad_arity)
VTime missing_arg_report = bootstrap.compile_report(missing_arg)
VTime builtin_missing_report = bootstrap.compile_report(builtin_missing)
VTime builtin_extra_report = bootstrap.compile_report(builtin_extra)
VTime mutating_builtin_report = bootstrap.compile_report(mutating_builtin_missing)

out "Checker call reports:"
out get(valid_report, 0)
out get(get(valid_report, 1), 0)
out get(builtin_report, 0)
out get(get(builtin_report, 1), 0)
out get(missing_func_report, 0)
out get(missing_func_report, 1)
out get(var_func_report, 0)
out get(var_func_report, 1)
out get(arity_report, 0)
out get(arity_report, 1)
out get(missing_arg_report, 0)
out get(missing_arg_report, 1)
out get(builtin_missing_report, 0)
out get(builtin_missing_report, 1)
out get(builtin_extra_report, 0)
out get(builtin_extra_report, 1)
out get(mutating_builtin_report, 0)
out get(mutating_builtin_report, 1)
out "Done!"
