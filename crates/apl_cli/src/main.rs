use std::{
    env, fs,
    io::{IsTerminal, Read},
    path::{Path, PathBuf},
    process,
};

fn main() {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        process::exit(2);
    };

    match command.as_str() {
        "check" | "run" => {
            let Some(path) = args.next() else {
                eprintln!("missing source path");
                process::exit(2);
            };

            let source = fs::read_to_string(&path).unwrap_or_else(|error| {
                eprintln!("failed to read `{path}`: {error}");
                process::exit(1);
            });

            let source = apl_compiler::compose_program(apl_compiler::STANDARD_PRELUDE, &source);
            let program = match apl_parser::parse_program(&source) {
                Ok(program) => program,
                Err(error) => {
                    eprintln!("parse error: {}", error.message);
                    process::exit(1);
                }
            };

            if let Err(error) = apl_core::validate_program(&program) {
                eprintln!("check error: {error:?}");
                process::exit(1);
            }

            if command == "check" {
                println!("ok: {} statement(s)", program.statements.len());
            } else {
                let input = read_input();

                match apl_runtime::run_program(&program, input) {
                    Ok(output) => print!("{}", output.stdout),
                    Err(apl_runtime::RuntimeError::Failed(message)) => {
                        eprintln!("APL fail: {message}");
                        process::exit(1);
                    }
                    Err(error) => {
                        eprintln!("runtime error: {error:?}");
                        process::exit(1);
                    }
                }
            }
        }
        "emit" => {
            let Some(path) = args.next() else {
                eprintln!("missing source path");
                process::exit(2);
            };
            let output_path = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(|| default_ir_path(&path));

            match apl_compiler::emit_ir_file(
                Path::new(&path),
                &output_path,
                apl_compiler::STANDARD_PRELUDE,
            ) {
                Ok(()) => println!("emitted: {}", output_path.display()),
                Err(error) => exit_compile_error("emit", error),
            }
        }
        "run-ir" => {
            let Some(path) = args.next() else {
                eprintln!("missing IR path");
                process::exit(2);
            };
            let bytes = fs::read(&path).unwrap_or_else(|error| {
                eprintln!("failed to read `{path}`: {error}");
                process::exit(1);
            });
            let input = read_input();

            match apl_runtime::run_ir_bytes(&bytes, input) {
                Ok(output) => print!("{}", output.stdout),
                Err(apl_runtime::RuntimeError::Failed(message)) => {
                    eprintln!("APL fail: {message}");
                    process::exit(1);
                }
                Err(error) => {
                    eprintln!("runtime error: {error:?}");
                    process::exit(1);
                }
            }
        }
        "emit-linked" => {
            let Some(path) = args.next() else {
                eprintln!("missing source path");
                process::exit(2);
            };
            let output_path = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(|| default_linked_path(&path));

            match apl_compiler::emit_linked_artifact_file(Path::new(&path), &output_path) {
                Ok(()) => println!("emitted linked artifact: {}", output_path.display()),
                Err(error) => exit_compile_error("emit-linked", error),
            }
        }
        "emit-module" => {
            let Some(path) = args.next() else {
                eprintln!("missing module source path");
                process::exit(2);
            };
            let output_path = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(|| default_module_path(&path));

            match apl_compiler::emit_module_artifact_file(Path::new(&path), &output_path) {
                Ok(()) => println!("emitted module artifact: {}", output_path.display()),
                Err(error) => exit_compile_error("emit-module", error),
            }
        }
        "emit-standard-module" => {
            let output_path = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("build").join("standard.aplmod"));

            match apl_compiler::emit_standard_module_artifact_file(&output_path) {
                Ok(()) => println!("emitted standard module: {}", output_path.display()),
                Err(error) => exit_compile_error("emit-standard-module", error),
            }
        }
        "extend-module" => {
            let Some(module_path) = args.next() else {
                eprintln!("missing module artifact path");
                process::exit(2);
            };
            let Some(source_path) = args.next() else {
                eprintln!("missing module source path");
                process::exit(2);
            };
            let output_path = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(|| default_module_path(&source_path));

            match apl_compiler::emit_extended_module_artifact_file(
                Path::new(&module_path),
                Path::new(&source_path),
                &output_path,
            ) {
                Ok(()) => println!("emitted module artifact: {}", output_path.display()),
                Err(error) => exit_compile_error("extend-module", error),
            }
        }
        "emit-linked-module" => {
            let Some(module_path) = args.next() else {
                eprintln!("missing module artifact path");
                process::exit(2);
            };
            let Some(source_path) = args.next() else {
                eprintln!("missing source path");
                process::exit(2);
            };
            let output_path = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(|| default_linked_path(&source_path));

            match apl_compiler::emit_linked_artifact_with_module_file(
                Path::new(&module_path),
                Path::new(&source_path),
                &output_path,
            ) {
                Ok(()) => println!("emitted linked artifact: {}", output_path.display()),
                Err(error) => exit_compile_error("emit-linked-module", error),
            }
        }
        "run-linked" => {
            let Some(path) = args.next() else {
                eprintln!("missing linked artifact path");
                process::exit(2);
            };
            let artifact = fs::read_to_string(&path).unwrap_or_else(|error| {
                eprintln!("failed to read `{path}`: {error}");
                process::exit(1);
            });
            let input = read_input();

            match apl_compiler::run_linked_artifact(&artifact, input) {
                Ok(output) => print!("{}", output.stdout),
                Err(error) => exit_compile_error("run-linked", error),
            }
        }
        "build" | "compile" | "build-linked" | "compile-linked" => {
            let Some(path) = args.next() else {
                eprintln!("missing source path");
                process::exit(2);
            };
            let output_dir = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(|| default_build_dir(&path));

            match apl_compiler::build_linked_executable_package(Path::new(&path), &output_dir) {
                Ok(output) => {
                    println!("generated: {}", output.package_dir.display());
                    println!("manifest: {}", output.manifest_path.display());
                    println!("artifact: {}", output.artifact_path.display());
                    println!("source: {}", output.source_path.display());
                    if command == "compile" || command == "compile-linked" {
                        compile_generated_package(
                            &output.package_name,
                            &output.package_dir,
                            &output.manifest_path,
                        );
                    } else {
                        println!(
                            "build with: cargo build --manifest-path {}",
                            output.manifest_path.display()
                        );
                    }
                }
                Err(error) => exit_compile_error("build-linked", error),
            }
        }
        "build-host" | "compile-host" => {
            let Some(path) = args.next() else {
                eprintln!("missing source path");
                process::exit(2);
            };
            let output_dir = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(|| default_build_dir(&path));

            match apl_compiler::build_executable_package(
                Path::new(&path),
                &output_dir,
                apl_compiler::STANDARD_PRELUDE,
            ) {
                Ok(output) => {
                    println!("generated: {}", output.package_dir.display());
                    println!("manifest: {}", output.manifest_path.display());
                    println!("source: {}", output.source_path.display());
                    if command == "compile-host" {
                        compile_generated_package(
                            &output.package_name,
                            &output.package_dir,
                            &output.manifest_path,
                        );
                    } else {
                        println!(
                            "build with: cargo build --manifest-path {}",
                            output.manifest_path.display()
                        );
                    }
                }
                Err(error) => exit_compile_error("build-host", error),
            }
        }
        _ => {
            eprintln!("unknown command `{command}`");
            print_usage();
            process::exit(2);
        }
    }
}

