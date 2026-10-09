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
- `pick` preserves each list element's own protection label instead of applying
  the aggregate list label to every item;
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

## Compilation Targets

APL's production target is AOT native compilation. The intended backend lowers
APL into a low-level native IR, can emit readable assembly or platform object
files, and uses the platform linker to produce PE `.exe`, ELF, and Mach-O
executables. AppImage packaging sits on top of the Linux ELF result. A future
freestanding profile must work without the Rust host so APL can participate in
OS-level development while unsafe memory and hardware operations remain behind
explicit C/C++/Rust/ASM component contracts.

Two bootstrap execution paths exist today. The Rust-hosted path serializes a
checked `Program` into binary `.aplc`, then lowers it on load to an in-memory
`CompiledProgram` with linear statement opcodes, jumps, and stack expression
opcodes. `.aplc` is not a stable emitted bytecode format. The APL-written
self-host path serializes list-based IR into `APLMOD2` and `APLLINK2`; its VM is
the portable semantic reference and bootstrap vehicle, not the final production
backend. A bytecode target may remain useful for portability and debugging, but
native object and executable output is the primary goal.

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

Generate a Rust-hosted package whose program payload is compiled by the
APL-written pipeline into `program.apllink`. The generated executable loads,
verifies, and executes that portable artifact:

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

Emit and run an APL-owned portable linked artifact. `emit-linked` enters the
APL-written lexer/parser/checker/IR/linker/artifact pipeline through a narrow
Rust host bridge and writes the resulting `APLLINK2:` payload. `run-linked`
loads, verifies, and executes that payload through the APL-written artifact
loader and VM. The host passes the input count explicitly, so empty input lines
remain valid values and are not confused with end of input:

```powershell
cargo run -p apl -- emit-linked examples\linked_hello.apl build\linked_hello.apllink
cargo run -p apl -- run-linked build\linked_hello.apllink
```

The linked compiler composes `std/runtime.apl` with the user source before the
APL checker and linker run. Runtime helpers such as `apl.pow_int` therefore
become ordinary functions in the artifact's single verified function table;
compiler implementation modules such as the lexer, parser, and VM are not
copied into the user artifact:

```powershell
cargo run -p apl -- emit-linked examples\compiled_runtime.apl build\compiled_runtime.apllink
cargo run -p apl -- run-linked build\compiled_runtime.apllink
```

The `.apllink` format is the current portable bootstrap format. It is distinct
from host-side binary `.aplc` IR and is not yet native machine code.

Precompile a reusable APL function module, then link new user source against it
without tokenizing or parsing the module source again:

```powershell
cargo run -p apl -- emit-module std\runtime.apl build\runtime.aplmod
cargo run -p apl -- emit-linked-module build\runtime.aplmod examples\compiled_runtime.apl build\compiled_runtime.apllink
cargo run -p apl -- run-linked build\compiled_runtime.apllink
```

Modules can be extended incrementally. The new source is checked against the
verified exports already stored in the module, then its unlinked IR and exports
are appended without assigning final function slots:

```powershell
cargo run -p apl -- emit-module std\runtime.apl build\runtime.aplmod
cargo run -p apl -- extend-module build\runtime.aplmod std\lexer.apl build\runtime_lexer.aplmod
```

Build the complete reusable compiler/runtime prelude as one module, then link
and run the self-host smoke-test through that precompiled module:

```powershell
cargo run -p apl -- emit-standard-module build\standard.aplmod
cargo run -p apl -- emit-linked-module build\standard.aplmod examples\bootstrap_runtime.apl build\bootstrap_runtime.apllink
cargo run --release -p apl -- run-linked build\bootstrap_runtime.apllink
```

`emit-standard-module` compiles `runtime`, `lexer`, `parser`, `checker`, `ir`,
`linker`, `verifier`, `vm`, `artifact`, and `bootstrap` sequentially inside one
APL invocation. The combined IR and symbol table stay in memory and are encoded
once. The current VM-in-VM smoke-test is computationally expensive, so release
mode is recommended for that final command.

`.aplmod` now uses the `APLMOD2:` portable format. It stores unlinked IR plus a
verified symbol table for functions, absolute variables, and `List`
declarations. Function symbols carry arity; absolute symbols carry their full
declared type. Decoding rejects malformed structure, trailing data,
inconsistent or duplicate exports, and any pre-existing `CALL_SLOT` opcode.
The user checker imports the symbols, then module IR and user IR are combined
and assigned one final function-slot namespace. The decoder remains compatible
with function-only `APLMOD1:` artifacts.

The explicit `build-linked` and `compile-linked` names are aliases for the
default `build` and `compile` commands:

