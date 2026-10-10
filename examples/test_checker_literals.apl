# Test numeric literal boundaries in the APL-written frontend.

AVStr min_source = "AVInt value = -9223372036854775808 out value"
AVStr positive_overflow = "AVInt value = 9223372036854775808"
AVStr negative_overflow = "AVInt value = -9223372036854775809"
AVStr trailing_dot = "AVFloat value = 1."

VTime min_report = bootstrap.run_report(min_source)
VTime positive_report = bootstrap.compile_report(positive_overflow)
VTime negative_report = bootstrap.compile_report(negative_overflow)
VTime float_report = bootstrap.compile_report(trailing_dot)

out "Checker literal reports:"
out get(min_report, 0)
out get(get(min_report, 1), 0)
out get(positive_report, 0)
out get(positive_report, 1)
out get(negative_report, 0)
out get(negative_report, 1)
out get(float_report, 0)
out get(float_report, 1)
out "Done!"
