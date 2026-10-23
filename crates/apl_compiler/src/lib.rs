use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
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
pub const NATIVE_PRELUDE: &str = include_str!("../../../std/native.apl");
pub const ABI_PRELUDE: &str = include_str!("../../../std/abi.apl");
pub const ASSEMBLY_PRELUDE: &str = include_str!("../../../std/assembly.apl");
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
    include_str!("../../../std/native.apl"),
    "\n\n",
    include_str!("../../../std/abi.apl"),
    "\n\n",
    include_str!("../../../std/assembly.apl"),
    "\n\n",
    include_str!("../../../std/bootstrap.apl")
);

const BOOTSTRAP_COMPILER_PROTOCOL: &str = "APLCOMP2:";
const BOOTSTRAP_COMPILER_LIBRARY_REQUEST: &str = "APLCREQ1:";
const NATIVE_COMPILER_PROTOCOL: &str = "APLNATIVE1:";
const NATIVE_COMPILER_REQUEST: &str = "APLNREQ1:";
const LEGACY_BOOTSTRAP_COMPILER_OK: &str = "APLCOMP1:O";
const LEGACY_BOOTSTRAP_COMPILER_ERROR: &str = "APLCOMP1:E";
const LINKED_RUN_PROTOCOL: &str = "APLRUN1:";
const LINKED_ARTIFACT_PROTOCOL: &str = "APLLINK3:";
pub const NATIVE_IR_PROTOCOL: &str = "APLLOW1:";
pub const NATIVE_ABI_PROTOCOL: &str = "APLABI1:";
pub const WINDOWS_X64_TARGET: &str = "x86_64-pc-windows-msvc";
pub const LINUX_X64_TARGET: &str = "x86_64-unknown-linux-gnu";
const LEGACY_LINKED_ARTIFACT_PROTOCOL: &str = "APLLINK2:";
const LIBRARY_DISCOVERY_PROTOCOL: &str = "APLLIB1:";
const NATIVE_INPUT_PROTOCOL: &[u8] = b"\0APLIN1:";
const LIBRARY_DIRECTORY: &str = "libraries";
const LIBRARY_EXTENSION: &str = "aplmod";

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeBuildOutput {
    pub package_dir: PathBuf,
    pub package_name: String,
    pub assembly_path: PathBuf,
    pub manifest_path: PathBuf,
    pub source_path: PathBuf,
    pub target: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkedRunStatus {
    Ok,
    Stop,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkedRunReport {
    pub status: LinkedRunStatus,
    pub output: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedLibrary {
    pub name: String,
    pub artifact: String,
}

#[derive(Debug)]
pub enum CompileError {
    Io(std::io::Error),
    Ir(apl_ir::IrError),
    Parse(apl_parser::ParseError),
    Check(apl_core::CheckError),
    Runtime(apl_runtime::RuntimeError),
    Bootstrap(String),
    LibraryNotFound { name: String, path: PathBuf },
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
        NATIVE_PRELUDE,
        ABI_PRELUDE,
        ASSEMBLY_PRELUDE,
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

pub fn compile_bootstrap_compiler_artifact() -> Result<String, CompileError> {
    let standard_module = compile_standard_module_artifact()?;
    let runtime_module = compile_module_artifact(RUNTIME_PRELUDE)?;
    let driver = render_bootstrap_compiler_driver(&runtime_module);
    compile_linked_artifact_with_precompiled_module(&standard_module, &driver)
}

pub fn compile_native_toolchain_artifact() -> Result<String, CompileError> {
    let standard_module = compile_standard_module_artifact()?;
    let runtime_module = compile_module_artifact(RUNTIME_PRELUDE)?;
    let driver = render_native_toolchain_driver(&runtime_module);
    compile_linked_artifact_with_precompiled_module(&standard_module, &driver)
}

pub fn emit_bootstrap_compiler_artifact_file(output_path: &Path) -> Result<(), CompileError> {
    fs::write(output_path, compile_bootstrap_compiler_artifact()?)?;
    Ok(())
}

pub fn compile_linked_artifact_with_bootstrap_compiler(
    compiler_artifact: &str,
    source: &str,
) -> Result<String, CompileError> {
    compile_linked_artifact_with_bootstrap_compiler_and_libraries(compiler_artifact, source, &[])
}

pub fn compile_linked_artifact_with_bootstrap_compiler_and_libraries(
    compiler_artifact: &str,
    source: &str,
    libraries: &[NamedLibrary],
) -> Result<String, CompileError> {
    let output = run_linked_artifact(
        compiler_artifact,
        bootstrap_compiler_input(source, libraries),
    )?;
    parse_bootstrap_compiler_output(&output.stdout)
}

pub fn compile_linked_artifact_with_native_compiler(
    compiler_executable: &Path,
    source: &str,
) -> Result<String, CompileError> {
    compile_linked_artifact_with_native_compiler_and_libraries(compiler_executable, source, &[])
}

pub fn compile_linked_artifact_with_native_compiler_and_libraries(
    compiler_executable: &Path,
    source: &str,
    libraries: &[NamedLibrary],
) -> Result<String, CompileError> {
    let output = run_native_compiler_process(
        compiler_executable,
        &bootstrap_compiler_input(source, libraries),
    )?;
    parse_bootstrap_compiler_output(&output)
}

pub fn compile_assembly_with_native_toolchain(
    compiler_executable: &Path,
    source: &str,
    target: &str,
) -> Result<String, CompileError> {
    compile_assembly_with_native_toolchain_and_libraries(compiler_executable, source, target, &[])
}

pub fn compile_assembly_with_native_toolchain_and_libraries(
    compiler_executable: &Path,
    source: &str,
    target: &str,
    libraries: &[NamedLibrary],
) -> Result<String, CompileError> {
    let input = native_compiler_input(source, target, libraries);
    let output = run_native_compiler_process(compiler_executable, &input)?;
    parse_native_compiler_output(&output)
}

fn run_native_compiler_process(
    compiler_executable: &Path,
    values: &[String],
) -> Result<String, CompileError> {
    let input = encode_native_input(values);
    let mut child = Command::new(compiler_executable)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            CompileError::Bootstrap(format!(
                "failed to start native APL compiler `{}`: {error}",
                compiler_executable.display()
            ))
        })?;
    child
        .stdin
        .take()
        .ok_or_else(|| CompileError::Bootstrap("native compiler stdin unavailable".to_owned()))?
        .write_all(&input)?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        let diagnostic = String::from_utf8_lossy(&output.stderr);
        return Err(CompileError::Bootstrap(format!(
            "native APL compiler exited with {}{}{}",
            output.status,
            if diagnostic.is_empty() { "" } else { ": " },
            diagnostic.trim_end()
        )));
    }
    let stdout = String::from_utf8(output.stdout).map_err(|_| {
        CompileError::Bootstrap("native APL compiler emitted non-UTF-8 output".to_owned())
    })?;
    Ok(stdout)
}

pub fn emit_linked_artifact_with_native_compiler_file(
    compiler_executable: &Path,
    source_path: &Path,
    output_path: &Path,
) -> Result<(), CompileError> {
    let source = fs::read_to_string(source_path)?;
    let libraries = resolve_named_libraries(source_path, &source)?;
    let artifact = compile_linked_artifact_with_native_compiler_and_libraries(
        compiler_executable,
        &source,
        &libraries,
    )?;
    fs::write(output_path, artifact)?;
    Ok(())
}

pub fn emit_assembly_with_native_toolchain_file(
    compiler_executable: &Path,
    source_path: &Path,
    target: &str,
    output_path: &Path,
) -> Result<(), CompileError> {
    let source = fs::read_to_string(source_path)?;
    let libraries = resolve_named_libraries(source_path, &source)?;
    let assembly = compile_assembly_with_native_toolchain_and_libraries(
        compiler_executable,
        &source,
        target,
        &libraries,
    )?;
    fs::write(output_path, assembly)?;
    Ok(())
}

pub fn emit_linked_artifact_with_bootstrap_compiler_file(
    compiler_path: &Path,
    source_path: &Path,
    output_path: &Path,
) -> Result<(), CompileError> {
    let compiler = fs::read_to_string(compiler_path)?;
    let source = fs::read_to_string(source_path)?;
    let libraries = resolve_named_libraries(source_path, &source)?;
    let artifact = compile_linked_artifact_with_bootstrap_compiler_and_libraries(
        &compiler, &source, &libraries,
    )?;
    fs::write(output_path, artifact)?;
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

pub fn compile_linked_artifact_with_libraries(
    source: &str,
    libraries: &[NamedLibrary],
) -> Result<String, CompileError> {
    let mut input = Vec::with_capacity(libraries.len() * 2 + 3);
    input.push(RUNTIME_PRELUDE.to_owned());
    input.push(libraries.len().to_string());
    for library in libraries {
        input.push(library.name.clone());
        input.push(library.artifact.clone());
    }
    input.push(source.to_owned());

    let output = run_bootstrap_bridge(LINKED_LIBRARIES_COMPILE_BRIDGE, input)?;
    bootstrap_stdout_artifact(output, "linked artifact")
}

pub fn requested_external_libraries(source: &str) -> Result<Vec<String>, CompileError> {
    let output = run_bootstrap_bridge(LIBRARY_DISCOVERY_BRIDGE, vec![source.to_owned()])?;
    parse_library_discovery_output(&output.stdout)
}

pub fn resolve_named_libraries(
    source_path: &Path,
    source: &str,
) -> Result<Vec<NamedLibrary>, CompileError> {
    let source_directory = source_path.parent().unwrap_or_else(|| Path::new("."));
    let library_directory = source_directory.join(LIBRARY_DIRECTORY);
    let mut libraries = Vec::new();

    for name in requested_external_libraries(source)? {
        let path = library_directory.join(format!("{name}.{LIBRARY_EXTENSION}"));
        let artifact = match fs::read_to_string(&path) {
            Ok(artifact) => artifact,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(CompileError::LibraryNotFound { name, path });
            }
            Err(error) => return Err(CompileError::Io(error)),
        };
        libraries.push(NamedLibrary { name, artifact });
    }

    Ok(libraries)
}

pub fn compile_linked_artifact_for_source(
    source_path: &Path,
    source: &str,
) -> Result<String, CompileError> {
    let libraries = resolve_named_libraries(source_path, source)?;
    if libraries.is_empty() {
        compile_linked_artifact(source)
    } else {
        compile_linked_artifact_with_libraries(source, &libraries)
    }
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
    let artifact = compile_linked_artifact_for_source(source_path, &source)?;
    fs::write(output_path, artifact)?;
    Ok(())
}

pub fn run_linked_artifact(
    artifact: &str,
    input: Vec<String>,
) -> Result<apl_runtime::RunOutput, CompileError> {
    let report = run_linked_artifact_report(artifact, input)?;
    if report.status == LinkedRunStatus::Fail {
        let reason = report
            .output
            .last()
            .cloned()
            .unwrap_or_else(|| "linked artifact failed".to_owned());
        return Err(CompileError::Runtime(apl_runtime::RuntimeError::Failed(
            reason,
        )));
    }

    Ok(apl_runtime::RunOutput {
        stdout: report.output.concat(),
    })
}

pub fn run_linked_artifact_report(
    artifact: &str,
    input: Vec<String>,
) -> Result<LinkedRunReport, CompileError> {
    let mut bridge_input = Vec::with_capacity(input.len() + 2);
    bridge_input.push(artifact.to_owned());
    bridge_input.push(input.len().to_string());
    bridge_input.extend(input);
    let output = run_bootstrap_bridge(LINKED_RUN_BRIDGE, bridge_input)?;
    parse_linked_run_report(&output.stdout)
}

pub fn inspect_linked_artifact(artifact: &str) -> Result<String, CompileError> {
    let output = run_bootstrap_bridge(LINKED_MANIFEST_BRIDGE, vec![artifact.to_owned()])?;
    Ok(output.stdout)
}

pub fn lower_linked_artifact_to_native_ir(artifact: &str) -> Result<String, CompileError> {
    let output = run_bootstrap_bridge(NATIVE_LOWER_BRIDGE, vec![artifact.to_owned()])?;
    bootstrap_stdout_artifact(output, "low IR artifact")
}

pub fn compile_native_ir_for_source(
    source_path: &Path,
    source: &str,
) -> Result<String, CompileError> {
    let linked = compile_linked_artifact_for_source(source_path, source)?;
    lower_linked_artifact_to_native_ir(&linked)
}

pub fn emit_native_ir_file(source_path: &Path, output_path: &Path) -> Result<(), CompileError> {
    let source = fs::read_to_string(source_path)?;
    let artifact = compile_native_ir_for_source(source_path, &source)?;
    fs::write(output_path, artifact)?;
    Ok(())
}

pub fn inspect_native_ir(artifact: &str) -> Result<String, CompileError> {
    let output = run_bootstrap_bridge(NATIVE_INSPECT_BRIDGE, vec![artifact.to_owned()])?;
    Ok(output.stdout)
}

pub fn lower_native_ir_to_abi(artifact: &str, target: &str) -> Result<String, CompileError> {
    let output = run_bootstrap_bridge(
        ABI_LOWER_BRIDGE,
        vec![artifact.to_owned(), target.to_owned()],
    )?;
    bootstrap_stdout_artifact(output, "native ABI artifact")
}

pub fn compile_native_abi_for_source(
    source_path: &Path,
    source: &str,
    target: &str,
) -> Result<String, CompileError> {
    let low_ir = compile_native_ir_for_source(source_path, source)?;
    lower_native_ir_to_abi(&low_ir, target)
}

pub fn emit_native_abi_file(
    source_path: &Path,
    target: &str,
    output_path: &Path,
) -> Result<(), CompileError> {
    let source = fs::read_to_string(source_path)?;
    let artifact = compile_native_abi_for_source(source_path, &source, target)?;
    fs::write(output_path, artifact)?;
    Ok(())
}

pub fn inspect_native_abi(artifact: &str) -> Result<String, CompileError> {
    let output = run_bootstrap_bridge(ABI_INSPECT_BRIDGE, vec![artifact.to_owned()])?;
    Ok(output.stdout)
}

pub fn emit_assembly_from_native_abi(artifact: &str) -> Result<String, CompileError> {
    let output = run_bootstrap_bridge(ASSEMBLY_EMIT_BRIDGE, vec![artifact.to_owned()])?;
    bootstrap_stdout_artifact(output, "assembly")
}

pub fn compile_assembly_for_source(
    source_path: &Path,
    source: &str,
    target: &str,
) -> Result<String, CompileError> {
    let linked = compile_linked_artifact_for_source(source_path, source)?;
    compile_assembly_for_linked_artifact(&linked, target)
}

pub fn compile_assembly_for_linked_artifact(
    artifact: &str,
    target: &str,
) -> Result<String, CompileError> {
    let low_ir = lower_linked_artifact_to_native_ir(artifact)?;
    let abi = lower_native_ir_to_abi(&low_ir, target)?;
    emit_assembly_from_native_abi(&abi)
}

pub fn emit_assembly_file(
    source_path: &Path,
    target: &str,
    output_path: &Path,
) -> Result<(), CompileError> {
    let source = fs::read_to_string(source_path)?;
    let assembly = compile_assembly_for_source(source_path, &source, target)?;
    fs::write(output_path, assembly)?;
    Ok(())
}

pub fn host_native_target() -> Result<&'static str, CompileError> {
    if cfg!(all(target_arch = "x86_64", target_os = "windows")) {
        Ok(WINDOWS_X64_TARGET)
    } else if cfg!(all(target_arch = "x86_64", target_os = "linux")) {
        Ok(LINUX_X64_TARGET)
    } else {
        Err(CompileError::Bootstrap(
            "native executable builds currently require x86-64 Windows or Linux".to_owned(),
        ))
    }
}

