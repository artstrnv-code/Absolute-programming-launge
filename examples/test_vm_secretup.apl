AVStr source = join(["AVStr token = ", char(34), "root", char(34), " AVStr typ = ", char(34), char(34), " AVStr level = ", char(34), char(34), " secretup(token) typ, level = info(token) out level out token secretup(token) typ, level = info(token) out level out token =self="], "")
VTime output = vm.run_source(source)

out "VM secretup output:"

pick(output): line {
  out line
}

out "Done!"
