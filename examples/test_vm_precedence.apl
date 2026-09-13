AVStr source = "AVInt x = 1 + 2 * 3 AVInt y = (1 + 2) * 3 AVBool ok = (x == 7) and (y == 9) or false out x out y out ok"
VTime output = vm.run_source(source)

out "VM precedence output:"

pick(output): line {
  out line
}

out "Done!"
