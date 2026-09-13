AVStr source = join(["AVStr raw = ", char(34), "42", char(34), " AVStr bad = ", char(34), "no", char(34), " ASVStr hidden = ", char(34), "7", char(34), " out int(raw) out int(bad) out bool(", char(34), "true", char(34), ") out str(42) out int(hidden)"], "")
VTime output = vm.run_source(source)

out "VM conversions output:"

pick(output): line {
  out line
}

out "Done!"
