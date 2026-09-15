# Test APL checker target validation.

AVStr missing_assign = "missing = 1 out missing"
AVStr list_assign = "List items = [] items = 1"
AVStr func_assign = "func go() { return 1 } go = 2"
AVStr secret_compound = "ASVInt hidden = 1 hidden += 1"
AVStr string_compound = join(["AVStr text = ", char(34), char(34), " text += 1"], "")
AVStr vtime_compound = "VTime temp = 1 temp += 1 out temp"
AVStr missing_secretup = "secretup(missing)"
AVStr list_secretup = "List items = [] secretup(items)"
AVStr missing_info_target = join(["AVStr level = ", char(34), char(34), " typ, level = info(level)"], "")
AVStr bad_info_target = join(["AVInt typ = 1 AVStr level = ", char(34), char(34), " typ, level = info(typ)"], "")
AVStr missing_info_source = join(["AVStr typ = ", char(34), char(34), " AVStr level = ", char(34), char(34), " typ, level = info(missing)"], "")

VTime assign_report = bootstrap.compile_report(missing_assign)
VTime list_assign_report = bootstrap.compile_report(list_assign)
VTime func_assign_report = bootstrap.compile_report(func_assign)
VTime secret_compound_report = bootstrap.compile_report(secret_compound)
VTime string_compound_report = bootstrap.compile_report(string_compound)
VTime vtime_compound_report = bootstrap.run_report(vtime_compound)
VTime secretup_report = bootstrap.compile_report(missing_secretup)
VTime list_report = bootstrap.compile_report(list_secretup)
VTime info_target_report = bootstrap.compile_report(missing_info_target)
VTime bad_info_report = bootstrap.compile_report(bad_info_target)
VTime info_source_report = bootstrap.compile_report(missing_info_source)

out "Checker target reports:"
out get(assign_report, 0)
out get(assign_report, 1)
out get(list_assign_report, 0)
out get(list_assign_report, 1)
out get(func_assign_report, 0)
out get(func_assign_report, 1)
out get(secret_compound_report, 0)
out get(secret_compound_report, 1)
out get(string_compound_report, 0)
out get(string_compound_report, 1)
out get(vtime_compound_report, 0)
out get(get(vtime_compound_report, 1), 0)
out get(secretup_report, 0)
out get(secretup_report, 1)
out get(list_report, 0)
out get(list_report, 1)
out get(info_target_report, 0)
out get(info_target_report, 1)
out get(bad_info_report, 0)
out get(bad_info_report, 1)
out get(info_source_report, 0)
out get(info_source_report, 1)
out "Done!"
