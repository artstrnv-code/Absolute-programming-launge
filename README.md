# APL

APL, Absoluta Programming Language, is a strict standalone language for native
applications, architecture, and systems programming. Contract-based
communication between isolated foreign-language containers is a first-party
library capability, not the limit of the language itself.

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
- `out` appends non-secret values without adding a newline;
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
- integer `+`, `-`, `*`, `/`, and unary negation are checked operations;
  overflow and integer division by zero produce runtime errors instead of
  panicking or depending on the Rust build profile;
- integer literals cover the full signed `i64` range, including
  `-9223372036854775808`; out-of-range literals are rejected before execution;
- decimal float literals require digits on both sides of `.`, so `1.0` is
  valid and `1.` is rejected;
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
freestanding profile must work without the Rust host so APL can implement
OS-level code directly. Raw memory and hardware access will live in explicit,
capability-controlled system libraries, allowing ordinary builds to exclude
them completely. The same library model is intended to cover APL-owned ISO and
disk-image construction.

Two bootstrap execution paths exist today. The Rust-hosted path serializes a
checked `Program` into binary `.aplc`, then lowers it on load to an in-memory
`CompiledProgram` with linear statement opcodes, jumps, and stack expression
opcodes. `.aplc` is not a stable emitted bytecode format. The APL-written
self-host path serializes list-based IR into `APLMOD3` and `APLLINK3`; its VM is
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

`out` writes exactly the rendered value. Consecutive statements therefore
compose one output stream, and line endings are explicit:

```apl
out "age="
out age
out "\n"
```

String literals support `\n`, `\r`, `\t`, `\0`, `\\`, and `\"`. Unknown or
unfinished escape sequences are syntax errors. A runnable demonstration lives
in `examples/test_output.apl`.

### Library Composition

The first composition layer controls the implicit `base` library:

```apl
include >> base
exclude "len" << base
exclude "pick" << base
out 42
```

Directives are top-level and apply to the complete program. They disappear
before runtime IR is emitted. Calling an excluded builtin or using excluded
`pick` is a compile error. Both the Rust reference checker and APL-written
checker enforce the same contract. Duplicate directives and unknown exports
are rejected.

Named external libraries are resolved to verified `APLMOD3` artifacts. For a
source file `project/program.apl`, `include >> math` loads
`project/libraries/math.aplmod`. Dotted names stay intact, so
`include >> unsafe.memory` resolves `unsafe.memory.aplmod`. The APL-written
bootstrap decoder validates the complete module before its exports are made
visible. Only explicitly included module IR enters the link, excluded exports
are unavailable to user code, and final `APLLINK3` reachability removes unused
APL functions.

The source-relative resolver is used by `check`, `run`, `emit-linked`, linked
package builds, `emit-linked-bootstrap`, and APL Studio when the buffer has a
saved path. A reusable bootstrap compiler accepts the resolved catalog through
its framed input stream and links it over its precompiled base `APLMOD3`, so it
does not recompile the runtime module for each user program. Final `APLLINK3`
artifacts carry a verified `APLMAN1` manifest containing implicit `base`, each
selected named library, and exclusions in source order. Legacy host `.aplc`
output does not carry this catalog. Module content digests and removal of native
builtin/data sections remain future linker work.

A runnable base-composition example is in
`examples/test_composition.apl`.

Build and run the external-library example:

```powershell
cargo run -p apl -- emit-module examples\libraries\math.apl examples\libraries\math.aplmod
cargo run -p apl -- run examples\with_math.apl
```

Containers, Docker execution, and binary `.aplb` bundles come after the language
base is strict and testable.

## Run

### APL Studio for Windows

Build and open the small native Win32 editor:

```powershell
cargo build -p apl_studio
.\target\debug\apl_studio.exe
```

APL Studio is a deliberately small source editor rather than a full IDE. It
opens and saves `.apl` files, runs the current buffer with `F5`, and shows APL
compiler diagnostics, runtime errors, and program output in the lower pane.
Both saved and unsaved buffers use the APL-written parser, checker, linker, and
portable VM path. An untitled buffer can run without being written to disk;
external `include` directives require a saved path so Studio can resolve the
source-relative `libraries` directory. Use `Ctrl+O` and `Ctrl+S` for the
standard file commands. A modified file that already has a path is saved before
it runs. Since Win32 text controls use NUL as a terminator, the output pane
renders an embedded NUL visibly as `\0`; the runtime value itself remains
unchanged. The `Input` pane supplies one value per line to successive `input`
and `secret input` expressions. Empty lines are preserved. Studio input is
visible plain text intended for language testing, not storage or entry of real
secrets.

