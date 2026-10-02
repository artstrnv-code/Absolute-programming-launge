# APL Context

This file records the language decisions we made during early design. Keep it
updated when the language changes.

## Identity

APL means Absoluta Programming Language.

APL is a middle-level architecture language: it is meant to control everything,
not implement everything directly. High-level work should be delegated to
languages like Python/JS. Low-level work should be delegated to C/Rust/ASM.
APL owns orchestration, contracts, routing, security boundaries, typed shared
state, and future container/component composition.

Long-term target: APL must become a native compiled language, not permanently a
language hosted by a Rust runtime. The current Rust implementation is a
bootstrap host. Later compiler stages should move toward native code or a
native kernel-suitable runtime path, because APL is intended to be capable of
OS-level development in the future.

Native APL does not imply exposing raw addresses, pointers, or manual allocation
as ordinary language features. Those mechanisms belong in low-level C/C++/Rust
or ASM components. APL should be able to produce and run native orchestration
code without a foreign managed runtime while keeping unsafe hardware and memory
operations behind explicit component contracts.

Core phrase:

```text
APL controls.
Components compute.
Contracts protect.
```

## Roadmap

Current high-level plan:

1. Build the base language.
2. Build the interpreted runtime.
3. Build the compiled runtime path and start moving standard behavior into APL.
4. Add containers/components.
5. Polish tooling, diagnostics, and packaging.

Current implementation has parser, checker, CLI, GUI shell, runtime v0.1
interpreter coverage for the current base language, and a first compiler
package generator. It is not yet a native APL bytecode or container bundle
system.

## Variable Families

Absolute variables are global architecture-level variables. Their names are
globally unique: two variables cannot share a name even if their types or
protection levels differ.

```apl
AVInt x = 4
ASVStr token = secret input
SASVStr master_key = "raw"
```

Families:

- `AV` is a normal absolute variable.
- `ASV` is an absolute secret variable.
- `SASV` is a super absolute secret variable.

Types:

- `Int`
- `Float`
- `Bool`
- `Str`
- `Bytes`
- `Json`

The declared type never changes. `AVStr` cannot become a number. `NONE` is an
empty value state and does not change the declared type.

## SASV

`SASV` is mainly for keys, root secrets, integrity checks, and security gates.

Important rule: `SASV` is not just "more secret ASV". Its value is for APL
runtime use only. Containers must never receive `SASV` values. The value must
not be copied into ordinary variables or output channels.

`info(SASV)` is allowed because it exposes metadata, not the value.

## VTime

`VTime` is not an absolute variable. It is a local temporary variable.

```apl
VTime tmp = input
```

Rules:

- local to the current function/block/loop scope;
- destroyed after leaving that scope;
- can hold any value type;
- can change its held value type;
- does not have its own protection level;
- can hold a value that carries a protection label;
- cannot be used with `info()`;
- cannot be passed to `secretup()`.

Protection is checked when writing from `VTime` into another variable.

Example:

```apl
ASVStr token = secret input
VTime item = token

AVStr public = item   # invalid: ASV -> AV downgrade
SASVStr sealed = item # valid: ASV -> SASV upgrade
```

## Names

Variable names are strict for future compatibility with other languages.

Allowed:

```text
[A-Za-z_][A-Za-z0-9_]*(\.[A-Za-z_][A-Za-z0-9_]*)*
```

Names may use ASCII letters, digits, `_`, and `.` namespaces. Digits cannot be
the first character of a name segment.

Reserved words cannot be used as names.

## Arithmetic

Only four arithmetic operators exist in the base language:

```apl
+
-
*
/
```

Compound assignments are allowed:

```apl
x += 1
x -= 1
x *= 2
x /= 2
```

Rules:

- `Int / Int` returns `Int`;
- no implicit `Int` to `Float` conversion;
- no string concatenation with `+`;
- roots, trigonometry, and heavy math belong in functions or external
  languages.

Power is exposed as `pow(base, exponent)`. The exponent is an integer; negative
exponents return `NONE`. `**` is recognized by the lexer for future grammar
work, but it is not currently an expression operator.

## Comparisons

Supported:

```apl
==
!=
>
<
>=
<=
```

`=self=` checks whether an absolute variable still equals its initial declared
value:

