use std::path::Path;
use std::process::{Command, Output};
use std::fs;
use std::os::unix::fs::PermissionsExt;

fn test_execution(program: &str) -> Result<Output, std::io::Error> {
    let temp_file = Path::new("/tmp/test_program");
    crate::compile(program, temp_file);
    let executable = PermissionsExt::from_mode(0o700);
    fs::set_permissions(temp_file, executable)?;
    Command::new(temp_file)
        .output()
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
        let Output { status, stdout, stderr } = test_execution(program).expect("failed to execute program");
        assert_eq!(status.code(), Some(0));
        assert_eq!(stdout, "Hello world\n".as_bytes().to_vec());
        assert_eq!(stderr, vec![]);
    }
}