pub fn build_native_executable_package(
    source_path: &Path,
    output_dir: &Path,
    target: &str,
) -> Result<NativeBuildOutput, CompileError> {
    let source = fs::read_to_string(source_path)?;
    let assembly = compile_assembly_for_source(source_path, &source, target)?;

    build_native_package(source_path, output_dir, target, &assembly)
}

pub fn build_native_linked_executable_package(
    artifact_path: &Path,
    output_dir: &Path,
    target: &str,
) -> Result<NativeBuildOutput, CompileError> {
    let artifact = fs::read_to_string(artifact_path)?;
    let assembly = compile_assembly_for_linked_artifact(&artifact, target)?;
    let output = build_native_package(artifact_path, output_dir, target, &assembly)?;
    fs::write(output.package_dir.join("program.apllink"), artifact)?;
    Ok(output)
}

pub fn build_native_bootstrap_compiler_package(
    output_dir: &Path,
    target: &str,
) -> Result<NativeBuildOutput, CompileError> {
    let artifact = compile_bootstrap_compiler_artifact()?;
    let assembly = compile_assembly_for_linked_artifact(&artifact, target)?;
    let identity = Path::new("bootstrap_compiler.apl");
    let output = build_native_package(identity, output_dir, target, &assembly)?;
    fs::write(output.package_dir.join("program.apllink"), artifact)?;
    Ok(output)
}

pub fn build_native_toolchain_package(
    output_dir: &Path,
    target: &str,
) -> Result<NativeBuildOutput, CompileError> {
    let artifact = compile_native_toolchain_artifact()?;
    let assembly = compile_assembly_for_linked_artifact(&artifact, target)?;
    let identity = Path::new("apl_native_toolchain.apl");
    let output = build_native_package(identity, output_dir, target, &assembly)?;
    fs::write(output.package_dir.join("program.apllink"), artifact)?;
    Ok(output)
}

pub fn build_native_executable_with_native_toolchain(
    compiler_executable: &Path,
    source_path: &Path,
    output_dir: &Path,
    target: &str,
) -> Result<NativeBuildOutput, CompileError> {
    let source = fs::read_to_string(source_path)?;
    let libraries = resolve_named_libraries(source_path, &source)?;
    let assembly = compile_assembly_with_native_toolchain_and_libraries(
        compiler_executable,
        &source,
        target,
        &libraries,
    )?;
    build_native_package(source_path, output_dir, target, &assembly)
}

