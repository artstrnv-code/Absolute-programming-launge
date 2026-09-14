# APL

APL, Absoluta Programming Language, is a small strict base language for
contract-based communication between isolated language containers.

The core idea is simple:

- `AV` variables are normal absolute variables;
- `ASV` variables are secret absolute variables;
- `SASV` variables are runtime-only super-secret values, mainly for keys;
- variable names are globally unique;
- types never change after declaration;
- arithmetic is intentionally small: `+`, `-`, `*`, `/`;
- `Int / Int` stays `Int`;
- strings are not concatenated with `+`;
- `NONE` is an empty value state, not a type change;
- `out` prints non-secret values;
- `input` reads user input and becomes `NONE` when conversion fails;
- `secret input` reads hidden input for `ASV` variables;
- `info(x)` can expose a variable's type and protection level without exposing
  its value;
- `while (condition) (limit)` always has an explicit iteration limit;
- `while` limit `-1` means unlimited, while values below `-1` are rejected;
- `break` and `continue` work inside `while` and `pick`;
- `pick(value): item` iterates over a value and exposes each item as scoped
  `VTime`;
- `VTime` is a local temporary variable that can change its value type;
- `func name(args)` defines functions with arguments as scoped `VTime`;
- function calls use `name(args)`;
- `int(x)`, `float(x)`, `bool(x)`, `str(x)`, `bytes(x)`, and `json(x)` are
  explicit conversions;
- `secretup(x)` raises an absolute variable from `AV` to `ASV`, or from `ASV`
  to `SASV`;
- `List name = [...]` stores flexible values;
- lists can contain other lists, so nested arrays from other languages remain
  representable;
- list elements carry protection labels, not the list itself;
- `value:ASV` and `value:SASV` tag list elements explicitly;
- `add(list, value)`, `get(list, index)`, and `pop(list)` provide minimal stack
  operations;
- `VTime` values can hold local lists, and `add(vtime_list, value)` /
  `pop(vtime_list)` work inside functions and loops;
- `split(value, separator)`, `join(values, separator)`, `len(value)`,
  `contains(value, needle)`, `ord(value)`, `char(value)`, and
  `pow(base, exponent)` provide minimal runtime helpers;
- `value[index]` reads one item and `value[start:end:step]` creates a slice;
- slice `start`, `end`, and `step` are optional, as in `items[:3]`,
  `items[1:]`, and `items[::2]`;
- `stop` exits normally and `fail reason` exits with failure;
- `stop` and `fail` reasons are external channels, so secret reasons are
  denied;
- logical expressions with `and` or `or` must be grouped with parentheses.

This repository starts with the Rust core: parser, contract model, router,
runtime, compiler package generator, and CLI. The standard prelude is composed
from APL files in `std/`, so some runtime behavior is already written in APL
itself.

## Current Syntax

```apl
AVInt x = 4
AVInt y = (x + 2) * 3
AVBool same = x =self=
AVInt age = input
ASVStr token = secret input
SASVStr master_key = "raw"
AVStr type_name = ""
AVStr protection = ""
List public_items = ["A", "P"]
List secret_items = [1:SASV, "abc":ASV]
List matrix = [[1, 2], [3, 4]]

func parse_age(raw) {
  VTime parsed = int(raw)

  if parsed == NONE {
    return NONE
  }

  return parsed
}

age = parse_age(str(age))
type_name, protection = info(master_key)
secretup(type_name)
add(public_items, "L")
add(secret_items, "hidden":ASV)

if (x < y) and (same == true) {
  x += 1
} else if x =self= {
  x = 10
} else {
  x /= 2
}

if age != NONE {
  out age
} else {
  out "bad age"
}

while (x < 20) (100) {
  x += 1

  if x == 12 {
    continue
  }

  if x == 15 {
    break
  }
}

pick("APL"): ch {
  VTime current = ch
  out current
}

VTime first = get(public_items, 0)
VTime last = pop(public_items)
VTime cell = matrix[0][1]
VTime head = public_items[:2]
VTime every_second = public_items[::2]
out first
out last
out cell

stop
```

Containers, Docker execution, and binary `.aplb` bundles come after the language
base is strict and testable.

## Run

Check syntax and language rules:

```powershell
cargo run -p apl -- check examples\hello.apl
```

Execute with the Rust-hosted APL runtime and standard APL prelude:

```powershell
cargo run -p apl -- run examples\hello.apl
cargo run -p apl -- run examples\calculator.apl
```

