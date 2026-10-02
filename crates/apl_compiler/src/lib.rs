use std::{
    fs,
    path::{Path, PathBuf},
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
    let output = run_bootstrap_bridge(
        LINKED_COMPILE_BRIDGE,
        vec![RUNTIME_PRELUDE.to_owned(), source.to_owned()],
    )?;
    output
        .stdout
        .strip_suffix('\n')
        .map(str::to_owned)
        .filter(|artifact| !artifact.is_empty())
        .ok_or_else(|| CompileError::Bootstrap("APL compiler returned no artifact".to_owned()))
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
    let program = apl_parser::parse_program(&source)?;
    apl_core::validate_program(&program)?;
    Ok(apl_runtime::run_program(&program, input)?)
}

const LINKED_COMPILE_BRIDGE: &str = r#"VTime aplhost.runtime_source = input
VTime aplhost.program_source = input
VTime aplhost.source = join([aplhost.runtime_source, char(10), aplhost.program_source], "")
VTime aplhost.compile_report = bootstrap.linked_artifact_report(aplhost.source)
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
}