fn build_native_package(
    identity_path: &Path,
    output_dir: &Path,
    target: &str,
    assembly: &str,
) -> Result<NativeBuildOutput, CompileError> {
    let package_dir = output_dir.to_path_buf();
    let src_dir = package_dir.join("src");
    fs::create_dir_all(&src_dir)?;

    let manifest_path = package_dir.join("Cargo.toml");
    let generated_source_path = src_dir.join("main.rs");
    let assembly_path = package_dir.join("program.s");
    let workspace_root = workspace_root_from_source(identity_path);
    let package_name = format!(
        "{}_native",
        compiled_package_name(identity_path.file_stem().and_then(|name| name.to_str()))
    );

    fs::write(
        &manifest_path,
        render_native_manifest(&workspace_root, &package_name),
    )?;
    fs::write(&assembly_path, assembly)?;
    fs::write(&generated_source_path, render_native_main())?;

    Ok(NativeBuildOutput {
        package_dir,
        package_name,
        assembly_path,
        manifest_path,
        source_path: generated_source_path,
        target: target.to_owned(),
    })
}

pub fn build_executable_package(
    source_path: &Path,
    output_dir: &Path,
    runtime_prelude: &str,
) -> Result<BuildOutput, CompileError> {
    let source = fs::read_to_string(source_path)?;
    let program = checked_program_with_prelude(runtime_prelude, &source)?;
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
    let artifact = compile_linked_artifact_for_source(source_path, &source)?;

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
    let program = checked_program_with_prelude(runtime_prelude, &source)?;
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

pub fn checked_program_with_prelude(
    runtime_prelude: &str,
    source: &str,
) -> Result<apl_core::Program, CompileError> {
    let prelude = apl_parser::parse_program(runtime_prelude)?;
    let program = apl_parser::parse_program(source)?;
    apl_core::validate_program_with_prelude(&prelude, &program)?;

    let mut statements = prelude.statements;
    statements.extend(program.statements);
    Ok(apl_core::Program::new(statements))
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
    if output.stdout.is_empty() {
        Err(CompileError::Bootstrap(format!(
            "APL compiler returned no {artifact_name}"
        )))
    } else {
        Ok(output.stdout)
    }
}

fn render_bootstrap_compiler_driver(runtime_module: &str) -> String {
    let runtime_module = apl_string_literal(runtime_module);
    format!(
        r#"func aplcompiler.emit_report(status, payload) {{
  out "{BOOTSTRAP_COMPILER_PROTOCOL}"
  out status
  out ":"
  out len(payload)
  out ":"
  out payload
  stop
}}

AVStr aplcompiler.runtime_module = {runtime_module}
AVStr aplcompiler.first_input = input
VTime aplcompiler.report = NONE

if aplcompiler.first_input == "{BOOTSTRAP_COMPILER_LIBRARY_REQUEST}" {{
  VTime aplcompiler.library_count = int(input)
  VTime aplcompiler.libraries = []
  VTime aplcompiler.library_index = 0

  while (aplcompiler.library_index < aplcompiler.library_count) (-1) {{
    VTime aplcompiler.library_name = input
    VTime aplcompiler.library_artifact = input
    add(aplcompiler.libraries, [aplcompiler.library_name, aplcompiler.library_artifact])
    aplcompiler.library_index += 1
  }}

  VTime aplcompiler.source = input
  aplcompiler.report = bootstrap.linked_artifact_with_module_and_libraries_report(aplcompiler.runtime_module, aplcompiler.libraries, aplcompiler.source)
}} else {{
  aplcompiler.report = bootstrap.linked_artifact_with_module_report(aplcompiler.runtime_module, aplcompiler.first_input)
}}

if get(aplcompiler.report, 0) != vm.FLOW_OK {{
  VTime aplcompiler.error_emitted = aplcompiler.emit_report(vm.FLOW_FAIL, get(aplcompiler.report, 1))
}}

VTime aplcompiler.artifact_emitted = aplcompiler.emit_report(vm.FLOW_OK, get(aplcompiler.report, 1))"#
    )
}

fn render_native_toolchain_driver(runtime_module: &str) -> String {
    let runtime_module = apl_string_literal(runtime_module);
    format!(
        r#"func aplnative.emit_report(status, payload) {{
  out "{NATIVE_COMPILER_PROTOCOL}"
  out status
  out ":"
  out len(payload)
  out ":"
  out payload
  stop
}}

AVStr aplnative.runtime_module = {runtime_module}
AVStr aplnative.request = input

if aplnative.request != "{NATIVE_COMPILER_REQUEST}" {{
  VTime aplnative.bad_request = aplnative.emit_report(vm.FLOW_FAIL, "invalid native compiler request")
}}

AVStr aplnative.target = input
VTime aplnative.library_count = int(input)
VTime aplnative.libraries = []
VTime aplnative.library_index = 0

while (aplnative.library_index < aplnative.library_count) (-1) {{
  VTime aplnative.library_name = input
  VTime aplnative.library_artifact = input
  add(aplnative.libraries, [aplnative.library_name, aplnative.library_artifact])
  aplnative.library_index += 1
}}

VTime aplnative.source = input
VTime aplnative.linked = bootstrap.linked_artifact_with_module_and_libraries_report(aplnative.runtime_module, aplnative.libraries, aplnative.source)

if get(aplnative.linked, 0) != vm.FLOW_OK {{
  VTime aplnative.link_error = aplnative.emit_report(vm.FLOW_FAIL, get(aplnative.linked, 1))
}}

VTime aplnative.low = native.lower_linked_artifact_report(get(aplnative.linked, 1))

if get(aplnative.low, 0) != vm.FLOW_OK {{
  VTime aplnative.low_error = aplnative.emit_report(vm.FLOW_FAIL, get(aplnative.low, 1))
}}

VTime aplnative.abi = abi.lower_encoded_report(get(aplnative.low, 1), aplnative.target)

if get(aplnative.abi, 0) != vm.FLOW_OK {{
  VTime aplnative.abi_error = aplnative.emit_report(vm.FLOW_FAIL, get(aplnative.abi, 1))
}}

VTime aplnative.assembly = assembly.emit_encoded_report(get(aplnative.abi, 1))

if get(aplnative.assembly, 0) != vm.FLOW_OK {{
  VTime aplnative.assembly_error = aplnative.emit_report(vm.FLOW_FAIL, get(aplnative.assembly, 1))
}}

VTime aplnative.emitted = aplnative.emit_report(vm.FLOW_OK, get(aplnative.assembly, 1))"#
    )
}

fn bootstrap_compiler_input(source: &str, libraries: &[NamedLibrary]) -> Vec<String> {
    if libraries.is_empty() {
        return vec![source.to_owned()];
    }

    let mut input = Vec::with_capacity(libraries.len() * 2 + 3);
    input.push(BOOTSTRAP_COMPILER_LIBRARY_REQUEST.to_owned());
    input.push(libraries.len().to_string());
    for library in libraries {
        input.push(library.name.clone());
        input.push(library.artifact.clone());
    }
    input.push(source.to_owned());
    input
}

fn native_compiler_input(source: &str, target: &str, libraries: &[NamedLibrary]) -> Vec<String> {
    let mut input = Vec::with_capacity(libraries.len() * 2 + 4);
    input.push(NATIVE_COMPILER_REQUEST.to_owned());
    input.push(target.to_owned());
    input.push(libraries.len().to_string());
    for library in libraries {
        input.push(library.name.clone());
        input.push(library.artifact.clone());
    }
    input.push(source.to_owned());
    input
}

fn encode_native_input(values: &[String]) -> Vec<u8> {
    let payload_len = values.iter().map(String::len).sum::<usize>();
    let mut encoded = Vec::with_capacity(NATIVE_INPUT_PROTOCOL.len() + payload_len + 32);
    encoded.extend_from_slice(NATIVE_INPUT_PROTOCOL);
    encoded.extend_from_slice(values.len().to_string().as_bytes());
    encoded.push(b':');
    for value in values {
        encoded.extend_from_slice(value.len().to_string().as_bytes());
        encoded.push(b':');
        encoded.extend_from_slice(value.as_bytes());
    }
    encoded
}

fn apl_string_literal(value: &str) -> String {
    let mut literal = String::with_capacity(value.len() + 2);
    literal.push('"');
    for ch in value.chars() {
        match ch {
            '\n' => literal.push_str("\\n"),
            '\r' => literal.push_str("\\r"),
            '\t' => literal.push_str("\\t"),
            '\0' => literal.push_str("\\0"),
            '\\' => literal.push_str("\\\\"),
            '"' => literal.push_str("\\\""),
            _ => literal.push(ch),
        }
    }
    literal.push('"');
    literal
}