Check syntax and language rules:

```powershell
cargo run -p apl -- check examples\hello.apl
```

`check` and `run` now always compile an `APLLINK3` artifact through the
APL-written frontend before returning or executing it. Their diagnostics are
therefore the APL frontend's stable text diagnostics. Lexer and parser failures
include one-based `line:column` locations counted in Unicode characters, for
example `parse error at 3:8: expected expression`. The successful path keeps
compact `[kind, value]` tokens; source offsets are collected by a lazy second
lexer pass only when a syntax diagnostic is produced. Semantic checker errors
do not carry source spans yet. The Rust reference parser retains its own source
locations for explicit host tools and differential tests.

Execute through the APL-written compiler and portable VM:

```powershell
cargo run -p apl -- run examples\hello.apl
cargo run -p apl -- run examples\calculator.apl
```

Direct terminal launches do not wait for an EOF marker when no input is piped.
Programs that consume `input` or `secret input` currently receive their input
as redirected lines:

```powershell
@("25", "password") | cargo run -p apl -- run examples\hello.apl
```

This terminal-aware behavior is shared by `run`, `run-ir`, `run-linked`, and
generated bootstrap executables. A future native runtime will provide a truly
interactive input source instead of pre-collecting redirected lines.

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
Rust host bridge and writes the resulting `APLLINK3:` payload. `run-linked`
loads, verifies, and executes that payload through the APL-written artifact
loader and VM. The host passes the input count explicitly, so empty input lines
remain valid values and are not confused with end of input:

```powershell
cargo run -p apl -- emit-linked examples\linked_hello.apl build\linked_hello.apllink
cargo run -p apl -- inspect-linked build\linked_hello.apllink
cargo run -p apl -- run-linked build\linked_hello.apllink
```

`inspect-linked` prints the verified `APLMAN1` library manifest without running
the program. New loaders still accept legacy `APLLINK2` artifacts; those report
that no manifest is available. Re-encoding a loaded v2 image upgrades it to
v3.

The runner returns a versioned `APLRUN1:` response from the APL bridge.
Terminal status and every output fragment are length-framed, so empty strings,
Unicode, colons, newlines, and NUL characters survive without being mistaken
for protocol delimiters. Library callers can use
`run_linked_artifact_report` to receive `LinkedRunStatus::{Ok, Stop, Fail}` and
the original output vector. The compatibility `run_linked_artifact` API still
concatenates successful output into `RunOutput`; `FAIL` becomes
`RuntimeError::Failed` with the actual APL failure reason.

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

The same verified module can be selected by source-level composition when it is
placed in the source file's `libraries` directory. Module export names remain
their declared names, so the example library exposes `math.square` and
`math.answer` after `include >> math`.

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

Build a reusable compiler program, then use that `APLLINK3` program to compile
new source from inside the APL VM:

```powershell
cargo run --release -p apl -- emit-bootstrap-compiler build\bootstrap_compiler.apllink
cargo run --release -p apl -- emit-linked-bootstrap build\bootstrap_compiler.apllink examples\linked_hello.apl build\linked_hello_selfhost.apllink
cargo run --release -p apl -- run-linked build\linked_hello_selfhost.apllink

# The same compiler artifact resolves examples\libraries\math.aplmod.
cargo run --release -p apl -- emit-linked-bootstrap build\bootstrap_compiler.apllink examples\with_math.apl build\with_math_selfhost.apllink
cargo run --release -p apl -- run-linked build\with_math_selfhost.apllink
```

The bootstrap compiler artifact contains the APL-written frontend, checker,
linker, verifier, artifact codec, VM, and an embedded relocatable runtime
module. It accepts source as input and returns either a new `APLLINK3` payload
or a diagnostic through the versioned, length-framed `APLCOMP2` response
protocol. Payload lengths count Unicode characters, so truncation, trailing
data, colons, newlines, and NUL characters are handled unambiguously. The host
still accepts `APLCOMP1` responses from compiler artifacts created by earlier
versions. For named libraries, the host sends an `APLCREQ1:` request followed
by the catalog count, `name`/`APLMOD3` pairs, and source as distinct VM input
values. The compiler validates the modules and includes only libraries selected
by source directives. The compiler artifact can be reused without rebuilding
or re-tokenizing its own compiler sources. An outer Rust bootstrap host still loads and runs the APL VM;
this is compiler-in-VM bootstrap, not native code generation yet. Release mode
is strongly recommended for both compiler construction and nested compilation.

The Rust reference frontend/runtime and the APL-written bootstrap path share a
differential regression corpus. It compares source acceptance, terminal status,
and public output across core syntax, control flow, functions, collections,
builtins, protection metadata, and explicit input streams:

