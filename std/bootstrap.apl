# APL bootstrap facade.
# This module exposes the APL-owned source -> tokens -> AST -> IR -> VM pipeline
# as one stable namespace for compiled APL programs.

func bootstrap.tokens(source) {
  return lexer.tokenize(source)
}

func bootstrap.ast(source) {
  return parser.parse_source(source)
}

func bootstrap.ir(source) {
  return ir.compile_ast(bootstrap.ast(source))
}

func bootstrap.run(source) {
  return vm.run_ir(bootstrap.ir(source))
}

func bootstrap.run_report(source) {
  return vm.run_ir_report(bootstrap.ir(source))
}

func bootstrap.run_with_input(source, inputs) {
  return vm.run_ir_with_input(bootstrap.ir(source), inputs)
}

func bootstrap.run_with_input_report(source, inputs) {
  return vm.run_ir_with_input_report(bootstrap.ir(source), inputs)
}