fn parse_bootstrap_compiler_output(output: &str) -> Result<String, CompileError> {
    if let Some(mut response) = output.strip_prefix(BOOTSTRAP_COMPILER_PROTOCOL) {
        let Some(status) = take_protocol_field(&mut response) else {
            return Err(invalid_bootstrap_compiler_response());
        };
        let Some(char_count) = take_protocol_usize(&mut response) else {
            return Err(invalid_bootstrap_compiler_response());
        };
        let Some(payload) = take_protocol_chars(&mut response, char_count) else {
            return Err(invalid_bootstrap_compiler_response());
        };

        if !response.is_empty() {
            return Err(invalid_bootstrap_compiler_response());
        }

        return match status {
            "OK" if is_linked_artifact(payload) => Ok(payload.to_owned()),
            "OK" => Err(CompileError::Bootstrap(
                "APL bootstrap compiler returned an invalid linked artifact".to_owned(),
            )),
            "FAIL" => Err(CompileError::Bootstrap(if payload.is_empty() {
                "APL bootstrap compiler failed without a diagnostic".to_owned()
            } else {
                payload.to_owned()
            })),
            _ => Err(invalid_bootstrap_compiler_response()),
        };
    }

    if let Some(artifact) = output.strip_prefix(LEGACY_BOOTSTRAP_COMPILER_OK) {
        if is_linked_artifact(artifact) {
            return Ok(artifact.to_owned());
        }
        return Err(CompileError::Bootstrap(
            "APL bootstrap compiler returned an invalid linked artifact".to_owned(),
        ));
    }

    if let Some(message) = output.strip_prefix(LEGACY_BOOTSTRAP_COMPILER_ERROR) {
        return Err(CompileError::Bootstrap(if message.is_empty() {
            "APL bootstrap compiler failed without a diagnostic".to_owned()
        } else {
            message.to_owned()
        }));
    }

    Err(invalid_bootstrap_compiler_response())
}

fn parse_native_compiler_output(output: &str) -> Result<String, CompileError> {
    let Some(mut response) = output.strip_prefix(NATIVE_COMPILER_PROTOCOL) else {
        return Err(invalid_native_compiler_response());
    };
    let Some(status) = take_protocol_field(&mut response) else {
        return Err(invalid_native_compiler_response());
    };
    let Some(char_count) = take_protocol_usize(&mut response) else {
        return Err(invalid_native_compiler_response());
    };
    let Some(payload) = take_protocol_chars(&mut response, char_count) else {
        return Err(invalid_native_compiler_response());
    };
    if !response.is_empty() {
        return Err(invalid_native_compiler_response());
    }

    match status {
        "OK" if payload.starts_with("# APLASM1 runtime-abi=1 target=") => Ok(payload.to_owned()),
        "OK" => Err(CompileError::Bootstrap(
            "native APL compiler returned invalid assembly".to_owned(),
        )),
        "FAIL" => Err(CompileError::Bootstrap(if payload.is_empty() {
            "native APL compiler failed without a diagnostic".to_owned()
        } else {
            payload.to_owned()
        })),
        _ => Err(invalid_native_compiler_response()),
    }
}

fn invalid_bootstrap_compiler_response() -> CompileError {
    CompileError::Bootstrap("APL bootstrap compiler returned an invalid response".to_owned())
}

fn invalid_native_compiler_response() -> CompileError {
    CompileError::Bootstrap("native APL compiler returned an invalid response".to_owned())
}

fn is_linked_artifact(value: &str) -> bool {
    value.starts_with(LINKED_ARTIFACT_PROTOCOL)
        || value.starts_with(LEGACY_LINKED_ARTIFACT_PROTOCOL)
}

fn parse_linked_run_report(output: &str) -> Result<LinkedRunReport, CompileError> {
    let Some(mut payload) = output.strip_prefix(LINKED_RUN_PROTOCOL) else {
        return Err(invalid_linked_run_response());
    };
    let status = match take_protocol_field(&mut payload) {
        Some("OK") => LinkedRunStatus::Ok,
        Some("STOP") => LinkedRunStatus::Stop,
        Some("FAIL") => LinkedRunStatus::Fail,
        _ => return Err(invalid_linked_run_response()),
    };
    let Some(output_count) = take_protocol_usize(&mut payload) else {
        return Err(invalid_linked_run_response());
    };
    let mut values = Vec::with_capacity(output_count);

    for _ in 0..output_count {
        let Some(char_count) = take_protocol_usize(&mut payload) else {
            return Err(invalid_linked_run_response());
        };
        let Some(value) = take_protocol_chars(&mut payload, char_count) else {
            return Err(invalid_linked_run_response());
        };
        values.push(value.to_owned());
    }

    if !payload.is_empty() {
        return Err(invalid_linked_run_response());
    }

    Ok(LinkedRunReport {
        status,
        output: values,
    })
}

fn take_protocol_field<'a>(payload: &mut &'a str) -> Option<&'a str> {
    let end = payload.find(':')?;
    let field = &payload[..end];
    *payload = &payload[end + 1..];
    Some(field)
}

fn take_protocol_usize(payload: &mut &str) -> Option<usize> {
    take_protocol_field(payload)?.parse().ok()
}

fn take_protocol_chars<'a>(payload: &mut &'a str, count: usize) -> Option<&'a str> {
    let mut byte_end = 0;
    let mut chars = payload.char_indices();
    for _ in 0..count {
        let (offset, ch) = chars.next()?;
        byte_end = offset + ch.len_utf8();
    }
    let value = &payload[..byte_end];
    *payload = &payload[byte_end..];
    Some(value)
}

fn invalid_linked_run_response() -> CompileError {
    CompileError::Bootstrap("invalid APL linked run response".to_owned())
}

fn parse_library_discovery_output(output: &str) -> Result<Vec<String>, CompileError> {
    let Some(mut payload) = output.strip_prefix(LIBRARY_DISCOVERY_PROTOCOL) else {
        return Err(invalid_library_discovery_response());
    };
    let Some(status) = take_protocol_field(&mut payload) else {
        return Err(invalid_library_discovery_response());
    };
    let Some(value_count) = take_protocol_usize(&mut payload) else {
        return Err(invalid_library_discovery_response());
    };
    let mut values = Vec::with_capacity(value_count);

    for _ in 0..value_count {
        let Some(char_count) = take_protocol_usize(&mut payload) else {
            return Err(invalid_library_discovery_response());
        };
        let Some(value) = take_protocol_chars(&mut payload, char_count) else {
            return Err(invalid_library_discovery_response());
        };
        values.push(value.to_owned());
    }

    if !payload.is_empty() {
        return Err(invalid_library_discovery_response());
    }

    match status {
        "OK" => Ok(values),
        "FAIL" if values.len() == 1 => Err(CompileError::Bootstrap(values.remove(0))),
        _ => Err(invalid_library_discovery_response()),
    }
}

fn invalid_library_discovery_response() -> CompileError {
    CompileError::Bootstrap("invalid APL library discovery response".to_owned())
}

const LINKED_COMPILE_BRIDGE: &str = r#"VTime aplhost.runtime_source = input
VTime aplhost.program_source = input
VTime aplhost.compile_report = bootstrap.linked_artifact_with_libraries_report(aplhost.runtime_source, [], aplhost.program_source)
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

const LINKED_LIBRARIES_COMPILE_BRIDGE: &str = r#"VTime aplhost.base_source = input
VTime aplhost.library_count = int(input)
VTime aplhost.libraries = []
VTime aplhost.library_index = 0
while (aplhost.library_index < aplhost.library_count) (-1) {
  VTime aplhost.library_name = input
  VTime aplhost.library_artifact = input
  add(aplhost.libraries, [aplhost.library_name, aplhost.library_artifact])
  aplhost.library_index += 1
}
VTime aplhost.source = input
VTime aplhost.compile_report = bootstrap.linked_artifact_with_libraries_report(aplhost.base_source, aplhost.libraries, aplhost.source)
if get(aplhost.compile_report, 0) != vm.FLOW_OK {
  fail get(aplhost.compile_report, 1)
}
out get(aplhost.compile_report, 1)"#;

const LINKED_RUN_BRIDGE: &str = r#"func aplhost.emit_run_report(status, output) {
  out "APLRUN1:"
  out status
  out ":"
  out len(output)
  out ":"

  pick(output): aplhost.item {
    out len(aplhost.item)
    out ":"
    out aplhost.item
  }

  stop
}

VTime aplhost.encoded = input
VTime aplhost.loaded_report = bootstrap.load_linked_artifact_report(aplhost.encoded)

