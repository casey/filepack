use super::*;

#[test]
fn broken_pipe_is_ignored() {
  let (reader, writer) = io::pipe().unwrap();

  drop(reader);

  let output = Command::new(env!("CARGO_BIN_EXE_filepack"))
    .arg("languages")
    .stdin(Stdio::null())
    .stdout(writer)
    .stderr(Stdio::piped())
    .output()
    .unwrap();

  assert!(output.status.success());

  assert_eq!(str::from_utf8(&output.stderr).unwrap(), "");
}
