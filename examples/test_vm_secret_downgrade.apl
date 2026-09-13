AVStr source = join(["out ", char(34), "before", char(34), " ASVStr token = ", char(34), "root", char(34), " VTime tmp = token AVStr public = tmp out public out ", char(34), "after", char(34)], "")
VTime output = vm.run_source(source)

out "VM secret downgrade output:"

pick(output): line {
  out line
}

out "Done!"
