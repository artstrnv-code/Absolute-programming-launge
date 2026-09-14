# Test APL bootstrap report facade.

AVStr ok_source = "AVInt x = input x += 1 out x"
AVStr stop_source = join(["out ", char(34), "before", char(34), " stop ", char(34), "done", char(34), " out ", char(34), "after", char(34)], "")
AVStr fail_source = join(["out ", char(34), "before", char(34), " fail ", char(34), "bad", char(34), " out ", char(34), "after", char(34)], "")

VTime ok_report = bootstrap.run_with_input_report(ok_source, ["4"])
VTime stop_report = bootstrap.run_report(stop_source)
VTime fail_report = bootstrap.run_report(fail_source)

out "Bootstrap reports:"
out get(ok_report, 0)
out get(get(ok_report, 1), 0)
out get(stop_report, 0)
out get(get(stop_report, 1), 0)
out get(get(stop_report, 1), 1)
out get(fail_report, 0)
out get(get(fail_report, 1), 0)
out get(get(fail_report, 1), 1)
out "Done!"