```apl
AVInt x = 4
x =self= # true
x = 5
x =self= # false
```

`=self=` is only for absolute variables.

## Conditions

Branching:

```apl
if condition {
} else if condition {
} else {
}
```

Logical operators:

```apl
and
or
not
```

When `and` or `or` are used, operands must be explicitly grouped with
parentheses. This is intentional, because APL conditions may control security
and access policy.

```apl
if (a == true) and (b == true) {
}
```

## Loops

`while` always has a limit:

```apl
while (condition) (limit) {
}
```

Rules:

- `limit` is an integer literal in parentheses;
- `-1` means unlimited;
- values below `-1` are invalid;
- reaching the limit is a safe exit, not an error;
- `break` and `continue` work inside loops.

## Pick

`pick(value): item` iterates over a value and creates `item` as a scoped
`VTime`.

```apl
pick("APL"): ch {
  out ch
}
```

`pick` remains supported, but slices/indexing are preferred for many
architecture/routing tasks.

## Lists

Lists are flexible containers:

```apl
List items = []
List mixed = [1, "abc", true, NONE]
List matrix = [[1, 2], [3, 4]]
```

Lists can contain other lists. This is important for compatibility with Python
nested lists, Java 2D arrays, and similar structures.

Lists themselves do not have a protection family like `AV/ASV/SASV`. Protection
lives on elements.

Element protection defaults to `AV`:

```apl
List items = [1, "abc"]
```

Explicit labels:

```apl
List items = [1:SASV, "abc":ASV, true:AV]
add(items, "hidden":ASV)
```

Minimal list operations:

```apl
add(items, value)
VTime x = get(items, 0)
VTime y = pop(items)
```

`get` and `pop` return `VTime` values carrying the element protection label.

## Indexing And Slices

Supported:

```apl
VTime first = values[0]
VTime cell = matrix[0][1]
VTime head = values[:3]
VTime tail = values[1:]
VTime every_second = values[::2]
VTime reversed = values[::-1]
VTime middle = values[1:3:1]
```

Slice shape:

```apl
value[start?:end?:step?]
```

`start`, `end`, and `step` are optional. Index and slice bounds must be `Int` or
`VTime`. Negative step is supported for reverse slices. A zero step returns
`NONE` instead of crashing.

No special nested syntax like `get(list, 0(0))`. Users can write:

```apl
VTime row = get(matrix, 0)
VTime cell = get(row, 0)
```

or:

```apl
VTime cell = matrix[0][0]
```

## Input And Output

Output:

```apl
out value
```

`out` is an external channel and must not output secret values. This includes
nested list values: if any element inside a list is `ASV` or `SASV`, rendering
the whole list to `out` is denied.

Input:

```apl
AVStr raw = input
AVInt age = input
ASVStr token = secret input
```

Bad typed input should become `NONE`, not crash the program. Most real input
validation is expected to happen in external languages such as Python later.

## Info

`info()` returns metadata into two `AVStr` variables:

```apl
AVStr typ = ""
AVStr level = ""

typ, level = info(master_key)
```

It returns:

- declared type as string;
- protection family as string.

`info()` works for absolute variables, including `SASV`. It does not work for
`VTime`.

## Functions

Functions:

```apl
func name(arg1, arg2) {
  return arg1
}
```

Call syntax:

```apl
AVInt result = add_one(4)
```

Function arguments are scoped `VTime` variables. Return values are dynamic and
are checked when written into typed absolute variables. Function parameters are
local to the function body; the APL bootstrap checker uses them while validating
the body, but does not leak parameter names into the outer declaration
namespace.

## Conversions

Explicit conversions:

```apl
int(x)
float(x)
bool(x)
str(x)
bytes(x)
json(x)
```

On invalid input, conversion returns `NONE`; it should not crash the program.

## Secretup

`secretup(x)` raises an absolute variable's protection:

```apl
AVStr token = input
secretup(token) # AV -> ASV
secretup(token) # ASV -> SASV
```

Downgrades are never allowed. `secretup()` does not apply to `VTime`.

## Stop And Fail

```apl
stop
stop "done"
fail "bad config"
```

`stop` exits normally. `fail` exits with failure.

`stop "reason"` and `fail "reason"` are also external channels, so secret
reasons are denied. `stop` inside a function stops the whole program, not only
that function call.

