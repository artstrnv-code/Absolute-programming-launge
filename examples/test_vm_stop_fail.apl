AVStr public_stop = join(["stop ", char(34), "done", char(34), " out ", char(34), "after", char(34)], "")
AVStr secret_stop = join(["ASVStr token = ", char(34), "root", char(34), " stop token out ", char(34), "after", char(34)], "")
AVStr public_fail = join(["fail ", char(34), "bad", char(34), " out ", char(34), "after", char(34)], "")

VTime stop_output = vm.run_source(public_stop)
VTime secret_output = vm.run_source(secret_stop)
VTime fail_output = vm.run_source(public_fail)

out "VM stop/fail output:"

pick(stop_output): line {
  out line
}

pick(secret_output): line {
  out line
}

pick(fail_output): line {
  out line
}

out "Done!"