fn read_input() -> Vec<String> {
    let mut stdin = std::io::stdin();
    if stdin.is_terminal() {
        return Vec::new();
    }

    let mut input = String::new();
    let _ = stdin.read_to_string(&mut input);
    input.lines().map(str::to_owned).collect()
}

fn print_usage() {
    eprintln!(
        "usage: apl <check|run|emit|run-ir|emit-module|emit-standard-module|extend-module|emit-linked|emit-linked-module|run-linked|build|compile|build-linked|compile-linked|build-host|compile-host> <file> [args]"
    );
}

fn default_build_dir(path: &str) -> PathBuf {
    let stem = Path::new(path)
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("program");
    PathBuf::from("build").join(stem)
}

fn default_ir_path(path: &str) -> PathBuf {
    let stem = Path::new(path)
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("program");
    PathBuf::from("build").join(format!("{stem}.aplc"))
}

fn default_linked_path(path: &str) -> PathBuf {
    let stem = Path::new(path)
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("program");
    PathBuf::from("build").join(format!("{stem}.apllink"))
}

fn default_module_path(path: &str) -> PathBuf {
    let stem = Path::new(path)
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("module");
    PathBuf::from("build").join(format!("{stem}.aplmod"))
}

fn exit_compile_error(verb: &str, error: apl_compiler::CompileError) -> ! {
    match error {
        apl_compiler::CompileError::Io(error) => eprintln!("{verb} io error: {error}"),
        apl_compiler::CompileError::Ir(error) => eprintln!("{verb} IR error: {error:?}"),
        apl_compiler::CompileError::Parse(error) => {
            eprintln!("{verb} parse error: {}", error.message)
        }
        apl_compiler::CompileError::Check(error) => eprintln!("{verb} check error: {error:?}"),
        apl_compiler::CompileError::Runtime(apl_runtime::RuntimeError::Failed(message)) => {
            eprintln!("{verb} APL fail: {message}")
        }
        apl_compiler::CompileError::Runtime(error) => {
            eprintln!("{verb} runtime error: {error:?}")
        }
        apl_compiler::CompileError::Bootstrap(message) => {
            eprintln!("{verb} bootstrap error: {message}")
        }
    }
    process::exit(1);
}

fn compile_generated_package(package_name: &str, package_dir: &Path, manifest_path: &Path) {
    let status = process::Command::new("cargo")
        .arg("build")
        .arg("--manifest-path")
        .arg(manifest_path)
        .status()
        .unwrap_or_else(|error| {
            eprintln!("failed to invoke cargo build: {error}");
            process::exit(1);
        });

    if !status.success() {
        process::exit(status.code().unwrap_or(1));
    }

    let executable_name = format!("{}{}", package_name, env::consts::EXE_SUFFIX);
    let executable_path = package_dir
        .join("target")
        .join("debug")
        .join(executable_name);
    println!("executable: {}", executable_path.display());
}
