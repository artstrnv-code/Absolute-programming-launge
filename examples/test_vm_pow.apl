AVStr source = "AVInt x = pow(2, 3) AVInt nested = 2 ** 3 ** 2 AVInt signed = -2 ** 2 AVFloat y = 2.0 ** 3 AVInt bad = 2 ** -1 ASVInt hidden = 2 out x out nested out signed out y == 8.0 out bad out hidden ** 2"
VTime output = vm.run_source(source)

out "VM pow output:"

pick(output): line {
  out line
}

out "Done!"
