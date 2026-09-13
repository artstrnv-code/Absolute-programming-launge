AVStr source = "AVInt x = 4 AVInt y = -x AVFloat z = -1.5 AVBool flag = false out y out z out not flag out -(x + 1)"
VTime output = vm.run_source(source)

out "VM unary output:"

pick(output): line {
  out line
}

out "Done!"
