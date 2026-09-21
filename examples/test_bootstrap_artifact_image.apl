# Test APL bootstrap artifact image facade.

AVStr source = "func inc(x) { return x + 1 } AVInt x = input out inc(x)"
VTime image_report = bootstrap.artifact_image_report(source)
VTime image = get(image_report, 1)
VTime first_run = bootstrap.run_artifact_image_with_input_report(image, ["4"])
VTime second_run = bootstrap.run_artifact_image_with_input_report(image, ["9"])
VTime invalid_run = bootstrap.run_artifact_image_report(["BAD"])

out "Bootstrap artifact image:"
out get(image_report, 0)
out get(image, 0)
out ir.opcode(get(get(image, 1), 0))
out get(first_run, 0)
out get(get(first_run, 1), 0)
out get(second_run, 0)
out get(get(second_run, 1), 0)
out get(invalid_run, 0)
out get(get(invalid_run, 1), 0)
out "Done!"
