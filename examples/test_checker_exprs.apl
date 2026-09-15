# Test APL checker expression validation.

AVStr valid = "AVInt x = 1 VTime y = x + 1 out y"
AVStr missing_out = "out missing"
AVStr missing_decl = "AVInt x = missing"
AVStr missing_assignment_value = "AVInt x = 1 x = missing"
AVStr missing_list = "List items = [missing]"
AVStr missing_if = "if missing { out 1 }"
AVStr bad_self = "VTime temp = 1 out temp =self="
AVStr missing_slice = "List items = [1] VTime part = items[:missing]"

VTime valid_report = bootstrap.run_report(valid)
VTime out_report = bootstrap.compile_report(missing_out)
VTime decl_report = bootstrap.compile_report(missing_decl)
VTime assign_report = bootstrap.compile_report(missing_assignment_value)
VTime list_report = bootstrap.compile_report(missing_list)
VTime if_report = bootstrap.compile_report(missing_if)
VTime self_report = bootstrap.compile_report(bad_self)
VTime slice_report = bootstrap.compile_report(missing_slice)

out "Checker expression reports:"
out get(valid_report, 0)
out get(get(valid_report, 1), 0)
out get(out_report, 0)
out get(out_report, 1)
out get(decl_report, 0)
out get(decl_report, 1)
out get(assign_report, 0)
out get(assign_report, 1)
out get(list_report, 0)
out get(list_report, 1)
out get(if_report, 0)
out get(if_report, 1)
out get(self_report, 0)
out get(self_report, 1)
out get(slice_report, 0)
out get(slice_report, 1)
out "Done!"