## Current Project State

Rust workspace:

- `apl_core`: AST and semantic checker.
- `apl_parser`: lexer/parser.
- `apl_runtime`: runtime v0.1 interpreter for the current base language.
- `apl_ir`: binary `.aplc` IR encoder/decoder for checked APL programs.
- `apl_compiler`: generates a standalone Rust package from APL source plus the
  APL runtime prelude. It also exposes a narrow host bridge that asks the
  APL-written bootstrap pipeline to compile source into portable `APLLINK2`
  artifacts and to load, verify, and run those artifacts. The bridge passes an
  explicit input count because the outer Rust-hosted `input` uses an empty
  string at exhaustion while the inner APL VM uses `NONE`. Linked compilation
  composes `std/runtime.apl` and user source inside the APL bridge before
  checking and lowering. Runtime helpers and user functions therefore receive
  one function table and one `CALL_SLOT` namespace without copying compiler
  implementation modules into the artifact.
- `apl_cli`: CLI with source `check`/`run`, host-side `.aplc` `emit`/`run-ir`,
  legacy Rust/`.aplc` package `build`/`compile`, APL-owned portable artifact
  `emit-linked`/`run-linked`, and portable-artifact package
  `build-linked`/`compile-linked` commands.
- `apl_router`: placeholder for future router/container phase.

APL-owned runtime code:

- The standard prelude is composed from APL files in `std/`.
- `std/runtime.apl`: core runtime helpers written in APL.
- Current core functions: `apl.is_none`, `apl.not_none`, `apl.first`,
  `apl.pow_int`.
- `std/lexer.apl`: the first APL-written lexer helpers:
  `lexer.is_keyword`, `lexer.is_digit`, `lexer.is_int`,
  `lexer.is_space`, `lexer.is_single_symbol`, `lexer.make_token`,
  `lexer.token_kind`, `lexer.token_value`, `lexer.token_type`, and
  `lexer.tokenize`.
- `lexer.tokenize(source)` now scans source text character by character in APL.
  It returns token records as nested lists: `[kind, value]`. It recognizes
  whitespace, `#` line comments, quoted string literals, `=self=`,
  two-character operators such as `==`, `!=`, `>=`, `<=`, `+=`, and `**`,
  single-character symbols, integer/float literals, keywords, and identifiers.
  It is still a bootstrap lexer, not the final parser.
- `examples/test_lexer.apl` is the current lexer smoke-test. It uses public
  `AVStr` input so the token list can be printed safely.
- `std/parser.apl`: the first APL-written parser bootstrap. It consumes lexer
  tokens and returns AST records as nested lists. Current node coverage:
  absolute variable declarations, `List` declarations, `VTime` declarations,
  `info()` assignments, simple assignments, expression statements,
  `if/else if/else` blocks, `while(condition)(limit)` blocks, `pick(value): item` blocks, function
  declarations, `return`, `break`, `continue`, `secretup`, `out`, `stop`, and
  `fail`.
  Bounded-loop parsing accepts both non-negative integer limits and the
  tokenized negative literal `-1`, matching the Rust parser and the language's
  unlimited-loop syntax.
  Current expression coverage: int, float, string, bool, `NONE`, `input`,
  `secret input`, variable references, list literals, tagged values `value:ASV` / `value:SASV`,
  function/builtin calls, postfix indexing/slicing, unary `-`/`not`
  expressions, and binary expressions with precedence levels for `*`/`/`,
  `+`/`-`, comparisons, `and`, and `or`.
- `examples/test_parser.apl` is the current parser smoke-test.
- `std/checker.apl`: the first APL-written semantic checker layer. It validates
  duplicate declaration/function names before IR lowering, including names found
  in nested `if`, `while`, `pick`, and function bodies. It also rejects unknown
  assignment targets, unknown/invalid `secretup` targets, and unknown/invalid
  `info()` targets/sources before VM execution. Assignments are limited to
  absolute variables or `VTime`, and compound assignments are limited to
  `VTime` or public numeric absolute variables. This moves the global name
  uniqueness and basic mutation-target rules into compiled APL-owned runtime
  code. Expression validation rejects unknown variable reads across
  declarations, assignments, output, conditions, function-call arguments, list
  literals, indexing/slicing, tags, and `=self=` targets. Function-call
  validation accepts known VM builtins, accepts user functions including
  top-level forward calls, rejects unknown function names, rejects attempts to
  call non-function values, and checks user-function argument counts.
