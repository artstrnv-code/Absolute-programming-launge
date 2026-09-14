# Test APL checker target validation.

AVStr missing_assign = "missing = 1 out missing"
AVStr missing_secretup = "secretup(missing)"
AVStr list_secretup = "List items = [] secretup(items)"
AVStr missing_info_target = join(["AVStr level = ", char(34), char(34), " typ, level = info(level)"], "")
AVStr bad_info_target = join(["AVInt typ = 1 AVStr level = ", char(34), char(34), " typ, level = info(typ)"], "")
AVStr missing_info_source = join(["AVStr typ = ", char(34), char(34), " AVStr level = ", char(34), char(34), " typ, level = info(missing)"], "")

VTime assign_report = bootstrap.compile_report(missing_assign)
VTime secretup_report = bootstrap.compile_report(missing_secretup)
VTime list_report = bootstrap.compile_report(list_secretup)
VTime info_target_report = bootstrap.compile_report(missing_info_target)
VTime bad_info_report = bootstrap.compile_report(bad_info_target)
VTime info_source_report = bootstrap.compile_report(missing_info_source)

out "Checker target reports:"
out get(assign_report, 0)
out get(assign_report, 1)
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
