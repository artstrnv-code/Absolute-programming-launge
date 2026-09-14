AVStr public_source = "AVInt x = 1 x += 2 out x"
AVStr secret_source = "ASVInt x = 1 x += 2 out x"
AVStr string_source = join(["AVStr text = ", char(34), "a", char(34), " text += ", char(34), "b", char(34), " out text"], "")
AVStr list_source = "List items = [1] items = [2] out items"
VTime public_output = vm.run_source(public_source)
VTime secret_output = vm.run_source(secret_source)
VTime string_output = vm.run_source(string_source)
VTime list_output = vm.run_source(list_source)

out "VM compound guards output:"
out get(public_output, 0)
out len(secret_output)
out len(string_output)
out len(list_output)
out "Done!"