- `examples/test_checker.apl`, `examples/test_checker_targets.apl`, and
  `examples/test_checker_exprs.apl`, and `examples/test_checker_calls.apl` are
  the checker smoke-tests.
- `std/ir.apl`: the first APL-written IR bootstrap. It lowers the parser AST
  into list-based IR instructions. Current instruction coverage mirrors the
  parser bootstrap: `DECL`, `ASSIGN`, `IF`, `WHILE`, `PICK`, `BREAK`,
  `CONTINUE`, `FUNC`, `RETURN`, `SECRETUP`, `LIST_DECL`, `VTIME_DECL`,
  `INFO_ASSIGN`, `EXPR`, `OUT`, `STOP`, `FAIL`, and `ERROR`.
  Current expression IR includes `LITERAL`, `LOAD`, `NONE`, `UNARY`,
  `BINARY`, `CALL`, `LIST`, `INDEX`, `SLICE`, `SELF`, and `TAG`.
  Omitted slice bounds are lowered to canonical `NONE` expressions, preserving
  their type through linked-artifact serialization.
- `examples/test_ir.apl` is the current IR smoke-test.
- `std/linker.apl`: the APL-written link stage. It recursively walks entry code,
  nested blocks, expression trees, and function bodies. User-function `CALL`
  expressions are resolved once to numeric `CALL_SLOT` expressions; builtin
  calls remain named. It emits versioned `APLLOAD2` images.
- `std/verifier.apl`: the APL-written loaded-image verifier. It recursively
  validates expression and instruction opcodes/arity, operators, literal types,
  nested blocks, builtin arity, function-table consistency, unique function and
  parameter names, loop limits, and `CALL_SLOT` bounds before VM execution.
- `std/artifact.apl`: the APL-written portable codec for linked images. It
  emits `APLLINK2:` wire strings with tagged scalar nodes, length-prefixed
  payloads, and recursive lists. The APL decoder reconstructs the linked image
  without source parsing and rejects bad headers, malformed/truncated nodes,
  unknown tags, trailing data, and structures rejected by the APL verifier.
