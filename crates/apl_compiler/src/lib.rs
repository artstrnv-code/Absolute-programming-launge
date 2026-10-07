use std::{
    fs,
    path::{Path, PathBuf},
    thread,
};

pub const RUNTIME_PRELUDE: &str = include_str!("../../../std/runtime.apl");
pub const LEXER_PRELUDE: &str = include_str!("../../../std/lexer.apl");
pub const PARSER_PRELUDE: &str = include_str!("../../../std/parser.apl");
pub const CHECKER_PRELUDE: &str = include_str!("../../../std/checker.apl");
pub const IR_PRELUDE: &str = include_str!("../../../std/ir.apl");
pub const LINKER_PRELUDE: &str = include_str!("../../../std/linker.apl");
pub const VERIFIER_PRELUDE: &str = include_str!("../../../std/verifier.apl");
pub const VM_PRELUDE: &str = include_str!("../../../std/vm.apl");
pub const ARTIFACT_PRELUDE: &str = include_str!("../../../std/artifact.apl");
pub const BOOTSTRAP_PRELUDE: &str = include_str!("../../../std/bootstrap.apl");
pub const STANDARD_PRELUDE: &str = concat!(
    include_str!("../../../std/runtime.apl"),
    "\n\n",
    include_str!("../../../std/lexer.apl"),
    "\n\n",
    include_str!("../../../std/parser.apl"),
    "\n\n",
    include_str!("../../../std/checker.apl"),
    "\n\n",
    include_str!("../../../std/ir.apl"),
    "\n\n",
    include_str!("../../../std/linker.apl"),
    "\n\n",
    include_str!("../../../std/verifier.apl"),
    "\n\n",
    include_str!("../../../std/vm.apl"),
    "\n\n",
    include_str!("../../../std/artifact.apl"),
    "\n\n",
    include_str!("../../../std/bootstrap.apl")
);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildOutput {
    pub package_dir: PathBuf,
    pub package_name: String,
    pub ir_path: PathBuf,
    pub manifest_path: PathBuf,
    pub source_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkedBuildOutput {
    pub package_dir: PathBuf,
    pub package_name: String,
    pub artifact_path: PathBuf,
    pub manifest_path: PathBuf,
    pub source_path: PathBuf,
}

#[derive(Debug)]
pub enum CompileError {
    Io(std::io::Error),
    Ir(apl_ir::IrError),
    Parse(apl_parser::ParseError),
    Check(apl_core::CheckError),
    Runtime(apl_runtime::RuntimeError),
    Bootstrap(String),
}

impl From<std::io::Error> for CompileError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<apl_parser::ParseError> for CompileError {
    fn from(error: apl_parser::ParseError) -> Self {
        Self::Parse(error)
    }
}

impl From<apl_ir::IrError> for CompileError {
    fn from(error: apl_ir::IrError) -> Self {
        Self::Ir(error)
    }
}

impl From<apl_core::CheckError> for CompileError {
    fn from(error: apl_core::CheckError) -> Self {
        Self::Check(error)
    }
}

impl From<apl_runtime::RuntimeError> for CompileError {
    fn from(error: apl_runtime::RuntimeError) -> Self {
        Self::Runtime(error)
    }
}

pub fn compile_linked_artifact(source: &str) -> Result<String, CompileError> {
    compile_linked_artifact_with_module(RUNTIME_PRELUDE, source)
}

pub fn compile_linked_artifact_with_module(
    module_source: &str,
    source: &str,
) -> Result<String, CompileError> {
    let output = run_bootstrap_bridge(
        LINKED_COMPILE_BRIDGE,
        vec![module_source.to_owned(), source.to_owned()],
    )?;
    bootstrap_stdout_artifact(output, "linked artifact")
}

pub fn compile_module_artifact(source: &str) -> Result<String, CompileError> {
    let output = run_bootstrap_bridge(MODULE_COMPILE_BRIDGE, vec![source.to_owned()])?;
    bootstrap_stdout_artifact(output, "module artifact")
}

pub fn compile_standard_module_artifact() -> Result<String, CompileError> {
    compile_module_artifact_sources(&[
        RUNTIME_PRELUDE,
        LEXER_PRELUDE,
        PARSER_PRELUDE,
        CHECKER_PRELUDE,
        IR_PRELUDE,
        LINKER_PRELUDE,
        VERIFIER_PRELUDE,
        VM_PRELUDE,
        ARTIFACT_PRELUDE,
        BOOTSTRAP_PRELUDE,
    ])
}

pub fn compile_module_artifact_sources(sources: &[&str]) -> Result<String, CompileError> {
    let mut input = Vec::with_capacity(sources.len() + 1);
    input.push(sources.len().to_string());
    input.extend(sources.iter().map(|source| (*source).to_owned()));

    let output = run_bootstrap_bridge(MODULE_SOURCES_COMPILE_BRIDGE, input)?;
    bootstrap_stdout_artifact(output, "module artifact")
}

pub fn emit_standard_module_artifact_file(output_path: &Path) -> Result<(), CompileError> {
    fs::write(output_path, compile_standard_module_artifact()?)?;
    Ok(())
}

pub fn emit_module_artifact_file(
    source_path: &Path,
    output_path: &Path,
) -> Result<(), CompileError> {
    let source = fs::read_to_string(source_path)?;
    let artifact = compile_module_artifact(&source)?;
    fs::write(output_path, artifact)?;
    Ok(())
}

pub fn extend_module_artifact(module_artifact: &str, source: &str) -> Result<String, CompileError> {
    let output = run_bootstrap_bridge(
        MODULE_EXTEND_BRIDGE,
        vec![module_artifact.to_owned(), source.to_owned()],
    )?;
    bootstrap_stdout_artifact(output, "module artifact")
}

pub fn emit_extended_module_artifact_file(
    module_path: &Path,
    source_path: &Path,
    output_path: &Path,
) -> Result<(), CompileError> {
    let module = fs::read_to_string(module_path)?;
    let source = fs::read_to_string(source_path)?;
    let artifact = extend_module_artifact(&module, &source)?;
    fs::write(output_path, artifact)?;
    Ok(())
}

pub fn compile_linked_artifact_with_precompiled_module(
    module_artifact: &str,
    source: &str,
) -> Result<String, CompileError> {
    let output = run_bootstrap_bridge(
        LINKED_MODULE_COMPILE_BRIDGE,
        vec![module_artifact.to_owned(), source.to_owned()],
    )?;
    bootstrap_stdout_artifact(output, "linked artifact")
}

pub fn emit_linked_artifact_with_module_file(
    module_path: &Path,
    source_path: &Path,
    output_path: &Path,
) -> Result<(), CompileError> {
    let module = fs::read_to_string(module_path)?;
    let source = fs::read_to_string(source_path)?;
    let artifact = compile_linked_artifact_with_precompiled_module(&module, &source)?;
    fs::write(output_path, artifact)?;
    Ok(())
}

pub fn emit_linked_artifact_file(
    source_path: &Path,
    output_path: &Path,
) -> Result<(), CompileError> {
    let source = fs::read_to_string(source_path)?;
    let artifact = compile_linked_artifact(&source)?;
    fs::write(output_path, artifact)?;
    Ok(())
}

pub fn run_linked_artifact(
    artifact: &str,
    input: Vec<String>,
) -> Result<apl_runtime::RunOutput, CompileError> {
    let mut bridge_input = Vec::with_capacity(input.len() + 2);
    bridge_input.push(artifact.to_owned());
    bridge_input.push(input.len().to_string());
    bridge_input.extend(input);
    run_bootstrap_bridge(LINKED_RUN_BRIDGE, bridge_input)
}

pub fn build_executable_package(
    source_path: &Path,
    output_dir: &Path,
    runtime_prelude: &str,
) -> Result<BuildOutput, CompileError> {
    let source = fs::read_to_string(source_path)?;
    let program_source = compose_program(runtime_prelude, &source);
    let program = apl_parser::parse_program(&program_source)?;
    apl_core::validate_program(&program)?;
    let ir = apl_ir::encode_program(&program)?;

    let package_dir = output_dir.to_path_buf();
    let src_dir = package_dir.join("src");
    fs::create_dir_all(&src_dir)?;

    let manifest_path = package_dir.join("Cargo.toml");
    let generated_source_path = src_dir.join("main.rs");
    let ir_path = package_dir.join("program.aplc");
    let workspace_root = workspace_root_from_source(source_path);

    let package_name =
        compiled_package_name(source_path.file_stem().and_then(|name| name.to_str()));

    fs::write(
        &manifest_path,
        render_manifest(&workspace_root, &package_name),
    )?;
    fs::write(&ir_path, &ir)?;
    fs::write(&generated_source_path, render_main(&ir))?;

    Ok(BuildOutput {
        package_dir,
        package_name,
        ir_path,
        manifest_path,
        source_path: generated_source_path,
    })
}

pub fn build_linked_executable_package(
    source_path: &Path,
    output_dir: &Path,
) -> Result<LinkedBuildOutput, CompileError> {
    let source = fs::read_to_string(source_path)?;
    let artifact = compile_linked_artifact(&source)?;

    let package_dir = output_dir.to_path_buf();
    let src_dir = package_dir.join("src");
    fs::create_dir_all(&src_dir)?;

    let manifest_path = package_dir.join("Cargo.toml");
    let generated_source_path = src_dir.join("main.rs");
    let artifact_path = package_dir.join("program.apllink");
    let workspace_root = workspace_root_from_source(source_path);
    let package_name =
        compiled_package_name(source_path.file_stem().and_then(|name| name.to_str()));

    fs::write(
        &manifest_path,
        render_linked_manifest(&workspace_root, &package_name),
    )?;
    fs::write(&artifact_path, artifact)?;
    fs::write(&generated_source_path, render_linked_main())?;

    Ok(LinkedBuildOutput {
        package_dir,
        package_name,
        artifact_path,
        manifest_path,
        source_path: generated_source_path,
    })
}

pub fn emit_ir_file(
    source_path: &Path,
    output_path: &Path,
    runtime_prelude: &str,
) -> Result<(), CompileError> {
    let source = fs::read_to_string(source_path)?;
    let program_source = compose_program(runtime_prelude, &source);
    let program = apl_parser::parse_program(&program_source)?;
    apl_core::validate_program(&program)?;
    let ir = apl_ir::encode_program(&program)?;
    fs::write(output_path, ir)?;
    Ok(())
}

pub fn compose_program(runtime_prelude: &str, source: &str) -> String {
    let mut combined = String::new();
    combined.push_str(runtime_prelude.trim());
    combined.push_str("\n\n");
    combined.push_str(source.trim());
    combined.push('\n');
    combined
}

fn run_bootstrap_bridge(
    bridge_source: &str,
    input: Vec<String>,
) -> Result<apl_runtime::RunOutput, CompileError> {
    let source = compose_program(STANDARD_PRELUDE, bridge_source);
    thread::Builder::new()
        .name("apl-bootstrap-compiler".to_owned())
        .stack_size(32 * 1024 * 1024)
        .spawn(move || {
            let program = apl_parser::parse_program(&source)?;
            apl_core::validate_program(&program)?;
            Ok(apl_runtime::run_program(&program, input)?)
        })?
        .join()
        .map_err(|_| CompileError::Bootstrap("APL compiler bridge panicked".to_owned()))?
}

fn bootstrap_stdout_artifact(
    output: apl_runtime::RunOutput,
    artifact_name: &str,
) -> Result<String, CompileError> {
    output
        .stdout
        .strip_suffix('\n')
        .map(str::to_owned)
        .filter(|artifact| !artifact.is_empty())
        .ok_or_else(|| CompileError::Bootstrap(format!("APL compiler returned no {artifact_name}")))
}

const LINKED_COMPILE_BRIDGE: &str = r#"VTime aplhost.runtime_source = input
VTime aplhost.program_source = input
VTime aplhost.source = join([aplhost.runtime_source, char(10), aplhost.program_source], "")
VTime aplhost.compile_report = bootstrap.linked_artifact_report(aplhost.source)
if get(aplhost.compile_report, 0) != vm.FLOW_OK {
  fail get(aplhost.compile_report, 1)
}
out get(aplhost.compile_report, 1)"#;

const MODULE_COMPILE_BRIDGE: &str = r#"VTime aplhost.source = input
VTime aplhost.module_report = bootstrap.module_report(aplhost.source)
if get(aplhost.module_report, 0) != vm.FLOW_OK {
  fail get(aplhost.module_report, 1)
}
out get(aplhost.module_report, 1)"#;

const MODULE_EXTEND_BRIDGE: &str = r#"VTime aplhost.module = input
VTime aplhost.source = input
VTime aplhost.module_report = bootstrap.extend_module_report(aplhost.module, aplhost.source)
if get(aplhost.module_report, 0) != vm.FLOW_OK {
  fail get(aplhost.module_report, 1)
}
out get(aplhost.module_report, 1)"#;

const MODULE_SOURCES_COMPILE_BRIDGE: &str = r#"VTime aplhost.source_count = int(input)
VTime aplhost.sources = []
VTime aplhost.source_index = 0
while (aplhost.source_index < aplhost.source_count) (-1) {
  add(aplhost.sources, input)
  aplhost.source_index += 1
}
VTime aplhost.module_report = bootstrap.modules_report(aplhost.sources)
if get(aplhost.module_report, 0) != vm.FLOW_OK {
  fail get(aplhost.module_report, 1)
}
out get(aplhost.module_report, 1)"#;

const LINKED_MODULE_COMPILE_BRIDGE: &str = r#"VTime aplhost.module = input
VTime aplhost.source = input
VTime aplhost.compile_report = bootstrap.linked_artifact_with_module_report(aplhost.module, aplhost.source)
if get(aplhost.compile_report, 0) != vm.FLOW_OK {
  fail get(aplhost.compile_report, 1)
}
out get(aplhost.compile_report, 1)"#;

const LINKED_RUN_BRIDGE: &str = r#"VTime aplhost.encoded = input
VTime aplhost.loaded_report = bootstrap.load_linked_artifact_report(aplhost.encoded)

if get(aplhost.loaded_report, 0) != vm.FLOW_OK {
  fail get(aplhost.loaded_report, 1)
}

VTime aplhost.inputs = []
VTime aplhost.input_count = int(input)
VTime aplhost.input_index = 0

while (aplhost.input_index < aplhost.input_count) (-1) {
  add(aplhost.inputs, input)
  aplhost.input_index += 1
}

VTime aplhost.run_report = bootstrap.run_loaded_image_with_input_report(get(aplhost.loaded_report, 1), aplhost.inputs)
VTime aplhost.output = get(aplhost.run_report, 1)

pick(aplhost.output): aplhost.line {
  out aplhost.line
}

if get(aplhost.run_report, 0) == vm.FLOW_FAIL {
  fail "linked artifact failed"
}

if get(aplhost.run_report, 0) == vm.FLOW_STOP {
  stop
}"#;

fn compiled_package_name(source_stem: Option<&str>) -> String {
    let name = source_stem
        .map(sanitize_package_name)
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "apl_program".to_owned());
    format!("{name}_compiled")
}

