use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

struct TestProject {
    root: PathBuf,
}

impl TestProject {
    fn new() -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "apl_cli_portable_{}_{}",
            std::process::id(),
            unique
        ));
        fs::create_dir_all(root.join("libraries")).unwrap();
        Self { root }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }
}

impl Drop for TestProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn apl() -> Command {
    Command::new(env!("CARGO_BIN_EXE_apl"))
}

#[test]
fn default_check_and_run_use_the_portable_apl_pipeline() {
    let project = TestProject::new();
    let main = project.path("main.apl");
    fs::write(&main, "func answer() { return 42 } out answer()").unwrap();

    let checked = apl().arg("check").arg(&main).output().unwrap();
    assert!(checked.status.success());
    assert_eq!(
        String::from_utf8(checked.stdout).unwrap(),
        "ok: portable linked program\n"
    );

    let run = apl().arg("run").arg(&main).output().unwrap();
    assert!(run.status.success());
    assert_eq!(String::from_utf8(run.stdout).unwrap(), "42");

    let invalid_source = "out \"\u{451}\u{436}\" AVInt value 1";
    let reference_error = apl_parser::parse_program(invalid_source).unwrap_err();
    assert_eq!((reference_error.line, reference_error.column), (1, 22));
    fs::write(&main, invalid_source).unwrap();
    let invalid = apl().arg("check").arg(&main).output().unwrap();
    assert!(!invalid.status.success());
    assert_eq!(
        String::from_utf8(invalid.stderr).unwrap(),
        "check error: parse error at 1:22: declaration expects =\n"
    );

    fs::write(&main, "out \"ok\"\nif true {\n  out missing\n}").unwrap();
    let semantic = apl().arg("check").arg(&main).output().unwrap();
    assert!(!semantic.status.success());
    assert_eq!(
        String::from_utf8(semantic.stderr).unwrap(),
        "check error: semantic error at 3:3: unknown variable `missing`\n"
    );
}

#[test]
fn default_run_resolves_named_modules_for_the_apl_compiler() {
    let project = TestProject::new();
    let library_source = project.path("math.apl");
    let library_artifact = project.path("libraries").join("math.aplmod");
    let main = project.path("main.apl");
    fs::write(
        &library_source,
        "func math.internal(value) { return value + 1 } func math.answer() { return math.internal(41) }",
    )
    .unwrap();
    fs::write(
        &main,
        "include >> math exclude \"math.internal\" << math out math.answer()",
    )
    .unwrap();

    let emitted = apl()
        .arg("emit-module")
        .arg(&library_source)
        .arg(&library_artifact)
        .output()
        .unwrap();
    assert!(
        emitted.status.success(),
        "{}",
        String::from_utf8_lossy(&emitted.stderr)
    );

    let run = apl().arg("run").arg(&main).output().unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8(run.stdout).unwrap(), "42");

    let low_ir = project.path("main.apllow");
    let emitted_low = apl()
        .arg("emit-native-ir")
        .arg(&main)
        .arg(&low_ir)
        .output()
        .unwrap();
    assert!(
        emitted_low.status.success(),
        "{}",
        String::from_utf8_lossy(&emitted_low.stderr)
    );
    assert!(fs::read_to_string(&low_ir)
        .unwrap()
        .starts_with(apl_compiler::NATIVE_IR_PROTOCOL));

    let inspected = apl()
        .arg("inspect-native-ir")
        .arg(&low_ir)
        .output()
        .unwrap();
    assert!(inspected.status.success());
    let description = String::from_utf8(inspected.stdout).unwrap();
    assert!(description.contains("low-ir APLLOW1"));
    assert!(description.contains("[CALL_SLOT, 1, 0]"));
    assert!(description.contains("manifest [APLMAN1, [[base, []], [math, [math.internal]]]]"));

    let native_abi = project.path("main.aplabi");
    let emitted_abi = apl()
        .arg("emit-native-abi")
        .arg(&main)
        .arg(apl_compiler::WINDOWS_X64_TARGET)
        .arg(&native_abi)
        .output()
        .unwrap();
    assert!(
        emitted_abi.status.success(),
        "{}",
        String::from_utf8_lossy(&emitted_abi.stderr)
    );
    assert!(fs::read_to_string(&native_abi)
        .unwrap()
        .starts_with(apl_compiler::NATIVE_ABI_PROTOCOL));

    let inspected_abi = apl()
        .arg("inspect-native-abi")
        .arg(&native_abi)
        .output()
        .unwrap();
    assert!(inspected_abi.status.success());
    let abi_description = String::from_utf8(inspected_abi.stdout).unwrap();
    assert!(abi_description.contains("native-abi APLABI1"));
    assert!(abi_description.contains("target x86_64-pc-windows-msvc win64"));
    assert!(abi_description.contains("CALL_SLOT [8, 1, 0]"));
    assert!(abi_description.contains("manifest [APLMAN1, [[base, []], [math, [math.internal]]]]"));
}
