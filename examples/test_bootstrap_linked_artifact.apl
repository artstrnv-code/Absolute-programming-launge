# Test portable linked APL artifacts produced and loaded by APL code.

VTime quote = char(34)
AVStr source = join([
  "func say() { return echo(",
  quote,
  "a:b,[x]",
  quote,
  ") } func echo(x) { return x } out say() AVInt x = input out echo(x) + 1 out 2.5 out true out NONE"
], "")

VTime encoded_report = bootstrap.linked_artifact_report(source)
VTime encoded = get(encoded_report, 1)
VTime loaded_report = bootstrap.load_linked_artifact_report(encoded)
VTime loaded = get(loaded_report, 1)
VTime entry_call = get(get(get(loaded, 1), 0), 1)
VTime say_call = get(get(get(get(loaded, 2), 2), 0), 0)[1]
VTime run_report = bootstrap.run_linked_artifact_with_input_report(encoded, ["4"])
VTime encoded_again = artifact.encode_loaded(loaded)
VTime bad_header = bootstrap.load_linked_artifact_report("BAD")
VTime trailing = bootstrap.load_linked_artifact_report(join([encoded, "x"], ""))
VTime truncated = bootstrap.load_linked_artifact_report(encoded[:-1])

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
out "Done!"