fn render_manifest(workspace_root: &Path, package_name: &str) -> String {
    let core_path = toml_path(&workspace_root.join("crates/apl_core"));
    let runtime_path = toml_path(&workspace_root.join("crates/apl_runtime"));

    format!(
        r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2021"

[workspace]

[dependencies]
apl_core = {{ path = "{core_path}" }}
apl_runtime = {{ path = "{runtime_path}" }}
"#
    )
}

fn render_linked_manifest(workspace_root: &Path, package_name: &str) -> String {
    let compiler_path = toml_path(&workspace_root.join("crates/apl_compiler"));

    format!(
        r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2021"

[workspace]

[dependencies]
apl_compiler = {{ path = "{compiler_path}" }}
"#
    )
}

fn render_main(ir: &[u8]) -> String {
    format!(
        r#"use std::{{io::Read, process}};

fn main() {{
    let mut stdin = String::new();
    let _ = std::io::stdin().read_to_string(&mut stdin);
    let input = stdin.lines().map(str::to_owned).collect();

    match apl_runtime::run_ir_bytes(COMPILED_APL_IR, input) {{
        Ok(output) => print!("{{}}", output.stdout),
        Err(apl_runtime::RuntimeError::Failed(message)) => {{
            eprintln!("APL fail: {{message}}");
            process::exit(1);
        }}
        Err(error) => {{
            eprintln!("APL runtime error: {{error:?}}");
            process::exit(1);
        }}
    }}
}}

const COMPILED_APL_IR: &[u8] = &[
{}
];
"#,
        render_byte_array(ir)
    )
}

