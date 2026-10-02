use std::{
    env, fs,
    io::Read,
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
                let mut stdin = String::new();
                let _ = std::io::stdin().read_to_string(&mut stdin);
                let input = stdin.lines().map(str::to_owned).collect();

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
            let mut stdin = String::new();
            let _ = std::io::stdin().read_to_string(&mut stdin);
            let input = stdin.lines().map(str::to_owned).collect();

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
        "run-linked" => {
            let Some(path) = args.next() else {
                eprintln!("missing linked artifact path");
                process::exit(2);
            };
            let artifact = fs::read_to_string(&path).unwrap_or_else(|error| {
                eprintln!("failed to read `{path}`: {error}");
                process::exit(1);
            });
            let mut stdin = String::new();
            let _ = std::io::stdin().read_to_string(&mut stdin);
            let input = stdin.lines().map(str::to_owned).collect();

            match apl_compiler::run_linked_artifact(&artifact, input) {
                Ok(output) => print!("{}", output.stdout),
                Err(error) => exit_compile_error("run-linked", error),
            }
        }
        "build" | "compile" => {
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
                    if command == "compile" {
                        compile_generated_package(&output);
                    } else {
                        println!(
                            "build with: cargo build --manifest-path {}",
                            output.manifest_path.display()
                        );
                    }
                }
                Err(error) => exit_compile_error("build", error),
            }
        }
        _ => {
            eprintln!("unknown command `{command}`");
            print_usage();
            process::exit(2);
        }
    }
}

fn print_usage() {
    eprintln!(
        "usage: apl <check|run|emit|run-ir|emit-linked|run-linked|build|compile> <file> [output]"
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

fn compile_generated_package(output: &apl_compiler::BuildOutput) {
    let status = process::Command::new("cargo")
        .arg("build")
        .arg("--manifest-path")
        .arg(&output.manifest_path)
        .status()
        .unwrap_or_else(|error| {
            eprintln!("failed to invoke cargo build: {error}");
            process::exit(1);
        });

    if !status.success() {
        process::exit(status.code().unwrap_or(1));
    }

    let executable_name = format!("{}{}", output.package_name, env::consts::EXE_SUFFIX);
    let executable_path = output
        .package_dir
        .join("target")
        .join("debug")
        .join(executable_name);
    println!("executable: {}", executable_path.display());
}