- `std/vm.apl`: the first APL-written VM bootstrap. It executes the list-based
  IR from `std/ir.apl`, keeps an append-only environment as
  `[names, values, kinds, types, initials]`,
  supports declarations with `AV`/`ASV`/`SASV` metadata, dynamic `VTime` values,
  lists, `get`/`len`/`add`/`pop`, `info()` metadata reads, assignments, nested
  `if/else if/else` blocks, bounded `while` blocks, loop flow with `break`/`continue`,
  `pick` iteration with scoped item cleanup, function tables, function calls,
  `return`, `secretup`, `out`, `stop`, and `fail`, and returns captured output
  as a list of strings. Direct and derived `out` of `ASV`/`SASV` values is
  denied by the APL
  VM bootstrap for variable loads, `VTime` copies, binary expressions, list
  literals, and `get` from secret-derived lists. The VM also rejects attempts
  to write `ASV`/`SASV` values into lower-protection absolute variables, while
  `VTime` and list values can continue carrying their own protection labels.
  Duplicate runtime declarations are rejected in the APL VM instead of silently
  shadowing absolute/list names; duplicate function names make the VM run return
  an empty output list. Assignment and `secretup` against unknown names are
  rejected by the VM instead of creating implicit variables. Compound assignment
  in the VM is limited to `VTime` or public numeric absolute variables, matching
  the Rust checker contract. `add`/`pop` only mutate existing `List` or `VTime`
  targets; invalid list mutation returns `NONE` without changing VM state.
  `info()` in the VM requires existing `AVStr` targets and rejects `VTime`
  sources; `info(SASV)` remains allowed because it exposes metadata, not value.
  The same external-channel rule is applied to `stop`/`fail` reasons: public
  reasons are captured, secret reasons become `DENIED`. VM expressions can now evaluate variable loads,
  literals, list literals, builtin calls, indexing, slicing, function calls,
  unary `-`/`not`, arithmetic operators `+`, `-`, `*`, `/`, comparison
  operators `==`, `!=`, `>`, `<`, `>=`, `<=`, grouped expressions, and logical
  operators `and`/`or`.
  `=self=` compares the current absolute-variable value against the initial
  declaration value kept in the VM environment. Conversion calls `int`, `float`,
  `bool`, `str`, `bytes`, and `json` are VM builtins and preserve the source
  value protection label. String/list helpers `split`, `join`, `contains`,
  `ord`, `char`, and `pow` are also available inside VM-executed code and
  preserve the strongest argument protection label. `secretup(name)` raises
  absolute-variable protection from `AV` to `ASV` and from `ASV` to `SASV`
  while preserving initial values. Tagged values `value:ASV` and `value:SASV`
  raise VM expression protection. VM list values keep a parallel per-element
  protection label table for `get`, `pop`, and slices; whole-list external
  output still uses the strongest aggregate list protection. `input` and
  `secret input` in VM-executed code consume an explicit input stream through
  `vm.run_source_with_input(source, inputs)` / `vm.run_ir_with_input(program, inputs)`.
  Exhausted input becomes `NONE`; `secret input` carries `ASV`
  protection. `vm.run_source(source)` still uses an empty stream. Absolute
  declarations and `=` assignments in VM-executed code coerce values to their
  declared type, so invalid typed input becomes `NONE`. The APL parser bootstrap
  handles arithmetic, comparison, and logical expression precedence before lowering to IR. This proves
  runtime behavior can be compiled and executed from APL code itself.
  `vm.load_ir_report(program)` prepares a linked image
  `["APLLOAD2", entry, functions]` by collecting the VM function table once,
  removing top-level `FUNC` instructions from the executable entry program, and
  asking the APL linker to resolve user calls to numeric function slots in both
  entry code and function bodies. `vm.run_loaded*` executes that image repeatedly
  without re-collecting functions, searching user functions by name, or stepping
  over function declarations as no-op runtime instructions. The legacy
  `vm.run_ir*` facade also loads and links before execution. Every loaded image,
  including an image decoded from untrusted portable text, must pass
  `verifier.loaded_image_is_valid` before execution.
- `std/bootstrap.apl`: the public APL-level facade over the bootstrap compiler
  and VM pipeline. It exposes `bootstrap.tokens(source)`,
  `bootstrap.ast(source)`, `bootstrap.ir(source)`,
  `bootstrap.compile_report(source)`, `bootstrap.artifact_report(source)`,
  `bootstrap.artifact(source)`, `bootstrap.artifact_image_report(source)`,
  `bootstrap.artifact_image(source)`, `bootstrap.run_artifact_image(image)`,
  `bootstrap.run_artifact_image_report(image)`,
  `bootstrap.run_artifact_image_with_input(image, inputs)`,
  `bootstrap.run_artifact_image_with_input_report(image, inputs)`,
  `bootstrap.load_artifact_image_report(image)`,
  `bootstrap.load_artifact_image(image)`, `bootstrap.loaded_image_report(source)`,
  `bootstrap.loaded_image(source)`, `bootstrap.run_loaded_image(loaded)`,
  `bootstrap.linked_artifact_report(source)`,
  `bootstrap.linked_artifact(source)`,
  `bootstrap.load_linked_artifact_report(encoded)`,
  `bootstrap.load_linked_artifact(encoded)`,
  `bootstrap.run_linked_artifact(encoded)`,
  `bootstrap.run_linked_artifact_report(encoded)`,
  `bootstrap.run_linked_artifact_with_input(encoded, inputs)`,
  `bootstrap.run_linked_artifact_with_input_report(encoded, inputs)`,
  `bootstrap.run_loaded_image_report(loaded)`,
  `bootstrap.run_loaded_image_with_input(loaded, inputs)`,
  `bootstrap.run_loaded_image_with_input_report(loaded, inputs)`,
  `bootstrap.run(source)`, and `bootstrap.run_with_input(source, inputs)`, so
  compiled APL programs can drive the APL-written runtime without directly
  stitching lexer/parser/IR/VM calls.
  Compile reports return `[OK, program]` or `[FAIL, message]` and prevent VM
  execution when parser/IR lowering produced `ERROR`.
  Artifact reports currently return `[OK, "APLIR1:..."]` or `[FAIL, message]`.
  This is a portable text IR image produced from APL code; binary `.aplc` and
  native output remain host-side/future stages.
  Artifact image reports return `[OK, ["APLIR1", program]]` or
  `[FAIL, message]`. These structured images can be run repeatedly through
  `bootstrap.run_artifact_image*` without re-tokenizing, re-parsing, or
  re-checking the original source text.
  Loaded image reports return `[OK, ["APLLOAD2", entry, functions]]` or
  `[FAIL, output]`. They also precompute the APL VM function table and store an
  entry program without top-level `FUNC` declarations. The APL linker rewrites
  user calls in the complete image to `CALL_SLOT`, so repeated
  `bootstrap.run_loaded_image*` calls do not re-tokenize, re-parse, re-check,
  re-collect or look up functions from the original source, or execute function
  declarations as no-op instructions.
  Portable linked artifact reports return `[OK, "APLLINK2:..."]` or
  `[FAIL, message]`. Their payload is encoded and decoded entirely by APL code;
  a decoded artifact is the same `APLLOAD2` image consumed by the loaded-image
  execution APIs and must pass the same structural verifier before it can run.
  The Rust compiler facade and CLI now expose this path through
  `compile_linked_artifact`, `emit_linked_artifact_file`,
  `run_linked_artifact`, `emit-linked`, and `run-linked`. The default file
  extension is `.apllink`; this is a portable bootstrap wire format, not native
  machine code and not the host-side binary `.aplc` format. The compiler bridge
  prepends the small APL runtime module as a separate input and performs source
  composition in APL before checking, IR lowering, function-slot linking, and
  artifact encoding. This is the first standard-module composition step; image
  merging after `CALL_SLOT` assignment remains intentionally unsupported.
  The source-level `bootstrap.run_report(source)` and
  `bootstrap.run_with_input_report(source, inputs)` facades now compile to a
  loaded image and execute that loaded image instead of directly calling
  `vm.run_ir*`.
  Status-preserving variants `bootstrap.run_report(source)` and
  `bootstrap.run_with_input_report(source, inputs)` return `[status, output]`,
  where status is `OK`, `STOP`, or `FAIL`.