fn render_linked_main() -> String {
    r#"use std::{io::Read, process};

fn main() {
    let mut stdin = String::new();
    let _ = std::io::stdin().read_to_string(&mut stdin);
    let input = stdin.lines().map(str::to_owned).collect();

    match apl_compiler::run_linked_artifact(APL_LINKED_ARTIFACT, input) {
        Ok(output) => print!("{}", output.stdout),
        Err(error) => {
            eprintln!("APL linked artifact error: {error:?}");
            process::exit(1);
        }
    }
}

const APL_LINKED_ARTIFACT: &str = include_str!("../program.apllink");
"#
    .to_owned()
}

fn render_byte_array(bytes: &[u8]) -> String {
    bytes
        .chunks(16)
        .map(|chunk| {
            let values = chunk
                .iter()
                .map(|byte| format!("0x{byte:02x}"))
                .collect::<Vec<_>>()
                .join(", ");
            format!("    {values},")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn workspace_root_from_source(source_path: &Path) -> PathBuf {
    source_path
        .canonicalize()
        .ok()
        .and_then(|path| {
            path.ancestors()
                .find(|ancestor| ancestor.join("crates/apl_runtime").exists())
                .map(Path::to_path_buf)
        })
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
}

fn toml_path(path: &Path) -> String {
    path.to_string_lossy()
        .trim_start_matches(r"\\?\")
        .replace('\\', "/")
}

fn sanitize_package_name(raw: &str) -> String {
    raw.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_program_puts_prelude_first() {
        let combined = compose_program("func apl.ok() { return true }", "out apl.ok()");
        assert!(combined.starts_with("func apl.ok()"));
        assert!(combined.contains("out apl.ok()"));
    }

    #[test]
    fn runtime_prelude_is_valid_apl() {
        let program = apl_parser::parse_program(STANDARD_PRELUDE).unwrap();
        apl_core::validate_program(&program).unwrap();
    }

    #[test]
    fn standard_prelude_includes_runtime_and_lexer_modules() {
        assert!(STANDARD_PRELUDE.contains("func apl.pow_int"));
        assert!(STANDARD_PRELUDE.contains("func lexer.tokenize"));
        assert!(STANDARD_PRELUDE.contains("func parser.parse_source"));
        assert!(STANDARD_PRELUDE.contains("func checker.validate_report"));
        assert!(STANDARD_PRELUDE.contains("func ir.compile_source"));
        assert!(STANDARD_PRELUDE.contains("func linker.link_image"));
        assert!(STANDARD_PRELUDE.contains("func verifier.loaded_image_is_valid"));
        assert!(STANDARD_PRELUDE.contains("func vm.run_source"));
        assert!(STANDARD_PRELUDE.contains("func artifact.encode_loaded_report"));
        assert!(STANDARD_PRELUDE.contains("func bootstrap.compile_report"));
        assert!(STANDARD_PRELUDE.contains("func bootstrap.artifact_report"));
        assert!(STANDARD_PRELUDE.contains("func bootstrap.artifact_image_report"));
        assert!(STANDARD_PRELUDE.contains("func bootstrap.run_artifact_image_with_input_report"));
        assert!(STANDARD_PRELUDE.contains("func bootstrap.loaded_image_report"));
        assert!(STANDARD_PRELUDE.contains("func bootstrap.linked_artifact_report"));
        assert!(STANDARD_PRELUDE.contains("func bootstrap.load_linked_artifact_report"));
        assert!(STANDARD_PRELUDE.contains("func bootstrap.module_report"));
        assert!(STANDARD_PRELUDE.contains("func bootstrap.load_module_report"));
        assert!(STANDARD_PRELUDE.contains("func bootstrap.linked_artifact_with_module_report"));
        assert!(STANDARD_PRELUDE.contains("func bootstrap.run_linked_artifact_with_input_report"));
        assert!(STANDARD_PRELUDE.contains("func bootstrap.run_loaded_image_with_input_report"));
        assert!(STANDARD_PRELUDE.contains("func bootstrap.run_with_input"));
        assert!(STANDARD_PRELUDE.contains("func bootstrap.run_with_input_report"));
    }

    #[test]
    fn generated_main_uses_precompiled_ast_without_parser() {
        let source = compose_program(
            STANDARD_PRELUDE,
            "AVInt value = apl.pow_int(2, 3)\nout value",
        );
        let program = apl_parser::parse_program(&source).unwrap();
        apl_core::validate_program(&program).unwrap();
        let ir = apl_ir::encode_program(&program).unwrap();

        let rendered = render_main(&ir);

        assert!(rendered.contains("COMPILED_APL_IR"));
        assert!(rendered.contains("run_ir_bytes"));
        assert!(!rendered.contains("parse_program"));
        assert!(!rendered.contains("PROGRAM_SOURCE"));
    }

    #[test]
    fn apl_bootstrap_compiles_and_runs_portable_artifact() {
        let artifact = compile_linked_artifact(
            "func inc(x) { return x + 1 } AVInt value = input out inc(value)",
        )
        .unwrap();

        assert!(artifact.starts_with("APLLINK2:"));
        let output = run_linked_artifact(&artifact, vec!["4".to_owned()]).unwrap();
        assert_eq!(output.stdout, "5\n");
    }

    #[test]
    fn linked_artifact_bridge_preserves_empty_input_lines() {
        let artifact = compile_linked_artifact(
            "AVStr first = input AVStr second = input out len(first) out second",
        )
        .unwrap();

        let output =
            run_linked_artifact(&artifact, vec![String::new(), "kept".to_owned()]).unwrap();
        assert_eq!(output.stdout, "0\nkept\n");
    }

    #[test]
    fn linked_artifact_includes_apl_runtime_module() {
        let artifact =
            compile_linked_artifact("AVInt value = apl.pow_int(3, 4) out value").unwrap();

        let output = run_linked_artifact(&artifact, vec![]).unwrap();
        assert_eq!(output.stdout, "81\n");
    }

    #[test]
    fn linked_artifact_accepts_an_explicit_apl_module() {
        let artifact = compile_linked_artifact_with_module(
            "func module.double(value) { return value * 2 }",
            "out module.double(21)",
        )
        .unwrap();

        let output = run_linked_artifact(&artifact, vec![]).unwrap();
        assert_eq!(output.stdout, "42\n");
    }

    #[test]
    fn precompiled_apl_module_links_without_module_source() {
        let module = compile_module_artifact(
            "func module.add_one(value) { return value + 1 } func module.twice(value) { return module.add_one(module.add_one(value)) }",
        )
        .unwrap();
        assert!(module.starts_with("APLMOD2:"));

        let artifact =
            compile_linked_artifact_with_precompiled_module(&module, "out module.twice(40)")
                .unwrap();
        let output = run_linked_artifact(&artifact, vec![]).unwrap();
        assert_eq!(output.stdout, "42\n");
    }

    #[test]
    fn precompiled_apl_module_extends_in_stages() {
        let base = compile_module_artifact(
            "AVInt module.base = 40 func module.add_one(value) { return value + 1 }",
        )
        .unwrap();
        let extended = extend_module_artifact(
            &base,
            "List module.extra = [1] func module.total() { return module.add_one(module.base) + get(module.extra, 0) }",
        )
        .unwrap();
        assert!(extended.starts_with("APLMOD2:"));

        let artifact =
            compile_linked_artifact_with_precompiled_module(&extended, "out module.total()")
                .unwrap();
        let output = run_linked_artifact(&artifact, vec![]).unwrap();
        assert_eq!(output.stdout, "42\n");
    }

    #[test]
    fn generated_linked_main_embeds_portable_artifact() {
        let rendered = render_linked_main();

        assert!(rendered.contains("APL_LINKED_ARTIFACT"));
        assert!(rendered.contains("program.apllink"));
        assert!(rendered.contains("run_linked_artifact"));
        assert!(!rendered.contains("run_ir_bytes"));
        assert!(!rendered.contains("parse_program"));
    }
}
