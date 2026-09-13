AVStr source = "AVFloat x = 1.5 AVFloat y = -2.5 AVFloat z = x + y out z out z < 0.0"
VTime output = vm.run_source(source)

out "VM float output:"

pick(output): line {
  out line
}

out "Done!"