```powershell
cargo test --workspace
cargo test --release -p apl_compiler bootstrap_compiler_artifact_compiles_inside_apl_vm -- --ignored
```

The second command is the expensive compiler-in-VM gate: a reusable compiler
artifact is executed inside the APL VM and must compile and run another APL
program. `run_report` output is always a list of rendered strings, including
the string `"NONE"` for `out NONE`; exhausted input is `NONE` in both runtimes.

`.aplmod` now uses the `APLMOD3:` portable format. It stores unlinked IR plus a
verified symbol table for functions, absolute variables, and `List`
declarations. Function symbols carry arity; absolute symbols carry their full
declared type, and every symbol carries its conservative protection state.
This lets the checker reject secret flow across separately compiled module
boundaries. Decoding rejects malformed structure, trailing data,
inconsistent or duplicate exports, and any pre-existing `CALL_SLOT` opcode.
The user checker imports the symbols, then module IR and user IR are combined
and assigned one final function-slot namespace. The decoder remains compatible
with `APLMOD1:` and `APLMOD2:` artifacts; legacy lists are imported as `SASV`
because those formats did not record their aggregate protection.

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
`=self=`, and common two-character operators. An unterminated quoted string
produces an explicit lexer error token instead of being accepted at EOF.
`lexer.tokenize_with_offsets(source)` additionally returns a parallel list of
Unicode-character offsets for diagnostics; normal compilation uses the compact
`lexer.tokenize(source)` result. A quick lexer smoke-test:

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
`"secret"` remain ordinary strings. Unmatched or unclosed blocks, groups,
calls, lists, indexes, and slices become explicit parser errors. Calls and list
literals require commas between values and reject trailing commas, so malformed
source cannot be lowered as a partial program. Declaration names, function
names and parameters, `pick` items, mutation targets, keywords, delimiters, and
operators must also have their grammar-defined token kinds; quoted strings can
no longer impersonate identifiers or syntax:

```powershell
.\emit_aplc.bat examples\test_parser.apl build\test_parser.aplc
.\run_aplc.bat build\test_parser.aplc
.\compile_apl.bat examples\test_parser.apl build\bat_test_parser
build\bat_test_parser\target\debug\test_parser_compiled.exe
cargo run -p apl -- run examples\test_parser_errors.apl
cargo run -p apl -- run examples\test_parser_token_roles.apl
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
builtin and user-function argument counts must match. Builtin argument types
are conservatively inferred from literals, declarations, slices, conversions,
and builtin results. Statically impossible calls are rejected, while `VTime`
and function results remain runtime-checked. Typed absolute declarations and
assignments require the exact declared value type when the expression type is
known; `VTime`, `NONE`, and input expressions remain dynamically assignable.
There is no implicit `Int`/`Float` widening. Unary and binary operands,
index/slice operands, `pick` values, and `if`/`while` conditions are also
validated when their types are statically known; `VTime` remains deferred to
runtime. Control-flow context is checked before lowering: `return` requires a
function, `break`/`continue` require the current function to be inside a
`while` or `pick`, and loop limits below `-1` are rejected.

Names are checked against the language's ASCII namespace grammar before
lowering or module import. Keywords and builtin names are reserved, function
parameters must be valid and unique, and a `pick` item cannot collide with an
existing visible name. Parameters are local values and may shadow a global
value inside their function; call syntax still resolves the separately declared
function symbol. Reading a bare function name as a value is rejected.

The checker also tracks `AV`/`ASV`/`SASV` through expressions, `VTime`, lists,
tags, mutations, `secretup`, function calls, and block merges. It rejects
protection downgrades, secret values sent to `out`/`stop`/`fail`, public
`=self=` checks on secret values, and invalid input/protection combinations
before IR lowering. Successful checks export protection-aware symbols for
`APLMOD3` modules. `add(list, value)` is a statement-only mutation, while both
`add` and `pop` require a named `List` or `VTime` target. Adding a protected
value to a `VTime` list raises the dynamic list protection, so later public
output or assignment cannot lose the element's protection. Compound assignment
to `VTime` keeps the maximum protection of its previous value and the right-hand
side; arithmetic on a secret temporary therefore cannot launder it back to
`AV`. The runtime independently refuses declaration and assignment downgrades,
so this invariant still holds if static checking is bypassed.

Function calls may also stand alone when their result is intentionally ignored:

```apl
func record(value) {
  return value
}