Generate a standalone Rust package from APL source plus the APL runtime prelude.
The generated executable embeds `.aplc` IR bytes and enters through
`apl_runtime::run_ir_bytes`, which loads a runtime `CompiledProgram`; it does
not parse APL source text at startup. `CompiledProgram` lowers top-level code
and functions into one linear statement opcode code segment with runtime-owned
expression bytecode before execution. Blocks are ranges inside that code
segment, and the runtime executes them with an explicit program counter;
`if/else` and `while` lower to jump opcodes:

```powershell
cargo run -p apl -- build examples\compiled_runtime.apl build\compiled_runtime
cargo build --manifest-path build\compiled_runtime\Cargo.toml
build\compiled_runtime\target\debug\compiled_runtime_compiled.exe
```

Emit and run an `.aplc` compiled artifact directly:

```powershell
cargo run -p apl -- emit examples\compiled_runtime.apl build\compiled_runtime.aplc
cargo run -p apl -- run-ir build\compiled_runtime.aplc
```

Or generate and compile in one command:

```powershell
cargo run -p apl -- compile examples\compiled_runtime.apl build\compiled_runtime
build\compiled_runtime\target\debug\compiled_runtime_compiled.exe
```

Quick Windows batch wrappers:

```powershell
.\emit_aplc.bat examples\compiled_runtime.apl build\compiled_runtime.aplc
.\run_aplc.bat build\compiled_runtime.aplc
.\compile_apl.bat examples\compiled_runtime.apl build\compiled_runtime_bat
```

`examples/bootstrap_runtime.apl` is the current self-host smoke-test. The
standalone executable embeds compiled APL IR, runs the APL-written
lexer/parser/IR/VM from the standard prelude, and that VM executes a nested APL
program with input, secret input, functions, nested lists, and secret-aware
output. User code reaches this through the APL-level `bootstrap.*` facade
instead of calling each internal module directly:

```powershell
.\emit_aplc.bat examples\bootstrap_runtime.apl build\bootstrap_runtime.aplc
.\run_aplc.bat build\bootstrap_runtime.aplc
.\compile_apl.bat examples\bootstrap_runtime.apl build\bat_bootstrap_runtime
build\bat_bootstrap_runtime\target\debug\bootstrap_runtime_compiled.exe
```

The facade can also expose intermediate compiler/runtime layers:

```powershell
.\emit_aplc.bat examples\test_bootstrap.apl build\test_bootstrap.aplc
.\run_aplc.bat build\test_bootstrap.aplc
.\compile_apl.bat examples\test_bootstrap.apl build\bat_test_bootstrap
build\bat_test_bootstrap\target\debug\test_bootstrap_compiled.exe
```

The standard prelude already contains the first APL-written lexer in
`std/lexer.apl`. It scans source text in APL and returns token records as
`[kind, value]` lists. It handles whitespace, line comments, quoted strings,
`=self=`, and common two-character operators. A quick lexer smoke-test:

```powershell
.\emit_aplc.bat examples\test_lexer.apl build\test_lexer.aplc
.\run_aplc.bat build\test_lexer.aplc
.\compile_apl.bat examples\test_lexer.apl build\bat_test_lexer
build\bat_test_lexer\target\debug\test_lexer_compiled.exe
```

The prelude also contains a first parser bootstrap in `std/parser.apl`. It
turns lexer tokens into small AST records for declarations, assignments,
`out`, `stop`, and `fail`:

```powershell
.\emit_aplc.bat examples\test_parser.apl build\test_parser.aplc
.\run_aplc.bat build\test_parser.aplc
.\compile_apl.bat examples\test_parser.apl build\bat_test_parser
build\bat_test_parser\target\debug\test_parser_compiled.exe
```

The next bootstrap compiler layer lives in `std/ir.apl`. It lowers the parser
AST into simple list-based IR instructions:

```powershell
.\emit_aplc.bat examples\test_ir.apl build\test_ir.aplc
.\run_aplc.bat build\test_ir.aplc
.\compile_apl.bat examples\test_ir.apl build\bat_test_ir
build\bat_test_ir\target\debug\test_ir_compiled.exe
```

