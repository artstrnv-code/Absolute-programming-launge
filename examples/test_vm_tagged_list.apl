AVStr source = join(["AVStr typ = ", char(34), char(34), " AVStr level = ", char(34), char(34), " List items = [1:SASV, ", char(34), "public", char(34), "] add(items, ", char(34), "hidden", char(34), ":ASV) typ, level = info(items) out typ out level out items out get(items, 0) out get(items, 1) out get(items, 2) pick(items): item { out item }"], "")
VTime output = vm.run_source(source)

out "VM tagged list output:"

pick(output): line {
  out line
}

out "Done!"