if get(aplhost.loaded_report, 0) != vm.FLOW_OK {
  VTime aplhost.invalid_emitted = aplhost.emit_run_report(vm.FLOW_FAIL, [get(aplhost.loaded_report, 1)])
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
VTime aplhost.report_emitted = aplhost.emit_run_report(get(aplhost.run_report, 0), aplhost.output)"#;

const LINKED_MANIFEST_BRIDGE: &str = r#"VTime aplhost.encoded = input
VTime aplhost.manifest_report = bootstrap.linked_artifact_manifest_report(aplhost.encoded)

if get(aplhost.manifest_report, 0) != vm.FLOW_OK {
  fail get(aplhost.manifest_report, 1)
}

VTime aplhost.manifest = get(aplhost.manifest_report, 1)

if aplhost.manifest == NONE {
  out "manifest unavailable (legacy APLLINK2)\n"
} else {
  out "manifest "
  out get(aplhost.manifest, 0)
  out "\n"

  pick(get(aplhost.manifest, 1)): aplhost.library {
    out get(aplhost.library, 0)
    out "\n"

    pick(get(aplhost.library, 1)): aplhost.exclusion {
      out "  exclude "
      out aplhost.exclusion
      out "\n"
    }
  }
}"#;

const NATIVE_LOWER_BRIDGE: &str = r#"VTime aplhost.encoded = input
VTime aplhost.lowered = native.lower_linked_artifact_report(aplhost.encoded)

if get(aplhost.lowered, 0) != vm.FLOW_OK {
  fail get(aplhost.lowered, 1)
}

out get(aplhost.lowered, 1)"#;

const NATIVE_INSPECT_BRIDGE: &str = r#"VTime aplhost.encoded = input
VTime aplhost.description = native.describe_image_report(aplhost.encoded)

if get(aplhost.description, 0) != vm.FLOW_OK {
  fail get(aplhost.description, 1)
}

out get(aplhost.description, 1)"#;

const ABI_LOWER_BRIDGE: &str = r#"VTime aplhost.encoded = input
VTime aplhost.target = input
VTime aplhost.lowered = abi.lower_encoded_report(aplhost.encoded, aplhost.target)

if get(aplhost.lowered, 0) != vm.FLOW_OK {
  fail get(aplhost.lowered, 1)
}

out get(aplhost.lowered, 1)"#;

const ABI_INSPECT_BRIDGE: &str = r#"VTime aplhost.encoded = input
VTime aplhost.description = abi.describe_image_report(aplhost.encoded)

if get(aplhost.description, 0) != vm.FLOW_OK {
  fail get(aplhost.description, 1)
}

out get(aplhost.description, 1)"#;

const ASSEMBLY_EMIT_BRIDGE: &str = r#"VTime aplhost.encoded = input
VTime aplhost.emitted = assembly.emit_encoded_report(aplhost.encoded)

if get(aplhost.emitted, 0) != vm.FLOW_OK {
  fail get(aplhost.emitted, 1)
}

out get(aplhost.emitted, 1)"#;

const LIBRARY_DISCOVERY_BRIDGE: &str = r#"func aplhost.emit_library_report(status, values) {
  out "APLLIB1:"
  out status
  out ":"
  out len(values)
  out ":"

  pick(values): aplhost.value {
    out len(aplhost.value)
    out ":"
    out aplhost.value
  }

  stop
}

VTime aplhost.report = bootstrap.requested_libraries_report(input)

if get(aplhost.report, 0) != vm.FLOW_OK {
  VTime aplhost.failure_emitted = aplhost.emit_library_report(vm.FLOW_FAIL, [get(aplhost.report, 1)])
}

VTime aplhost.success_emitted = aplhost.emit_library_report(vm.FLOW_OK, get(aplhost.report, 1))"#;

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

fn render_native_manifest(workspace_root: &Path, package_name: &str) -> String {
    let runtime_path = toml_path(&workspace_root.join("crates/apl_native_runtime"));

    format!(
        r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2021"

[workspace]

[dependencies]
apl_native_runtime = {{ path = "{runtime_path}" }}
"#
    )
}

fn render_native_main() -> String {
    r#"#![allow(bad_asm_style)]

use std::{arch::global_asm, ffi::c_void, process};

use apl_native_runtime::NativeContext;

global_asm!(include_str!("../program.s"));

extern "C" {
    fn apl_program_entry(context: *mut c_void) -> i32;
}

fn main() {
    let mut context = NativeContext::from_stdin().unwrap_or_else(|error| {
        eprintln!("APL input error: {error}");
        process::exit(1);
    });
    let status = unsafe { apl_program_entry((&mut context as *mut NativeContext).cast()) };
    process::exit(status);
}
"#
    .to_owned()
}