The first APL-written VM bootstrap lives in `std/vm.apl`. It executes that
list-based IR for declarations, `VTime`, lists, `get`/`len`/`add`/`pop`,
indexing/slicing, `info()` metadata reads, assignments, `if/else if/else` blocks,
bounded `while`, loop flow with `break`/`continue`, `pick`, functions, `return`,
`out`, `stop`, and `fail`. It also denies direct and derived `out` of
`ASV`/`SASV` values for the current bootstrap expression set, and applies the
same rule to `stop`/`fail` reasons. It rejects duplicate runtime declarations
instead of silently shadowing absolute/list names, and rejects assignment or
`secretup` against unknown names. Compound assignment in the VM is limited to
`VTime` or public numeric absolute variables. `add`/`pop` only mutate existing
`List` or `VTime` targets. `info()` requires existing public string targets and
rejects `VTime` sources. Conditions can use comparisons and grouped
logical operators such as `(a == true) and (b != true)`. Expressions support
`=self=` checks against the initial absolute-variable value and explicit
conversion calls such as `int(raw)` and `str(value)`. `secretup(name)` raises
absolute-variable protection inside the VM, and the VM rejects attempts to
write `ASV`/`SASV` values into lower-protection absolute variables. String/list helpers such as
`split`, `join`, `contains`, `ord`, `char`, and `pow` are available in
VM-executed code. Tagged values like `value:ASV` and `value:SASV` are parsed, and VM list
values keep per-element protection labels for `get`/`pop`. `input` and
`secret input` read from an explicit VM input stream when using
`vm.run_source_with_input(source, inputs)`; exhausted input becomes `NONE`.
Absolute declarations and `=` assignments in VM-executed code coerce values to
their declared type, so invalid typed input becomes `NONE`.
The bootstrap parser now handles expression
precedence for arithmetic, comparisons, and `and`/`or`, including `Float`
literals and unary `-`/`not` expressions:

`std/bootstrap.apl` is the public APL-level facade over these modules. It
exposes `bootstrap.tokens(source)`, `bootstrap.ast(source)`,
`bootstrap.ir(source)`, `bootstrap.run(source)`,
`bootstrap.run_with_input(source, inputs)`, `bootstrap.run_report(source)`, and
`bootstrap.run_with_input_report(source, inputs)`. Report calls return
`[status, output]`, where status is `OK`, `STOP`, or `FAIL`.