- `examples/test_vm.apl`, `examples/test_vm_if.apl`,
  `examples/test_vm_else_if.apl`, and `examples/test_vm_else.apl`,
  `examples/test_vm_while.apl`, and
  `examples/test_vm_loop_flow.apl`, `examples/test_vm_func.apl`,
  `examples/test_vm_list.apl`, `examples/test_vm_list_mutation_guards.apl`,
  `examples/test_vm_index_slice.apl`, and
  `examples/test_vm_pick.apl`, `examples/test_vm_security.apl`,
  `examples/test_vm_duplicate_names.apl`, `examples/test_vm_unknown_targets.apl`,
  `examples/test_vm_compound_guards.apl`, `examples/test_vm_info_guards.apl`,
  `examples/test_vm_secret_flow.apl`, `examples/test_vm_secret_downgrade.apl`,
  `examples/test_vm_stop_fail.apl`,
  `examples/test_vm_logic.apl`, `examples/test_vm_precedence.apl`,
  `examples/test_vm_float.apl`, `examples/test_vm_unary.apl`,
  `examples/test_vm_pow.apl`, `examples/test_vm_self.apl`,
  `examples/test_vm_conversions.apl`, `examples/test_vm_secretup.apl`,
  `examples/test_vm_string_helpers.apl`, `examples/test_vm_tagged_list.apl`,
  `examples/test_vm_input.apl`, `examples/test_vm_input_stream.apl`, and
  `examples/test_vm_typed_input.apl` are the
  current VM smoke-tests. `examples/test_bootstrap.apl`,
  `examples/test_bootstrap_report.apl`, and
  `examples/test_bootstrap_compile_report.apl`,
  `examples/test_bootstrap_artifact.apl`, and
  `examples/test_bootstrap_artifact_image.apl`, and
  `examples/test_bootstrap_loaded_image.apl`, and
  `examples/test_bootstrap_linked_artifact.apl` are smoke-tests for the
  APL-level bootstrap facade.

Rust-hosted builtins that support the APL prelude:

- `split(value, separator)`: splits `Str`/`VTime` string values into a `List`.
- `len(value)`: returns length for `Str`, `Bytes`, `List`, or matching `VTime`.
- `contains(value, needle)`: checks string containment.
- `join(values, separator)`: joins a `List` of strings into a `Str`.
- `ord(value)`: returns the Unicode code point of the first character in a
  string.
- `char(value)`: returns a one-character string from an integer Unicode code
  point.
- `pow(base, exponent)`: raises an `Int` or `Float` base to a non-negative
  integer exponent and returns `NONE` for negative exponents or invalid inputs.
- `split`, `contains`, and `pow` preserve the strongest protection level of their
  inputs. Secret-derived pieces remain secret and cannot be sent to `out`.
- Local `VTime` lists can be mutated with `add(vtime_list, value)` and
  `pop(vtime_list)`, which lets APL runtime code build temporary buffers inside
  functions.
- Slice typing is shape-preserving in the checker: `Str[:] -> Str`,
  `Bytes[:] -> Bytes`, `List[:] -> List`, and dynamic `VTime[:] -> VTime`.

Compiled runtime path:

- `cargo run -p apl -- emit examples\compiled_runtime.apl build\compiled_runtime.aplc`
  parses/checks APL plus the prelude and writes a binary `.aplc` artifact.
- `cargo run -p apl -- run-ir build\compiled_runtime.aplc` decodes and executes
  that artifact without parsing APL source text.
- `cargo run -p apl -- build examples\compiled_runtime.apl build\compiled_runtime`
  validates the APL prelude plus user source and generates a separate Rust
  package.
- `cargo build --manifest-path build\compiled_runtime\Cargo.toml` compiles that
  generated package.
- `cargo run -p apl -- compile examples\compiled_runtime.apl build\compiled_runtime`
  generates the package and invokes `cargo build` for it.
- The generated executable embeds `.aplc` IR bytes and enters through
  `apl_runtime::run_ir_bytes`.
- `examples/bootstrap_runtime.apl` is the current self-host smoke-test: a
  generated standalone executable embeds compiled APL IR, runs the APL-written
  lexer/parser/IR/VM from the standard prelude through `bootstrap.run_with_input`,
  and that VM executes a nested APL program with input, secret input, functions,
  nested lists, typed input coercion, and secret-aware output.
- `apl_runtime::compile_ir_bytes` loads IR into `CompiledProgram`, splitting
  executable entry code from the function table before execution.
- `CompiledProgram` stores one linear statement opcode `code` segment. Blocks
  are ranges inside that segment, and entry code/function bodies point at those
  ranges by `BlockId`.
- Expressions are lowered into runtime-owned stack opcodes in
  `CompiledExpression`.
- `apl_runtime::run_compiled_program` executes that loaded compiled object.
- Generated executables do not depend on `apl_parser` and do not parse APL
  source text at startup. Decode/execution is behind the runtime API so the
  future VM can replace AST execution without changing compiled program entry.
- The Rust host runtime now executes statement opcodes from the linear `code`
  segment through `BlockFrame` values with an explicit program counter (`pc`)
  and uses a small expression VM.

Portable linked package path:

- `cargo run -p apl -- build-linked examples\compiled_runtime.apl build\compiled_runtime_linked`
  asks the APL-written pipeline to compose `std/runtime.apl` with user source,
  check and lower it, assign function slots, and encode `program.apllink`.
- `cargo run -p apl -- compile-linked examples\compiled_runtime.apl build\compiled_runtime_linked`
  also invokes Cargo for the generated launcher package.
- The generated launcher embeds `program.apllink` with `include_str!` and calls
  `apl_compiler::run_linked_artifact`. Artifact decoding, structural
  verification, and VM execution remain APL-owned.
- The launcher and loader remain Rust-hosted bootstrap code; `.apllink` is not
  native machine code.
- Legacy `build`/`compile` remains available for self-host examples that execute
  compiler modules from the complete standard prelude. Compiling that entire
  compiler source through the currently interpreted APL lexer/parser on every
  build is not yet practical; a preloaded compiler image is the next migration
  layer.
- `if/else if/else` is compiled into `JumpIfFalse`, `ExecScopedBlock`, and
  `Jump` statement opcodes.
- `while` is compiled into `LoopCheck`, `ExecLoopBody`, and `Jump` statement
  opcodes. Loop iteration counts live in the current `BlockFrame`.
