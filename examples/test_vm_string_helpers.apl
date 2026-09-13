AVStr source = join(["AVStr text = ", char(34), "A P L", char(34), " List parts = split(text, ", char(34), " ", char(34), ") ASVStr hidden = ", char(34), "secret words", char(34), " out len(parts) out join(parts, ", char(34), "-", char(34), ") out contains(text, ", char(34), "P", char(34), ") out ord(", char(34), "A", char(34), ") out char(90) out split(hidden, ", char(34), " ", char(34), ")"], "")
VTime output = vm.run_source(source)

out "VM string helpers output:"

pick(output): line {
  out line
}

out "Done!"