record(42)
```

Secret `=self=` checks remain rejected because exposing whether a `SASV` value
changed leaks information about that value. Password verification needs a
future explicit `secure_eq(candidate, secret)` security-library primitive that
declassifies exactly its Boolean result and can become constant-time in the
native backend.

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
.\emit_aplc.bat examples\test_checker_builtin_types.apl build\test_checker_builtin_types.aplc
.\run_aplc.bat build\test_checker_builtin_types.aplc
.\compile_apl.bat examples\test_checker_builtin_types.apl build\bat_test_checker_builtin_types
build\bat_test_checker_builtin_types\target\debug\test_checker_builtin_types_compiled.exe
.\emit_aplc.bat examples\test_checker_assign_types.apl build\test_checker_assign_types.aplc
.\run_aplc.bat build\test_checker_assign_types.aplc
.\compile_apl.bat examples\test_checker_assign_types.apl build\bat_test_checker_assign_types
build\bat_test_checker_assign_types\target\debug\test_checker_assign_types_compiled.exe
.\emit_aplc.bat examples\test_checker_operand_types.apl build\test_checker_operand_types.aplc
.\run_aplc.bat build\test_checker_operand_types.aplc
.\compile_apl.bat examples\test_checker_operand_types.apl build\bat_test_checker_operand_types
build\bat_test_checker_operand_types\target\debug\test_checker_operand_types_compiled.exe
.\emit_aplc.bat examples\test_checker_control_flow.apl build\test_checker_control_flow.aplc
.\run_aplc.bat build\test_checker_control_flow.aplc
.\compile_apl.bat examples\test_checker_control_flow.apl build\bat_test_checker_control_flow
build\bat_test_checker_control_flow\target\debug\test_checker_control_flow_compiled.exe
.\emit_aplc.bat examples\test_checker_protection.apl build\test_checker_protection.aplc
.\run_aplc.bat build\test_checker_protection.aplc
.\compile_apl.bat examples\test_checker_protection.apl build\bat_test_checker_protection
build\bat_test_checker_protection\target\debug\test_checker_protection_compiled.exe
.\emit_aplc.bat examples\test_checker_names.apl build\test_checker_names.aplc
.\run_aplc.bat build\test_checker_names.aplc
.\compile_apl.bat examples\test_checker_names.apl build\bat_test_checker_names
build\bat_test_checker_names\target\debug\test_checker_names_compiled.exe
.\emit_aplc.bat examples\test_checker_mutations.apl build\test_checker_mutations.aplc
.\run_aplc.bat build\test_checker_mutations.aplc
.\compile_apl.bat examples\test_checker_mutations.apl build\bat_test_checker_mutations
build\bat_test_checker_mutations\target\debug\test_checker_mutations_compiled.exe
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
it discovers user functions reachable from entry code, follows transitive and
recursive calls, and removes unreachable function names, parameters, and
bodies. It preserves source order among retained functions, then resolves their
names to newly assigned numeric slots and emits `CALL_SLOT` expressions.
Builtin calls remain named. The resulting `APLLOAD2` image therefore avoids
repeated string lookup and does not carry unused APL function code. This is the
reachability foundation for future `include`/`exclude` composition; native
builtin elimination still belongs to the future native linker.

Relocatable `.aplmod` validation uses a separate complete-link pass. It checks
every exported function without deleting exports that are unused until a later
program links the module. Reachability pruning happens only when producing the
final loaded image or `APLLINK3` artifact.

`std/verifier.apl` validates an entire loaded image before the VM accepts it.
It checks instruction and expression opcodes and arities, operators, literal
types, recursive blocks, builtin arities, function-table consistency, unique
function/parameter names, every `CALL_SLOT` bound, and the complete `APLMAN1`
library/exclusion shape.

`std/artifact.apl` is the APL-written portable artifact codec. It converts a
linked image to an `APLLINK3:` wire string and reconstructs it without calling
the Rust parser or IR codec. V3 requires a verified `APLMAN1` composition
manifest; the decoder remains compatible with manifest-less `APLLINK2` and
upgrades it when re-encoding. Scalars are tagged, strings are length-prefixed,
and malformed headers, truncated payloads, unknown tags, trailing data, and
invalid manifests are rejected before execution. Decoded structure also has to
pass the APL verifier.

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
`bootstrap.linked_artifact_manifest_report(encoded)`,
`bootstrap.linked_artifact_manifest(encoded)`,
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

Portable linked artifacts use an `APLLINK3:` header and decode to
`["APLLOAD2", entry, functions, manifest]`. They can be decoded, inspected, and
run by APL code through the `bootstrap.*linked_artifact*` APIs. The loader also
accepts legacy `APLLINK2:` images without a manifest. Unlike the older
diagnostic `APLIR1:` string, both linked formats are unambiguous and
round-trippable.

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
