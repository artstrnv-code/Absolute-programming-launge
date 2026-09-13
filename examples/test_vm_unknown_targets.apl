AVStr assign_source = join(["out ", char(34), "before", char(34), " missing = 4 out ", char(34), "after", char(34)], "")
AVStr secretup_source = join(["out ", char(34), "start", char(34), " secretup(missing) out ", char(34), "after", char(34)], "")
VTime assign_output = vm.run_source(assign_source)
VTime secretup_output = vm.run_source(secretup_source)

out "VM unknown targets output:"
out len(assign_output)
out get(assign_output, 0)
out get(assign_output, 1)
out len(secretup_output)
out get(secretup_output, 0)
out get(secretup_output, 1)
out "Done!"
