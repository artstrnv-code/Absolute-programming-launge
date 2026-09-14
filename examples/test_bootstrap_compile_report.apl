# Test APL bootstrap compile report facade.

AVStr ok_source = "AVInt x = 1 out x"
AVStr bad_source = "AVInt x 1 out x"

VTime ok_report = bootstrap.compile_report(ok_source)
VTime bad_report = bootstrap.compile_report(bad_source)
VTime bad_run = bootstrap.run_report(bad_source)

out "Bootstrap compile reports:"
out get(ok_report, 0)
out ir.opcode(get(get(ok_report, 1), 0))
out get(bad_report, 0)
out get(bad_report, 1)
out get(bad_run, 0)
out get(get(bad_run, 1), 0)
out "Done!"
