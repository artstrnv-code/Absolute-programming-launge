AVStr ok_source = join(["AVStr typ = ", char(34), char(34), " AVStr level = ", char(34), char(34), " SASVStr master = ", char(34), "root", char(34), " typ, level = info(master) out typ out level"], "")
AVStr missing_target_source = join(["out ", char(34), "before", char(34), " SASVStr master = ", char(34), "root", char(34), " typ, level = info(master) out typ"], "")
AVStr secret_target_source = join(["ASVStr typ = ", char(34), char(34), " AVStr level = ", char(34), char(34), " AVStr public = ", char(34), "x", char(34), " typ, level = info(public) out level"], "")
AVStr vtime_source = join(["AVStr typ = ", char(34), char(34), " AVStr level = ", char(34), char(34), " VTime tmp = 1 typ, level = info(tmp) out typ"], "")
VTime ok_output = vm.run_source(ok_source)
VTime missing_target_output = vm.run_source(missing_target_source)
VTime secret_target_output = vm.run_source(secret_target_source)
VTime vtime_output = vm.run_source(vtime_source)

out "VM info guards output:"
out get(ok_output, 0)
out get(ok_output, 1)
out len(missing_target_output)
out get(missing_target_output, 0)
out len(secret_target_output)
out len(vtime_output)
out "Done!"
