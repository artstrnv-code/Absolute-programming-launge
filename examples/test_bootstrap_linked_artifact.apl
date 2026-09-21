# Test portable linked APL artifacts produced and loaded by APL code.

VTime quote = char(34)
AVStr source = join([
  "func say() { return echo(",
  quote,
  "a:b,[x]",
  quote,
  ")[::-1] } func echo(x) { return x } out say() AVInt x = input out echo(x) + 1 out 2.5 out true out NONE"
], "")

VTime encoded_report = bootstrap.linked_artifact_report(source)
VTime encoded = get(encoded_report, 1)
VTime loaded_report = bootstrap.load_linked_artifact_report(encoded)
VTime loaded = get(loaded_report, 1)
VTime entry_call = get(get(get(loaded, 1), 0), 1)
VTime say_expr = get(get(get(get(loaded, 2), 2), 0), 0)[1]
VTime say_call = get(say_expr, 1)
VTime run_report = bootstrap.run_linked_artifact_with_input_report(encoded, ["4"])
VTime encoded_again = artifact.encode_loaded(loaded)
VTime bad_header = bootstrap.load_linked_artifact_report("BAD")
VTime trailing = bootstrap.load_linked_artifact_report(join([encoded, "x"], ""))
VTime truncated = bootstrap.load_linked_artifact_report(encoded[:-1])
VTime bad_slot_image = ["APLLOAD2", [["OUT", ["CALL_SLOT", 9, []]]], [[], [], [], true]]
VTime bad_slot_encoded = join([artifact.HEADER, artifact.encode_node(artifact.wrap_loaded(bad_slot_image))], "")
VTime bad_slot_load = bootstrap.load_linked_artifact_report(bad_slot_encoded)
VTime bad_slot_encode = artifact.encode_loaded_report(bad_slot_image)
VTime bad_builtin_image = ["APLLOAD2", [["OUT", ["CALL", "len", []]]], [[], [], [], true]]
VTime bad_builtin_encoded = join([artifact.HEADER, artifact.encode_node(artifact.wrap_loaded(bad_builtin_image))], "")
VTime bad_builtin_load = bootstrap.load_linked_artifact_report(bad_builtin_encoded)
VTime bad_functions_image = ["APLLOAD2", [], [["f"], [[]], [], true]]
VTime bad_functions_encoded = join([artifact.HEADER, artifact.encode_node(artifact.wrap_loaded(bad_functions_image))], "")
VTime bad_functions_load = bootstrap.load_linked_artifact_report(bad_functions_encoded)

out "Bootstrap linked artifact:"
out get(encoded_report, 0)
out encoded[:len(artifact.HEADER)]
out get(loaded_report, 0)
out get(loaded, 0)
out ir.expr_opcode(entry_call)
out get(entry_call, 1)
out ir.expr_opcode(say_call)
out get(say_call, 1)
out encoded_again == encoded
out get(run_report, 0)
out get(get(run_report, 1), 0)
out get(get(run_report, 1), 1)
out get(get(run_report, 1), 2)
out get(get(run_report, 1), 3)
out get(get(run_report, 1), 4)
out get(bad_header, 0)
out get(bad_header, 1)
out get(trailing, 0)
out get(trailing, 1)
out get(truncated, 0)
out get(bad_slot_load, 0)
out get(bad_slot_load, 1)
out get(bad_slot_encode, 0)
out get(bad_slot_encode, 1)
out get(bad_builtin_load, 0)
out get(bad_builtin_load, 1)
out get(bad_functions_load, 0)
out get(bad_functions_load, 1)
out "Done!"
