#![cfg(test)]
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use rand::distr::{Alphanumeric, SampleString};

use crate::assembling::assemble;
use crate::compiling::CompilationResult;
use crate::linking;

fn test_full_compiler(program: &str) -> Result<Output, std::io::Error> {
    let temp_file = &random_file_name();
    crate::compile(program, temp_file);
    test_executable(temp_file)
}

pub fn test_assembler(input: CompilationResult) -> Result<Output, std::io::Error> {
    let assembled = assemble(input);
    let linked = linking::elf::create_elf(assembled);
    let temp_file = &random_file_name();
    fs::write(temp_file, linked).expect("failed to write to file");
    test_executable(temp_file)
}

fn test_executable(path: &Path) -> Result<Output, std::io::Error> {
    let executable = PermissionsExt::from_mode(0o700);
    fs::set_permissions(path, executable)?;
    Command::new(path)
        .output()
}

fn random_file_name() -> PathBuf {
    let mut path = "/tmp/test_program".to_string();
    let mut rng = rand::rng();
    Alphanumeric.append_string(&mut rng, &mut path, 20);
    PathBuf::from(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_world() {
        let program = r#"
            Int main() {
                print("Hello world\n");
                return 0;
            }
        "#;
        let Output { status, stdout, stderr } = test_full_compiler(program).expect("failed to execute program");
        assert_eq!(status.code(), Some(0));
        assert_eq!(stdout, "Hello world\n".as_bytes().to_vec());
        assert_eq!(stderr, vec![]);
    }

    #[test]
    fn test_return_code() {
        let program = include_str!("../test_programs/ReturnCode.poo");
        let Output { status, stdout, stderr } = test_full_compiler(program).expect("failed to execute program");
        assert_eq!(status.code(), Some(123));
        assert_eq!(stdout, vec![]);
        assert_eq!(stderr, vec![]);
    }

    #[test]
    fn test_stdout_syscall() {
        let program = include_str!("../test_programs/StdoutSyscall.poo");
        let Output { status, stdout, stderr } = test_full_compiler(program).expect("failed to execute program");
        assert_eq!(status.code(), Some(0));
        assert_eq!(stdout, "Hello World!".as_bytes().to_vec());
        assert_eq!(stderr, vec![]);
    }
}