fn render_main(ir: &[u8]) -> String {
    format!(
        r#"use std::{{io::{{IsTerminal, Read}}, process}};

fn read_input() -> Vec<String> {{
    let mut stdin = std::io::stdin();
    if stdin.is_terminal() {{
        return Vec::new();
    }}

    let mut input = String::new();
    let _ = stdin.read_to_string(&mut input);
    input.lines().map(str::to_owned).collect()
}}

fn main() {{
    let input = read_input();

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
    r#"use std::{io::{IsTerminal, Read}, process};

fn read_input() -> Vec<String> {
    let mut stdin = std::io::stdin();
    if stdin.is_terminal() {
        return Vec::new();
    }

    let mut input = String::new();
    let _ = stdin.read_to_string(&mut input);
    input.lines().map(str::to_owned).collect()
}

fn main() {
    let input = read_input();

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
    let find_workspace = |path: &Path| {
        path.ancestors()
            .find(|ancestor| ancestor.join("crates/apl_runtime").exists())
            .map(Path::to_path_buf)
    };
    source_path
        .canonicalize()
        .ok()
        .and_then(|path| find_workspace(&path))
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .and_then(|path| find_workspace(&path))
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
    fn bootstrap_compiler_request_frames_named_libraries() {
        assert_eq!(
            bootstrap_compiler_input("out 1", &[]),
            vec!["out 1".to_owned()]
        );

        let libraries = vec![
            NamedLibrary {
                name: "math".to_owned(),
                artifact: "APLMOD3:math".to_owned(),
            },
            NamedLibrary {
                name: "unsafe.memory".to_owned(),
                artifact: "APLMOD3:memory\n\0".to_owned(),
            },
        ];
        assert_eq!(
            bootstrap_compiler_input("include >> math out math.answer()", &libraries),
            vec![
                BOOTSTRAP_COMPILER_LIBRARY_REQUEST.to_owned(),
                "2".to_owned(),
                "math".to_owned(),
                "APLMOD3:math".to_owned(),
                "unsafe.memory".to_owned(),
                "APLMOD3:memory\n\0".to_owned(),
                "include >> math out math.answer()".to_owned(),
            ]
        );
    }

    #[test]
    fn native_input_protocol_preserves_value_boundaries() {
        let values = vec![
            "line1\nline2".to_owned(),
            "\u{410}\u{41f}".to_owned(),
            String::new(),
        ];
        assert_eq!(
            encode_native_input(&values),
            b"\0APLIN1:3:11:line1\nline24:\xD0\x90\xD0\x9F0:"
        );
    }

    #[test]
    fn source_library_resolution_is_local_and_preserves_dotted_names() {
        let source = "include >> unsafe.memory include >> base include >> unsafe.memory out 1";
        assert_eq!(
            requested_external_libraries(source).unwrap(),
            vec!["unsafe.memory"]
        );

        let source_path = Path::new("missing_project").join("program.apl");
        let error = resolve_named_libraries(&source_path, source).unwrap_err();
        assert!(matches!(
            error,
            CompileError::LibraryNotFound { name, path }
                if name == "unsafe.memory"
                    && path == Path::new("missing_project")
                        .join("libraries")
                        .join("unsafe.memory.aplmod")
        ));

        let CompileError::Bootstrap(message) =
            requested_external_libraries("AVInt value 1").unwrap_err()
        else {
            panic!("expected APL parser diagnostic from library discovery");
        };
        assert_eq!(message, "parse error at 1:13: declaration expects =");
    }

    #[test]
    fn library_discovery_protocol_is_length_framed_and_validated() {
        assert_eq!(
            parse_library_discovery_output("APLLIB1:OK:2:4:math13:unsafe.memory").unwrap(),
            vec!["math", "unsafe.memory"]
        );

        let CompileError::Bootstrap(message) =
            parse_library_discovery_output("APLLIB1:FAIL:1:11:bad: source").unwrap_err()
        else {
            panic!("expected discovery diagnostic");
        };
        assert_eq!(message, "bad: source");

        for malformed in [
            "unknown",
            "APLLIB1:MAYBE:0:",
            "APLLIB1:OK:x:",
            "APLLIB1:OK:1:4:abc",
            "APLLIB1:OK:0:trailing",
            "APLLIB1:FAIL:0:",
        ] {
            assert!(matches!(
                parse_library_discovery_output(malformed),
                Err(CompileError::Bootstrap(_))
            ));
        }
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
        assert!(STANDARD_PRELUDE.contains("func lexer.tokenize_with_offsets"));
        assert!(STANDARD_PRELUDE.contains("func lexer.position_at"));
        assert!(STANDARD_PRELUDE.contains("func parser.parse_source"));
        assert!(STANDARD_PRELUDE.contains("func parser.parse_source_with_positions"));
        assert!(STANDARD_PRELUDE.contains("func parser.node_token_index"));
        assert!(STANDARD_PRELUDE.contains("func parser.diagnostic"));
        assert!(STANDARD_PRELUDE.contains("func parser.source_diagnostic"));
        assert!(STANDARD_PRELUDE.contains("func checker.validate_report"));
        assert!(STANDARD_PRELUDE.contains("func ir.compile_source"));
        assert!(STANDARD_PRELUDE.contains("func linker.link_image"));
        assert!(STANDARD_PRELUDE.contains("func verifier.loaded_image_is_valid"));
        assert!(STANDARD_PRELUDE.contains("func vm.run_source"));
        assert!(STANDARD_PRELUDE.contains("func artifact.encode_loaded_report"));
        assert!(STANDARD_PRELUDE.contains("func artifact.wrap_value"));
        assert!(STANDARD_PRELUDE.contains("func native.lower_loaded_image_report"));
        assert!(STANDARD_PRELUDE.contains("func native.lower_linked_artifact_report"));
        assert!(STANDARD_PRELUDE.contains("func native.decode_image_report"));
        assert!(STANDARD_PRELUDE.contains("func abi.lower_image_report"));
        assert!(STANDARD_PRELUDE.contains("func abi.lower_encoded_report"));
        assert!(STANDARD_PRELUDE.contains("func abi.decode_image_report"));
        assert!(STANDARD_PRELUDE.contains("func assembly.emit_image_report"));
        assert!(STANDARD_PRELUDE.contains("func assembly.emit_encoded_report"));
        assert!(STANDARD_PRELUDE.contains("func bootstrap.compile_report"));
        assert!(STANDARD_PRELUDE.contains("func bootstrap.check_error"));
        assert!(STANDARD_PRELUDE.contains("func bootstrap.located_check_error"));
        assert!(STANDARD_PRELUDE.contains("func bootstrap.requested_libraries_report"));
        assert!(STANDARD_PRELUDE.contains("func bootstrap.artifact_report"));
        assert!(STANDARD_PRELUDE.contains("func bootstrap.artifact_image_report"));
        assert!(STANDARD_PRELUDE.contains("func bootstrap.run_artifact_image_with_input_report"));
        assert!(STANDARD_PRELUDE.contains("func bootstrap.loaded_image_report"));
        assert!(STANDARD_PRELUDE.contains("func bootstrap.linked_artifact_report"));
        assert!(STANDARD_PRELUDE.contains("func bootstrap.load_linked_artifact_report"));
        assert!(STANDARD_PRELUDE.contains("func bootstrap.linked_artifact_manifest_report"));
        assert!(STANDARD_PRELUDE.contains("func bootstrap.module_report"));
        assert!(STANDARD_PRELUDE.contains("func bootstrap.load_module_report"));
        assert!(STANDARD_PRELUDE.contains("func bootstrap.linked_artifact_with_module_report"));
        assert!(STANDARD_PRELUDE
            .contains("func bootstrap.linked_artifact_with_module_and_libraries_report"));
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
        assert!(rendered.contains("stdin.is_terminal()"));
        assert!(!rendered.contains("parse_program"));
        assert!(!rendered.contains("PROGRAM_SOURCE"));
    }

    #[test]
    fn apl_bootstrap_compiles_and_runs_portable_artifact() {
        let artifact = compile_linked_artifact(
            "func inc(x) { return x + 1 } AVInt value = input out inc(value)",
        )
        .unwrap();

        assert!(artifact.starts_with(LINKED_ARTIFACT_PROTOCOL));
        let output = run_linked_artifact(&artifact, vec!["4".to_owned()]).unwrap();
        assert_eq!(output.stdout, "5");
    }

    #[test]
    fn linked_artifact_honors_base_composition_directives() {
        let artifact = compile_linked_artifact(
            r#"include >> base exclude "pick" << base exclude "len" << base out 42"#,
        )
        .unwrap();

        assert!(artifact.starts_with(LINKED_ARTIFACT_PROTOCOL));
        assert!(!artifact.contains("Include"));
        assert!(!artifact.contains("Exclude"));
        assert_eq!(run_linked_artifact(&artifact, vec![]).unwrap().stdout, "42");
        assert_eq!(
            inspect_linked_artifact(&artifact).unwrap(),
            "manifest APLMAN1\nbase\n  exclude pick\n  exclude len\n"
        );

        let CompileError::Runtime(apl_runtime::RuntimeError::Failed(message)) =
            compile_linked_artifact(r#"exclude "len" << base out len([1])"#).unwrap_err()
        else {
            panic!("expected excluded base symbol to fail before artifact emission");
        };
        assert_eq!(
            message,
            "semantic error at 1:23: excluded symbol `len` from library `base`"
        );
    }

    #[test]
    fn linked_artifact_lowers_to_inspectable_native_ir() {
        let linked = compile_linked_artifact(
            "func double(value) { return value * 2 } AVInt answer = double(21) out answer",
        )
        .unwrap();
        let lowered = lower_linked_artifact_to_native_ir(&linked).unwrap();

        assert!(lowered.starts_with(NATIVE_IR_PROTOCOL));

        let description = inspect_native_ir(&lowered).unwrap();
        assert!(description.starts_with("low-ir APLLOW1\nentry\n"));
        assert!(description.contains("[CALL_SLOT, 0, 1]"));
        assert!(description.contains("slot 0 double[value]"));
        assert!(description.contains("[BINARY, *]"));
        assert!(description.contains("manifest [APLMAN1, [[base, []]]]"));

        assert!(matches!(
            inspect_native_ir("BAD"),
            Err(CompileError::Runtime(apl_runtime::RuntimeError::Failed(message)))
                if message == "invalid low IR header"
        ));
    }

    #[test]
    fn native_ir_lowers_to_inspectable_platform_abi() {
        let linked = compile_linked_artifact(
            "func double(value) { return value * 2 } AVInt answer = double(21) out answer",
        )
        .unwrap();
        let low_ir = lower_linked_artifact_to_native_ir(&linked).unwrap();
        let lowered = lower_native_ir_to_abi(&low_ir, WINDOWS_X64_TARGET).unwrap();

        assert!(lowered.starts_with(NATIVE_ABI_PROTOCOL));

        let description = inspect_native_abi(&lowered).unwrap();
        assert!(description
            .starts_with("native-abi APLABI1\ntarget x86_64-pc-windows-msvc win64\nconstants\n"));
        assert!(description.contains("CALL_SLOT [8, 0, 1]"));
        assert!(description.contains("slot 0 double refs="));
        assert!(description.contains("BINARY [6, 10]"));
        assert!(description.contains("manifest [APLMAN1, [[base, []]]]"));

        let CompileError::Runtime(apl_runtime::RuntimeError::Failed(message)) =
            lower_native_ir_to_abi(&low_ir, "not-a-target").unwrap_err()
        else {
            panic!("expected unsupported ABI target failure");
        };
        assert_eq!(message, "unsupported native target: not-a-target");

        assert!(matches!(
            inspect_native_abi("BAD"),
            Err(CompileError::Runtime(apl_runtime::RuntimeError::Failed(message)))
                if message == "invalid native ABI header"
        ));
    }

    #[test]
    fn native_abi_emits_platform_assembly() {
        let linked = compile_linked_artifact(
            "func double(value) { return value * 2 } AVInt answer = double(21) out answer",
        )
        .unwrap();
        let low_ir = lower_linked_artifact_to_native_ir(&linked).unwrap();
        let native_abi = lower_native_ir_to_abi(&low_ir, WINDOWS_X64_TARGET).unwrap();
        let assembly = emit_assembly_from_native_abi(&native_abi).unwrap();

        assert!(assembly.starts_with(
            "# APLASM1 runtime-abi=1 target=x86_64-pc-windows-msvc\n.intel_syntax noprefix\n.text\n"
        ));
        assert!(assembly.contains(".globl apl_program_entry"));
        assert!(assembly.contains("sub rsp, 40"));
        assert!(assembly.contains("call apl_fn_0"));
        assert!(assembly.contains("call apl_rt_binary"));
        assert!(assembly.contains("call apl_rt_finish"));
        assert!(assembly.contains(".section .rdata,\"dr\""));

        assert!(matches!(
            emit_assembly_from_native_abi("BAD"),
            Err(CompileError::Runtime(apl_runtime::RuntimeError::Failed(message)))
                if message == "invalid native ABI header"
        ));
    }

    #[test]
    fn generated_host_x64_assembly_is_accepted_by_rustc() {
        if std::env::consts::ARCH != "x86_64" {
            return;
        }

        let target = if cfg!(windows) {
            WINDOWS_X64_TARGET
        } else {
            LINUX_X64_TARGET
        };
        let source = "func inc(value) { return value + 1 } AVInt x = 0 while (x < 2) (3) { x += 1 } List values = [x] pick(values): item { out inc(item) }";
        let assembly =
            compile_assembly_for_source(Path::new("assembly_smoke.apl"), source, target).unwrap();
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "apl_assembly_smoke_{}_{}",
            std::process::id(),
            unique
        ));
        fs::create_dir_all(&directory).unwrap();
        let assembly_path = directory.join("program.s");
        let source_path = directory.join("main.rs");
        let executable_path = directory.join(format!("smoke{}", std::env::consts::EXE_SUFFIX));
        fs::write(&assembly_path, assembly).unwrap();

        let symbols = [
            "apl_rt_push_const",
            "apl_rt_push_none",
            "apl_rt_input",
            "apl_rt_secret_input",
            "apl_rt_load",
            "apl_rt_unary",
            "apl_rt_binary",
            "apl_rt_builtin_get",
            "apl_rt_builtin_split",
            "apl_rt_builtin_join",
            "apl_rt_builtin_contains",
            "apl_rt_builtin_pow",
            "apl_rt_builtin_add",
            "apl_rt_builtin_len",
            "apl_rt_builtin_ord",
            "apl_rt_builtin_char",
            "apl_rt_builtin_pop",
            "apl_rt_builtin_int",
            "apl_rt_builtin_float",
            "apl_rt_builtin_bool",
            "apl_rt_builtin_str",
            "apl_rt_builtin_bytes",
            "apl_rt_builtin_json",
            "apl_rt_list",
            "apl_rt_index",
            "apl_rt_slice",
            "apl_rt_self",
            "apl_rt_tag",
            "apl_rt_declare",
            "apl_rt_declare_list",
            "apl_rt_declare_vtime",
            "apl_rt_assign",
            "apl_rt_info_assign",
            "apl_rt_drop",
            "apl_rt_secretup",
            "apl_rt_out",
            "apl_rt_stop",
            "apl_rt_fail",
            "apl_rt_return",
            "apl_rt_branch_false",
            "apl_rt_scope_enter",
            "apl_rt_scope_exit",
            "apl_rt_unwind",
            "apl_rt_loop_enter",
            "apl_rt_loop_check",
            "apl_rt_loop_exit",
            "apl_rt_pick_enter",
            "apl_rt_pick_next",
            "apl_rt_pick_exit",
            "apl_rt_function_enter",
            "apl_rt_finish",
        ];
        let mut wrapper = format!(
            "use std::arch::global_asm;\nglobal_asm!(include_str!({:?}));\n",
            assembly_path.to_string_lossy()
        );
        for symbol in symbols {
            wrapper.push_str(&format!(
                "#[no_mangle] pub extern \"C\" fn {symbol}() -> i32 {{ 0 }}\n"
            ));
        }
        wrapper.push_str("fn main() {}\n");
        fs::write(&source_path, wrapper).unwrap();

        let output = std::process::Command::new("rustc")
            .arg("--edition=2021")
            .arg(&source_path)
            .arg("-o")
            .arg(&executable_path)
            .output()
            .unwrap();
        let _ = fs::remove_dir_all(&directory);

        assert!(
            output.status.success(),
            "rustc rejected generated assembly:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn linked_artifact_bridge_preserves_empty_input_lines() {
        let artifact = compile_linked_artifact(
            "AVStr first = input AVStr second = input out len(first) out second",
        )
        .unwrap();

        let output =
            run_linked_artifact(&artifact, vec![String::new(), "kept".to_owned()]).unwrap();
        assert_eq!(output.stdout, "0kept");
    }

    #[test]
    fn linked_artifact_bridge_preserves_stop_and_fail_outcomes() {
        let stopped = compile_linked_artifact(r#"out "before" stop "done" out "after""#).unwrap();
        assert_eq!(
            run_linked_artifact_report(&stopped, vec![]).unwrap(),
            LinkedRunReport {
                status: LinkedRunStatus::Stop,
                output: vec!["before".to_owned(), "done".to_owned()],
            }
        );
        let stopped_output = run_linked_artifact(&stopped, vec![]).unwrap();
        assert_eq!(stopped_output.stdout, "beforedone");

        let failed = compile_linked_artifact(r#"out "before" fail "bad" out "after""#).unwrap();
        assert_eq!(
            run_linked_artifact_report(&failed, vec![]).unwrap(),
            LinkedRunReport {
                status: LinkedRunStatus::Fail,
                output: vec!["before".to_owned(), "bad".to_owned()],
            }
        );
        let CompileError::Runtime(apl_runtime::RuntimeError::Failed(reason)) =
            run_linked_artifact(&failed, vec![]).unwrap_err()
        else {
            panic!("expected linked APL failure");
        };
        assert_eq!(reason, "bad");
    }

    #[test]
    fn linked_run_protocol_is_length_framed_and_validated() {
        assert_eq!(
            parse_linked_run_report("APLRUN1:STOP:3:0:6:Привет4:a\0:b").unwrap(),
            LinkedRunReport {
                status: LinkedRunStatus::Stop,
                output: vec![String::new(), "Привет".to_owned(), "a\0:b".to_owned()],
            }
        );

        for malformed in [
            "unknown",
            "APLRUN1:BAD:0:",
            "APLRUN1:OK:x:",
            "APLRUN1:OK:1:4:abc",
            "APLRUN1:OK:0:trailing",
        ] {
            assert!(matches!(
                parse_linked_run_report(malformed),
                Err(CompileError::Bootstrap(_))
            ));
        }
    }

    #[test]
    fn linked_artifact_report_preserves_arbitrary_output_fragments() {
        let artifact = compile_linked_artifact(r#"out "" out "Привет" out "a\0:b""#).unwrap();
        let report = run_linked_artifact_report(&artifact, vec![]).unwrap();

        assert_eq!(
            report,
            LinkedRunReport {
                status: LinkedRunStatus::Ok,
                output: vec![String::new(), "Привет".to_owned(), "a\0:b".to_owned()],
            }
        );
        assert_eq!(
            run_linked_artifact(&artifact, vec![]).unwrap().stdout,
            "Приветa\0:b"
        );

        assert_eq!(
            run_linked_artifact_report("BAD", vec![]).unwrap(),
            LinkedRunReport {
                status: LinkedRunStatus::Fail,
                output: vec!["invalid linked artifact header".to_owned()],
            }
        );
    }

    #[test]
    fn linked_artifact_includes_apl_runtime_module() {
        let artifact =
            compile_linked_artifact("AVInt value = apl.pow_int(3, 4) out value").unwrap();

        let output = run_linked_artifact(&artifact, vec![]).unwrap();
        assert_eq!(output.stdout, "81");
    }

    #[test]
    fn linked_artifact_accepts_an_explicit_apl_module() {
        let artifact = compile_linked_artifact_with_module(
            "func module.double(value) { return value * 2 }",
            "out module.double(21)",
        )
        .unwrap();

        let output = run_linked_artifact(&artifact, vec![]).unwrap();
        assert_eq!(output.stdout, "42");
    }

    #[test]
    fn precompiled_apl_module_links_without_module_source() {
        let module = compile_module_artifact(
            "func module.add_one(value) { return value + 1 } func module.twice(value) { return module.add_one(module.add_one(value)) }",
        )
        .unwrap();
        assert!(module.starts_with("APLMOD3:"));

        let artifact =
            compile_linked_artifact_with_precompiled_module(&module, "out module.twice(40)")
                .unwrap();
        let output = run_linked_artifact(&artifact, vec![]).unwrap();
        assert_eq!(output.stdout, "42");
    }

    #[test]
    fn named_library_include_and_exclude_control_exports() {
        let math = compile_module_artifact(
            "func math.internal(value) { return value + 1 } func math.answer() { return math.internal(41) }",
        )
        .unwrap();
        let libraries = vec![NamedLibrary {
            name: "math".to_owned(),
            artifact: math,
        }];

        let artifact = compile_linked_artifact_with_libraries(
            r#"include >> math exclude "math.internal" << math out math.answer()"#,
            &libraries,
        )
        .unwrap();
        assert_eq!(run_linked_artifact(&artifact, vec![]).unwrap().stdout, "42");
        assert_eq!(
            inspect_linked_artifact(&artifact).unwrap(),
            "manifest APLMAN1\nbase\nmath\n  exclude math.internal\n"
        );

        let excluded = compile_linked_artifact_with_libraries(
            r#"include >> math exclude "math.internal" << math out math.internal(1)"#,
            &libraries,
        )
        .unwrap_err();
        assert!(matches!(
            excluded,
            CompileError::Runtime(apl_runtime::RuntimeError::Failed(message))
                if message == "semantic error at 1:49: excluded symbol `math.internal` from library `math`"
        ));

        let missing_include =
            compile_linked_artifact_with_libraries("out math.answer()", &libraries).unwrap_err();
        assert!(matches!(
            missing_include,
            CompileError::Runtime(apl_runtime::RuntimeError::Failed(message))
                if message == "semantic error at 1:1: unknown function `math.answer`"
        ));
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
        assert!(extended.starts_with("APLMOD3:"));

        let artifact =
            compile_linked_artifact_with_precompiled_module(&extended, "out module.total()")
                .unwrap();
        let output = run_linked_artifact(&artifact, vec![]).unwrap();
        assert_eq!(output.stdout, "42");
    }

    #[test]
    fn bootstrap_compiler_protocol_is_versioned_and_validated() {
        let artifact = "APLLINK3:payload:Привет\0";
        let encoded = format!(
            "{BOOTSTRAP_COMPILER_PROTOCOL}OK:{}:{artifact}",
            artifact.chars().count()
        );
        assert_eq!(parse_bootstrap_compiler_output(&encoded).unwrap(), artifact);

        let legacy = "APLLINK2:legacy";
        let encoded = format!(
            "{BOOTSTRAP_COMPILER_PROTOCOL}OK:{}:{legacy}",
            legacy.chars().count()
        );
        assert_eq!(parse_bootstrap_compiler_output(&encoded).unwrap(), legacy);

        let diagnostic = "declaration:\nожидалось =\0";
        let encoded = format!(
            "{BOOTSTRAP_COMPILER_PROTOCOL}FAIL:{}:{diagnostic}",
            diagnostic.chars().count()
        );
        let CompileError::Bootstrap(message) =
            parse_bootstrap_compiler_output(&encoded).unwrap_err()
        else {
            panic!("expected bootstrap protocol error");
        };
        assert_eq!(message, diagnostic);

        assert_eq!(
            parse_bootstrap_compiler_output("APLCOMP1:OAPLLINK2:legacy").unwrap(),
            "APLLINK2:legacy"
        );

        assert!(matches!(
            parse_bootstrap_compiler_output("APLCOMP1:Onot-an-artifact"),
            Err(CompileError::Bootstrap(_))
        ));
        assert!(matches!(
            parse_bootstrap_compiler_output("APLCOMP2:OK:20:APLLINK2:short"),
            Err(CompileError::Bootstrap(_))
        ));
        assert!(matches!(
            parse_bootstrap_compiler_output("APLCOMP2:OK:14:APLLINK2:shortx"),
            Err(CompileError::Bootstrap(_))
        ));
        assert!(matches!(
            parse_bootstrap_compiler_output("APLCOMP2:MAYBE:0:"),
            Err(CompileError::Bootstrap(_))
        ));
        assert!(matches!(
            parse_bootstrap_compiler_output("unknown"),
            Err(CompileError::Bootstrap(_))
        ));
    }

    #[test]
    fn bootstrap_compiler_driver_embeds_runtime_module_safely() {
        let encoded = "APLMOD3:line\nquote=\" slash=\\ tab=\t nul=\0";
        let driver = render_bootstrap_compiler_driver(encoded);
        let program = apl_parser::parse_program(&driver).unwrap();

        let Some(declaration) = program.statements.iter().find_map(|statement| {
            let apl_core::Statement::VariableDecl(declaration) = statement else {
                return None;
            };
            (declaration.name == "aplcompiler.runtime_module").then_some(declaration)
        }) else {
            panic!("expected embedded runtime module declaration");
        };
        assert_eq!(
            declaration.initial_value,
            apl_core::Expression::Literal(apl_core::Value::Str(encoded.to_owned()))
        );
        assert!(driver.contains(BOOTSTRAP_COMPILER_PROTOCOL));
        assert!(driver.contains(BOOTSTRAP_COMPILER_LIBRARY_REQUEST));
        assert!(driver.contains("bootstrap.linked_artifact_with_module_and_libraries_report"));
        assert!(!driver.contains(LEGACY_BOOTSTRAP_COMPILER_OK));
        assert!(!driver.contains(LEGACY_BOOTSTRAP_COMPILER_ERROR));
    }

    #[test]
    fn native_toolchain_protocol_and_driver_are_versioned() {
        let assembly = "# APLASM1 runtime-abi=1 target=x86_64-pc-windows-msvc\n\0";
        let encoded = format!(
            "{NATIVE_COMPILER_PROTOCOL}OK:{}:{assembly}",
            assembly.chars().count()
        );
        assert_eq!(parse_native_compiler_output(&encoded).unwrap(), assembly);

        let diagnostic = "bad source:\nexpected =\0";
        let encoded = format!(
            "{NATIVE_COMPILER_PROTOCOL}FAIL:{}:{diagnostic}",
            diagnostic.chars().count()
        );
        let CompileError::Bootstrap(message) = parse_native_compiler_output(&encoded).unwrap_err()
        else {
            panic!("expected native compiler diagnostic");
        };
        assert_eq!(message, diagnostic);

        for malformed in [
            "unknown",
            "APLNATIVE1:MAYBE:0:",
            "APLNATIVE1:OK:3:bad",
            "APLNATIVE1:OK:20:# APLASM1 short",
            "APLNATIVE1:FAIL:0:",
        ] {
            assert!(matches!(
                parse_native_compiler_output(malformed),
                Err(CompileError::Bootstrap(_))
            ));
        }

        let runtime_module = "APLMOD3:line\nquote=\" slash=\\ tab=\t nul=\0";
        let driver = render_native_toolchain_driver(runtime_module);
        apl_parser::parse_program(&driver).unwrap();
        assert!(driver.contains(NATIVE_COMPILER_PROTOCOL));
        assert!(driver.contains(NATIVE_COMPILER_REQUEST));
        assert!(driver.contains("native.lower_linked_artifact_report"));
        assert!(driver.contains("abi.lower_encoded_report"));
        assert!(driver.contains("assembly.emit_encoded_report"));
    }

    #[test]
    #[ignore = "expensive compiler-in-VM bootstrap gate"]
    fn bootstrap_compiler_artifact_compiles_inside_apl_vm() {
        let compiler = compile_bootstrap_compiler_artifact().unwrap();
        assert!(compiler.starts_with(LINKED_ARTIFACT_PROTOCOL));
        assert_eq!(
            inspect_linked_artifact(&compiler).unwrap(),
            "manifest APLMAN1\nbase\n"
        );

        let artifact = compile_linked_artifact_with_bootstrap_compiler(
            &compiler,
            "func double(value) { return value * 2 } out double(21)",
        )
        .unwrap();
        let output = run_linked_artifact(&artifact, vec![]).unwrap();
        assert_eq!(output.stdout, "42");
        assert_eq!(
            inspect_linked_artifact(&artifact).unwrap(),
            "manifest APLMAN1\nbase\n"
        );

        let math = compile_module_artifact(
            "func math.hidden(value) { return value + 1 } func math.answer() { return math.hidden(41) }",
        )
        .unwrap();
        let libraries = vec![NamedLibrary {
            name: "math".to_owned(),
            artifact: math,
        }];
        let artifact = compile_linked_artifact_with_bootstrap_compiler_and_libraries(
            &compiler,
            r#"include >> math exclude "math.hidden" << math out math.answer()"#,
            &libraries,
        )
        .unwrap();
        let output = run_linked_artifact(&artifact, vec![]).unwrap();
        assert_eq!(output.stdout, "42");
        assert_eq!(
            inspect_linked_artifact(&artifact).unwrap(),
            "manifest APLMAN1\nbase\nmath\n  exclude math.hidden\n"
        );

        let CompileError::Bootstrap(message) =
            compile_linked_artifact_with_bootstrap_compiler(&compiler, "AVInt value 1")
                .unwrap_err()
        else {
            panic!("expected nested compiler diagnostic");
        };
        assert_eq!(message, "parse error at 1:13: declaration expects =");
    }

    #[test]
    #[ignore = "expensive native self-host toolchain gate"]
    fn native_toolchain_compiles_and_runs_a_host_executable() {
        if std::env::consts::ARCH != "x86_64" || (!cfg!(windows) && !cfg!(target_os = "linux")) {
            return;
        }
        let target = host_native_target().unwrap();
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "apl_native_toolchain_{}_{}",
            std::process::id(),
            unique
        ));
        let toolchain_dir = directory.join("toolchain");
        let program_dir = directory.join("program");
        fs::create_dir_all(&directory).unwrap();

        let toolchain = build_native_toolchain_package(&toolchain_dir, target).unwrap();
        let built = Command::new("cargo")
            .arg("build")
            .arg("--manifest-path")
            .arg(&toolchain.manifest_path)
            .output()
            .unwrap();
        assert!(
            built.status.success(),
            "{}",
            String::from_utf8_lossy(&built.stderr)
        );
        let toolchain_executable = toolchain_dir.join("target").join("debug").join(format!(
            "{}{}",
            toolchain.package_name,
            std::env::consts::EXE_SUFFIX
        ));

        let CompileError::Bootstrap(message) =
            compile_assembly_with_native_toolchain(&toolchain_executable, "AVInt value 1", target)
                .unwrap_err()
        else {
            panic!("expected native toolchain diagnostic");
        };
        assert_eq!(message, "parse error at 1:13: declaration expects =");

        let source_path = directory.join("answer.apl");
        fs::create_dir_all(directory.join("libraries")).unwrap();
        let math = compile_module_artifact(
            "func math.hidden(value) { return value + 1 } func math.answer() { return math.hidden(41) }",
        )
        .unwrap();
        fs::write(directory.join("libraries").join("math.aplmod"), math).unwrap();
        fs::write(
            &source_path,
            "include >> math\nexclude \"math.hidden\" << math\nout 2 ** 3 ** 2\nout \"\\n\"\nout math.answer()\nstop\n",
        )
        .unwrap();
        let program = build_native_executable_with_native_toolchain(
            &toolchain_executable,
            &source_path,
            &program_dir,
            target,
        )
        .unwrap();
        let built = Command::new("cargo")
            .arg("build")
            .arg("--manifest-path")
            .arg(&program.manifest_path)
            .output()
            .unwrap();
        assert!(
            built.status.success(),
            "{}",
            String::from_utf8_lossy(&built.stderr)
        );
        let executable = program_dir.join("target").join("debug").join(format!(
            "{}{}",
            program.package_name,
            std::env::consts::EXE_SUFFIX
        ));
        let output = Command::new(executable).output().unwrap();
        let _ = fs::remove_dir_all(&directory);
        assert!(output.status.success());
        assert_eq!(String::from_utf8(output.stdout).unwrap(), "512\n42");
    }

    #[test]
    fn generated_linked_main_embeds_portable_artifact() {
        let rendered = render_linked_main();

        assert!(rendered.contains("APL_LINKED_ARTIFACT"));
        assert!(rendered.contains("program.apllink"));
        assert!(rendered.contains("run_linked_artifact"));
        assert!(rendered.contains("stdin.is_terminal()"));
        assert!(!rendered.contains("run_ir_bytes"));
        assert!(!rendered.contains("parse_program"));
    }
}
