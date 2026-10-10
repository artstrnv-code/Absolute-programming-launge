AVStr source = join(["AVBool a = true AVBool b = false ASVBool hidden = true if (a == true) and (b != true) { out ", char(34), "ok", char(34), " } if (hidden == true) or (b == true) { out hidden == true }"], "")
VTime output = vm.run_source(source)

out "VM logic output:"

pick(output): line {
  out line
}

out "Done!"
