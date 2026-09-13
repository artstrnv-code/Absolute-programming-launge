AVStr source = "AVInt x = pow(2, 3) AVFloat y = pow(2.0, 3) AVInt bad = pow(2, -1) ASVInt hidden = 2 out x out y == 8.0 out bad out pow(hidden, 2)"
VTime output = vm.run_source(source)

out "VM pow output:"

pick(output): line {
  out line
}

out "Done!"
