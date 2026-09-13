AVStr source = "AVInt x = 0 while ( x < 10 ) ( 20 ) { x += 1 if x == 2 { continue } if x == 5 { break } out x } out x"
VTime vm_output = vm.run_source(source)

out "VM loop flow output:"
pick(vm_output): line {
  out line
}
out "Done!"
