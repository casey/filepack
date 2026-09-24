use super::*;

#[test]
fn filepack_magic_is_recognized_by_file_command() {
  if !cfg!(unix) {
    return;
  }

  let test = Test::new()
    .write("metadata.yaml", "title: Foo")
    .arg("create")
    .success();

  fs::write(test.path().join("foo"), b"\x89filepack\0\x83foo").unwrap();
  fs::write(test.path().join("bar"), b"\x88filepack\x88archive\0").unwrap();
  fs::write(test.path().join("baz"), b"\0baz").unwrap();

  let cases = [
    ("manifest.filepack", "filepack archive\n"),
    ("metadata.filemeta", "filepack metadata\n"),
    ("foo", "filepack\n"),
    ("bar", "data\n"),
    ("baz", "data\n"),
  ];

  for (path, expected) in cases {
    let path = test.path().join(path);

    let output = Command::new("file")
      .args([
        "--brief",
        "--magic-file",
        concat!(env!("CARGO_MANIFEST_DIR"), "/filepack.magic"),
      ])
      .arg(path)
      .output()
      .unwrap();
    assert!(output.status.success());
    assert_eq!(str::from_utf8(&output.stderr).unwrap(), "");
    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
  }
}