```powershell
.\emit_aplc.bat examples\test_vm.apl build\test_vm.aplc
.\run_aplc.bat build\test_vm.aplc
.\compile_apl.bat examples\test_vm.apl build\bat_test_vm
build\bat_test_vm\target\debug\test_vm_compiled.exe
.\emit_aplc.bat examples\test_vm_if.apl build\test_vm_if.aplc
.\run_aplc.bat build\test_vm_if.aplc
.\compile_apl.bat examples\test_vm_if.apl build\bat_test_vm_if
build\bat_test_vm_if\target\debug\test_vm_if_compiled.exe
.\emit_aplc.bat examples\test_vm_else_if.apl build\test_vm_else_if.aplc
.\run_aplc.bat build\test_vm_else_if.aplc
.\compile_apl.bat examples\test_vm_else_if.apl build\bat_test_vm_else_if
build\bat_test_vm_else_if\target\debug\test_vm_else_if_compiled.exe
.\emit_aplc.bat examples\test_vm_else.apl build\test_vm_else.aplc
.\run_aplc.bat build\test_vm_else.aplc
.\compile_apl.bat examples\test_vm_else.apl build\bat_test_vm_else
build\bat_test_vm_else\target\debug\test_vm_else_compiled.exe
.\emit_aplc.bat examples\test_vm_while.apl build\test_vm_while.aplc
.\run_aplc.bat build\test_vm_while.aplc
.\compile_apl.bat examples\test_vm_while.apl build\bat_test_vm_while
build\bat_test_vm_while\target\debug\test_vm_while_compiled.exe
.\emit_aplc.bat examples\test_vm_loop_flow.apl build\test_vm_loop_flow.aplc
.\run_aplc.bat build\test_vm_loop_flow.aplc
.\compile_apl.bat examples\test_vm_loop_flow.apl build\bat_test_vm_loop_flow
build\bat_test_vm_loop_flow\target\debug\test_vm_loop_flow_compiled.exe
.\emit_aplc.bat examples\test_vm_func.apl build\test_vm_func.aplc
.\run_aplc.bat build\test_vm_func.aplc
.\compile_apl.bat examples\test_vm_func.apl build\bat_test_vm_func
build\bat_test_vm_func\target\debug\test_vm_func_compiled.exe
.\emit_aplc.bat examples\test_vm_list.apl build\test_vm_list.aplc
.\run_aplc.bat build\test_vm_list.aplc
.\compile_apl.bat examples\test_vm_list.apl build\bat_test_vm_list
build\bat_test_vm_list\target\debug\test_vm_list_compiled.exe
.\emit_aplc.bat examples\test_vm_list_mutation_guards.apl build\test_vm_list_mutation_guards.aplc
.\run_aplc.bat build\test_vm_list_mutation_guards.aplc
.\compile_apl.bat examples\test_vm_list_mutation_guards.apl build\bat_test_vm_list_mutation_guards
build\bat_test_vm_list_mutation_guards\target\debug\test_vm_list_mutation_guards_compiled.exe
.\emit_aplc.bat examples\test_vm_index_slice.apl build\test_vm_index_slice.aplc
.\run_aplc.bat build\test_vm_index_slice.aplc
.\compile_apl.bat examples\test_vm_index_slice.apl build\bat_test_vm_index_slice
build\bat_test_vm_index_slice\target\debug\test_vm_index_slice_compiled.exe
.\emit_aplc.bat examples\test_vm_pick.apl build\test_vm_pick.aplc
.\run_aplc.bat build\test_vm_pick.aplc
.\compile_apl.bat examples\test_vm_pick.apl build\bat_test_vm_pick
build\bat_test_vm_pick\target\debug\test_vm_pick_compiled.exe
.\emit_aplc.bat examples\test_vm_security.apl build\test_vm_security.aplc
.\run_aplc.bat build\test_vm_security.aplc
.\compile_apl.bat examples\test_vm_security.apl build\bat_test_vm_security
build\bat_test_vm_security\target\debug\test_vm_security_compiled.exe
.\emit_aplc.bat examples\test_vm_duplicate_names.apl build\test_vm_duplicate_names.aplc
.\run_aplc.bat build\test_vm_duplicate_names.aplc
.\compile_apl.bat examples\test_vm_duplicate_names.apl build\bat_test_vm_duplicate_names
build\bat_test_vm_duplicate_names\target\debug\test_vm_duplicate_names_compiled.exe
.\emit_aplc.bat examples\test_vm_unknown_targets.apl build\test_vm_unknown_targets.aplc
.\run_aplc.bat build\test_vm_unknown_targets.aplc
.\compile_apl.bat examples\test_vm_unknown_targets.apl build\bat_test_vm_unknown_targets
build\bat_test_vm_unknown_targets\target\debug\test_vm_unknown_targets_compiled.exe
.\emit_aplc.bat examples\test_vm_compound_guards.apl build\test_vm_compound_guards.aplc
.\run_aplc.bat build\test_vm_compound_guards.aplc
.\compile_apl.bat examples\test_vm_compound_guards.apl build\bat_test_vm_compound_guards
build\bat_test_vm_compound_guards\target\debug\test_vm_compound_guards_compiled.exe
.\emit_aplc.bat examples\test_vm_info_guards.apl build\test_vm_info_guards.aplc
.\run_aplc.bat build\test_vm_info_guards.aplc
.\compile_apl.bat examples\test_vm_info_guards.apl build\bat_test_vm_info_guards
build\bat_test_vm_info_guards\target\debug\test_vm_info_guards_compiled.exe
.\emit_aplc.bat examples\test_vm_secret_flow.apl build\test_vm_secret_flow.aplc
.\run_aplc.bat build\test_vm_secret_flow.aplc
.\compile_apl.bat examples\test_vm_secret_flow.apl build\bat_test_vm_secret_flow
build\bat_test_vm_secret_flow\target\debug\test_vm_secret_flow_compiled.exe
.\emit_aplc.bat examples\test_vm_secret_downgrade.apl build\test_vm_secret_downgrade.aplc
.\run_aplc.bat build\test_vm_secret_downgrade.aplc
.\compile_apl.bat examples\test_vm_secret_downgrade.apl build\bat_test_vm_secret_downgrade
build\bat_test_vm_secret_downgrade\target\debug\test_vm_secret_downgrade_compiled.exe
.\emit_aplc.bat examples\test_vm_stop_fail.apl build\test_vm_stop_fail.aplc
.\run_aplc.bat build\test_vm_stop_fail.aplc
.\compile_apl.bat examples\test_vm_stop_fail.apl build\bat_test_vm_stop_fail
build\bat_test_vm_stop_fail\target\debug\test_vm_stop_fail_compiled.exe
.\emit_aplc.bat examples\test_vm_logic.apl build\test_vm_logic.aplc
.\run_aplc.bat build\test_vm_logic.aplc
.\compile_apl.bat examples\test_vm_logic.apl build\bat_test_vm_logic
build\bat_test_vm_logic\target\debug\test_vm_logic_compiled.exe
.\emit_aplc.bat examples\test_vm_precedence.apl build\test_vm_precedence.aplc
.\run_aplc.bat build\test_vm_precedence.aplc
.\compile_apl.bat examples\test_vm_precedence.apl build\bat_test_vm_precedence
build\bat_test_vm_precedence\target\debug\test_vm_precedence_compiled.exe
.\emit_aplc.bat examples\test_vm_float.apl build\test_vm_float.aplc
.\run_aplc.bat build\test_vm_float.aplc
.\compile_apl.bat examples\test_vm_float.apl build\bat_test_vm_float
build\bat_test_vm_float\target\debug\test_vm_float_compiled.exe
.\emit_aplc.bat examples\test_vm_unary.apl build\test_vm_unary.aplc
.\run_aplc.bat build\test_vm_unary.aplc
.\compile_apl.bat examples\test_vm_unary.apl build\bat_test_vm_unary
build\bat_test_vm_unary\target\debug\test_vm_unary_compiled.exe
.\emit_aplc.bat examples\test_vm_pow.apl build\test_vm_pow.aplc
.\run_aplc.bat build\test_vm_pow.aplc
.\compile_apl.bat examples\test_vm_pow.apl build\bat_test_vm_pow
build\bat_test_vm_pow\target\debug\test_vm_pow_compiled.exe
.\emit_aplc.bat examples\test_vm_self.apl build\test_vm_self.aplc
.\run_aplc.bat build\test_vm_self.aplc
.\compile_apl.bat examples\test_vm_self.apl build\bat_test_vm_self
build\bat_test_vm_self\target\debug\test_vm_self_compiled.exe
.\emit_aplc.bat examples\test_vm_conversions.apl build\test_vm_conversions.aplc
.\run_aplc.bat build\test_vm_conversions.aplc
.\compile_apl.bat examples\test_vm_conversions.apl build\bat_test_vm_conversions
build\bat_test_vm_conversions\target\debug\test_vm_conversions_compiled.exe
.\emit_aplc.bat examples\test_vm_secretup.apl build\test_vm_secretup.aplc
.\run_aplc.bat build\test_vm_secretup.aplc
.\compile_apl.bat examples\test_vm_secretup.apl build\bat_test_vm_secretup
build\bat_test_vm_secretup\target\debug\test_vm_secretup_compiled.exe
.\emit_aplc.bat examples\test_vm_string_helpers.apl build\test_vm_string_helpers.aplc
.\run_aplc.bat build\test_vm_string_helpers.aplc
.\compile_apl.bat examples\test_vm_string_helpers.apl build\bat_test_vm_string_helpers
build\bat_test_vm_string_helpers\target\debug\test_vm_string_helpers_compiled.exe
.\emit_aplc.bat examples\test_vm_tagged_list.apl build\test_vm_tagged_list.aplc
.\run_aplc.bat build\test_vm_tagged_list.aplc
.\compile_apl.bat examples\test_vm_tagged_list.apl build\bat_test_vm_tagged_list
build\bat_test_vm_tagged_list\target\debug\test_vm_tagged_list_compiled.exe
.\emit_aplc.bat examples\test_vm_input.apl build\test_vm_input.aplc
.\run_aplc.bat build\test_vm_input.aplc
.\compile_apl.bat examples\test_vm_input.apl build\bat_test_vm_input
build\bat_test_vm_input\target\debug\test_vm_input_compiled.exe
.\emit_aplc.bat examples\test_vm_input_stream.apl build\test_vm_input_stream.aplc
.\run_aplc.bat build\test_vm_input_stream.aplc
.\compile_apl.bat examples\test_vm_input_stream.apl build\bat_test_vm_input_stream
build\bat_test_vm_input_stream\target\debug\test_vm_input_stream_compiled.exe
.\emit_aplc.bat examples\test_vm_typed_input.apl build\test_vm_typed_input.aplc
.\run_aplc.bat build\test_vm_typed_input.aplc
.\compile_apl.bat examples\test_vm_typed_input.apl build\bat_test_vm_typed_input
build\bat_test_vm_typed_input\target\debug\test_vm_typed_input_compiled.exe
.\emit_aplc.bat examples\test_bootstrap_report.apl build\test_bootstrap_report.aplc
.\run_aplc.bat build\test_bootstrap_report.aplc
.\compile_apl.bat examples\test_bootstrap_report.apl build\bat_test_bootstrap_report
build\bat_test_bootstrap_report\target\debug\test_bootstrap_report_compiled.exe
```

Open the minimal GUI runner:

```powershell
powershell -ExecutionPolicy Bypass -File tools\apl_gui.ps1
```