```powershell
cargo run -p apl -- build-linked examples\compiled_runtime.apl build\compiled_runtime_linked
cargo build --manifest-path build\compiled_runtime_linked\Cargo.toml
build\compiled_runtime_linked\target\debug\compiled_runtime_compiled.exe

cargo run -p apl -- compile-linked examples\compiled_runtime.apl build\compiled_runtime_linked
build\compiled_runtime_linked\target\debug\compiled_runtime_compiled.exe
```

The generated launcher embeds `program.apllink` and enters through
`apl_compiler::run_linked_artifact`, which invokes the APL-written loader,
verifier, and VM. The launcher is still a Rust bootstrap host; this is not yet
native APL output.

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
.\compile_linked_apl.bat examples\compiled_runtime.apl build\compiled_runtime_linked_bat
```

`compile_linked_apl.bat` follows the default portable-artifact path.
`compile_apl.bat` intentionally invokes legacy `compile-host`; it remains for
self-host/compiler tests that need the complete compiler prelude inside the
host `.aplc` image.

`examples/bootstrap_runtime.apl` is the current self-host smoke-test. It can be
linked against `standard.aplmod`, after which the APL-written VM executes the
APL-written lexer/parser/checker/IR/linker/VM and runs a nested APL program with
input, secret input, functions, nested lists, and secret-aware output. User code
reaches this through the APL-level `bootstrap.*` facade instead of calling each
internal module directly. The older host `.aplc` route remains available for
focused bootstrap diagnostics:

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
`out`, `stop`, and `fail`. Syntax delimiters are matched by token kind as well
as value, so string literals such as `"("`, `"["`, `"]"`, `"input"`, and
`"secret"` remain ordinary strings:

```powershell
.\emit_aplc.bat examples\test_parser.apl build\test_parser.aplc
.\run_aplc.bat build\test_parser.aplc
.\compile_apl.bat examples\test_parser.apl build\bat_test_parser
build\bat_test_parser\target\debug\test_parser_compiled.exe
```

`std/checker.apl` is the first APL-written semantic checker layer. It currently
validates duplicate declaration/function names, unknown assignment targets,
invalid `secretup` targets, and invalid `info()` targets/sources before IR
lowering. Assignments are limited to absolute variables or `VTime`, and
compound assignments are limited to `VTime` or public numeric absolute
variables. Expression validation rejects unknown variable reads across
declarations, assignments, output, conditions, function-call arguments, list
literals, indexing/slicing, tags, and `=self=` targets. It also validates
function-call targets: VM builtins are allowed, user functions may be called
before their top-level declaration, non-functions cannot be called, and
user-function argument counts must match:

```powershell
.\emit_aplc.bat examples\test_checker.apl build\test_checker.aplc
.\run_aplc.bat build\test_checker.aplc
.\compile_apl.bat examples\test_checker.apl build\bat_test_checker
build\bat_test_checker\target\debug\test_checker_compiled.exe
.\emit_aplc.bat examples\test_checker_targets.apl build\test_checker_targets.aplc
.\run_aplc.bat build\test_checker_targets.aplc
.\compile_apl.bat examples\test_checker_targets.apl build\bat_test_checker_targets
build\bat_test_checker_targets\target\debug\test_checker_targets_compiled.exe
.\emit_aplc.bat examples\test_checker_exprs.apl build\test_checker_exprs.aplc
.\run_aplc.bat build\test_checker_exprs.aplc
.\compile_apl.bat examples\test_checker_exprs.apl build\bat_test_checker_exprs
build\bat_test_checker_exprs\target\debug\test_checker_exprs_compiled.exe
.\emit_aplc.bat examples\test_checker_calls.apl build\test_checker_calls.aplc
.\run_aplc.bat build\test_checker_calls.aplc
.\compile_apl.bat examples\test_checker_calls.apl build\bat_test_checker_calls
build\bat_test_checker_calls\target\debug\test_checker_calls_compiled.exe
```

The next bootstrap compiler layer lives in `std/ir.apl`. It lowers the parser
AST into simple list-based IR instructions:

```powershell
.\emit_aplc.bat examples\test_ir.apl build\test_ir.aplc
.\run_aplc.bat build\test_ir.aplc
.\compile_apl.bat examples\test_ir.apl build\bat_test_ir
build\bat_test_ir\target\debug\test_ir_compiled.exe
```

`std/linker.apl` is the APL-written link stage. When an IR program is loaded,
it recursively walks entry code and function bodies, resolves user-function
names to stable numeric slots, and emits `CALL_SLOT` expressions. Builtin calls
remain named. The resulting `APLLOAD2` image therefore avoids repeated string
lookup for user functions during execution.

`std/verifier.apl` validates an entire loaded image before the VM accepts it.
It checks instruction and expression opcodes and arities, operators, literal
types, recursive blocks, builtin arities, function-table consistency, unique
function/parameter names, and every `CALL_SLOT` bound.

`std/artifact.apl` is the APL-written portable artifact codec. It converts a
linked image to an `APLLINK2:` wire string and reconstructs it without calling
the Rust parser or IR codec. Scalars are tagged, strings are length-prefixed,
and malformed headers, truncated payloads, unknown tags, and trailing data are
rejected before execution. Decoded structure also has to pass the APL verifier.

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
The VM environment stores compact binding records with an explicit scope depth.
Function calls and `if`/`while`/`pick` bodies remove their local `VTime`
bindings on exit while preserving updates to outer and absolute bindings and
the input cursor. This permits the same local name in independent functions and
blocks without weakening global uniqueness for absolute variables and lists.
The bootstrap parser now handles expression
precedence for arithmetic, comparisons, and `and`/`or`, including `Float`
literals and unary `-`/`not` expressions:

`std/bootstrap.apl` is the public APL-level facade over these modules. It
exposes `bootstrap.tokens(source)`, `bootstrap.ast(source)`,
`bootstrap.ir(source)`, `bootstrap.run(source)`,
`bootstrap.compile_report(source)`, `bootstrap.run_with_input(source, inputs)`,
`bootstrap.artifact_report(source)`, `bootstrap.artifact_image_report(source)`,
`bootstrap.run_artifact_image(image)`,
`bootstrap.run_artifact_image_with_input(image, inputs)`,
`bootstrap.loaded_image_report(source)`,
`bootstrap.linked_artifact_report(source)`,
`bootstrap.load_linked_artifact_report(encoded)`,
`bootstrap.run_linked_artifact(encoded)`,
`bootstrap.run_linked_artifact_with_input(encoded, inputs)`,
`bootstrap.run_loaded_image(loaded)`,
`bootstrap.run_loaded_image_with_input(loaded, inputs)`,
`bootstrap.run_report(source)`, and
`bootstrap.run_with_input_report(source, inputs)`. Report calls return
`[status, output]`, where status is `OK`, `STOP`, or `FAIL`. Compile reports
return `[OK, program]` or `[FAIL, message]`, and `run_report` stops before VM
execution when source cannot be lowered to IR. Artifact reports currently emit a
portable text IR image with an `APLIR1:` header; binary `.aplc` and native
output remain host-side/future stages. Artifact image reports return structured
`["APLIR1", program]` images that can be run repeatedly without re-tokenizing,
re-parsing, or re-checking the original source text. Loaded image reports
return `["APLLOAD2", entry, functions]` images with the VM function table
already collected, user calls linked to numeric slots, and top-level `FUNC`
declarations removed from executable entry code. Repeated runs skip source
compilation, function-table collection, user-function name lookup, and function
declaration no-ops. The source-level `bootstrap.run*` facade compiles to a
linked loaded image before execution.

Portable linked artifacts use an `APLLINK2:` header and can be decoded and run
by APL code through the `bootstrap.*linked_artifact*` APIs. Unlike the older
diagnostic `APLIR1:` string, this format is unambiguous and round-trippable.

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
.\emit_aplc.bat examples\test_bootstrap_compile_report.apl build\test_bootstrap_compile_report.aplc
.\run_aplc.bat build\test_bootstrap_compile_report.aplc
.\compile_apl.bat examples\test_bootstrap_compile_report.apl build\bat_test_bootstrap_compile_report
build\bat_test_bootstrap_compile_report\target\debug\test_bootstrap_compile_report_compiled.exe
.\emit_aplc.bat examples\test_bootstrap_artifact.apl build\test_bootstrap_artifact.aplc
.\run_aplc.bat build\test_bootstrap_artifact.aplc
.\compile_apl.bat examples\test_bootstrap_artifact.apl build\bat_test_bootstrap_artifact
build\bat_test_bootstrap_artifact\target\debug\test_bootstrap_artifact_compiled.exe
.\emit_aplc.bat examples\test_bootstrap_artifact_image.apl build\test_bootstrap_artifact_image.aplc
.\run_aplc.bat build\test_bootstrap_artifact_image.aplc
.\compile_apl.bat examples\test_bootstrap_artifact_image.apl build\bat_test_bootstrap_artifact_image
build\bat_test_bootstrap_artifact_image\target\debug\test_bootstrap_artifact_image_compiled.exe
.\emit_aplc.bat examples\test_bootstrap_loaded_image.apl build\test_bootstrap_loaded_image.aplc
.\run_aplc.bat build\test_bootstrap_loaded_image.aplc
.\compile_apl.bat examples\test_bootstrap_loaded_image.apl build\bat_test_bootstrap_loaded_image
build\bat_test_bootstrap_loaded_image\target\debug\test_bootstrap_loaded_image_compiled.exe
```

Open the minimal GUI runner:

```powershell
powershell -ExecutionPolicy Bypass -File tools\apl_gui.ps1
```