- Direct machine-code generation is a later stage.
- This is the first bootstrap-friendly step, not final native compilation.

Runtime v0.1 covers:

- absolute variables, typed assignment, `NONE`, `=self=`;
- `VTime` scopes in functions/blocks/loops;
- functions and return values;
- `if/else if/else`;
- `while(condition)(limit)`, `break`, `continue`;
- `pick`;
- list literals, nested lists, `add`, `get`, `pop`;
- indexing and slicing;
- string/list helper builtins: `split`, `len`, `contains`, `join`, `ord`,
  `char`, `pow`;
- `input`, `secret input`, `out`;
- `info`, conversions, `secretup`, `stop`, `fail`;
- recursive list protection checks before output/error text.
- secret downgrade rejection for absolute variables.
- duplicate declaration rejection in the APL VM.
- unknown mutation target rejection in the APL VM.
- compound assignment guards in the APL VM.
- list mutation guards in the APL VM.
- `info()` target/source guards in the APL VM.

Useful commands:

```powershell
cd D:\veles\apl
cargo test
cargo run -p apl -- check examples\hello.apl
cargo run -p apl -- run examples\hello.apl
cargo run -p apl -- run examples\calculator.apl
cargo run -p apl -- emit examples\compiled_runtime.apl build\compiled_runtime.aplc
cargo run -p apl -- run-ir build\compiled_runtime.aplc
cargo run -p apl -- build examples\compiled_runtime.apl build\compiled_runtime
cargo build --manifest-path build\compiled_runtime\Cargo.toml
cargo run -p apl -- compile examples\compiled_runtime.apl build\compiled_runtime
build\compiled_runtime\target\debug\compiled_runtime_compiled.exe
.\emit_aplc.bat examples\compiled_runtime.apl build\compiled_runtime.aplc
.\run_aplc.bat build\compiled_runtime.aplc
.\compile_apl.bat examples\compiled_runtime.apl build\compiled_runtime_bat
.\emit_aplc.bat examples\bootstrap_runtime.apl build\bootstrap_runtime.aplc
.\run_aplc.bat build\bootstrap_runtime.aplc
.\compile_apl.bat examples\bootstrap_runtime.apl build\bat_bootstrap_runtime
build\bat_bootstrap_runtime\target\debug\bootstrap_runtime_compiled.exe
.\emit_aplc.bat examples\test_bootstrap.apl build\test_bootstrap.aplc
.\run_aplc.bat build\test_bootstrap.aplc
.\compile_apl.bat examples\test_bootstrap.apl build\bat_test_bootstrap
build\bat_test_bootstrap\target\debug\test_bootstrap_compiled.exe
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
.\emit_aplc.bat examples\test_bootstrap_linked_artifact.apl build\test_bootstrap_linked_artifact.aplc
.\run_aplc.bat build\test_bootstrap_linked_artifact.aplc
.\compile_apl.bat examples\test_bootstrap_linked_artifact.apl build\bat_test_bootstrap_linked_artifact
build\bat_test_bootstrap_linked_artifact\target\debug\test_bootstrap_linked_artifact_compiled.exe
.\emit_aplc.bat examples\test_lexer.apl build\test_lexer.aplc
.\run_aplc.bat build\test_lexer.aplc
.\compile_apl.bat examples\test_lexer.apl build\bat_test_lexer
build\bat_test_lexer\target\debug\test_lexer_compiled.exe
.\emit_aplc.bat examples\test_parser.apl build\test_parser.aplc
.\run_aplc.bat build\test_parser.aplc
.\compile_apl.bat examples\test_parser.apl build\bat_test_parser
build\bat_test_parser\target\debug\test_parser_compiled.exe
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
.\emit_aplc.bat examples\test_ir.apl build\test_ir.aplc
.\run_aplc.bat build\test_ir.aplc
.\compile_apl.bat examples\test_ir.apl build\bat_test_ir
build\bat_test_ir\target\debug\test_ir_compiled.exe
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
powershell -ExecutionPolicy Bypass -File tools\apl_gui.ps1
```

The minimal GUI is implemented in `tools/apl_gui.ps1`.
Quick Windows compiler wrappers live at repository root:
`compile_apl.bat`, `compile_linked_apl.bat`, `emit_aplc.bat`, and
`run_aplc.bat`.
