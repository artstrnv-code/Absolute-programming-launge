# Test list-mutation contracts in the APL checker.

List mutation_sources = [
  "VTime items = [1] add(items, 2) VTime item = pop(items) out item",
  "List items = [1] VTime result = add(items, 2)",
  "add([1], 2)",
  "VTime item = pop([1])",
  "VTime items = [1] ASVInt hidden = 2 add(items, hidden) out items"
]

pick(mutation_sources): mutation_source {
  VTime mutation_report = bootstrap.compile_report(mutation_source)
  out get(mutation_report, 0)

  if get(mutation_report, 0) == checker.STATUS_FAIL {
    out get(mutation_report, 1)
  }
}

out "Done!"
