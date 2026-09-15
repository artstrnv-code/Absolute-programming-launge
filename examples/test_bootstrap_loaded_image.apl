# Test APL bootstrap loaded image facade.

AVStr source = "func inc(x) { return x + 1 } AVInt x = input out inc(x)"
VTime loaded_report = bootstrap.loaded_image_report(source)
VTime loaded = get(loaded_report, 1)
VTime first_run = bootstrap.run_loaded_image_with_input_report(loaded, ["4"])
VTime second_run = bootstrap.run_loaded_image_with_input_report(loaded, ["9"])
VTime invalid_load = bootstrap.load_artifact_image_report(["BAD"])
VTime invalid_run = bootstrap.run_loaded_image_report(["BAD"])

out "Bootstrap loaded image:"
out get(loaded_report, 0)
out get(loaded, 0)
out ir.opcode(get(get(loaded, 1), 0))
out get(first_run, 0)
out get(get(first_run, 1), 0)
out get(second_run, 0)
out get(get(second_run, 1), 0)
out get(invalid_load, 0)
out get(get(invalid_load, 1), 0)
out get(invalid_run, 0)
out get(get(invalid_run, 1), 0)
out "Done!"
