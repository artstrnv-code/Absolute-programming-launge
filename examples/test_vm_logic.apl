AVStr source = join(["AVBool a = true AVBool b = false ASVBool secret = true if (a == true) and (b != true) { out ", char(34), "ok", char(34), " } if (secret == true) or (b == true) { out secret == true }"], "")
VTime output = vm.run_source(source)

out "VM logic output:"

pick(output): line {
  out line
}

out "Done!"
