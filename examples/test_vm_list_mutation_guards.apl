AVStr source = "List items = [1] add(items, 2) AVInt x = 7 VTime bad_add = add(x, 9) VTime missing_add = add(missing, 1) VTime missing_pop = pop(missing) out len(items) out pop(items) out len(items) out bad_add out missing_add out missing_pop"
VTime output = vm.run_source(source)

out "VM list mutation guards output:"

pick(output): line {
  out line
}

out "Done!"
