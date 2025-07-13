use super::*;

#[test]
fn creates_archive_for_multiple_files() {
  let dir = TempDir::new().unwrap();

  let test_files = [("bar", "hello world"), ("quux", "more content")];

  for (filename, content) in &test_files {
    dir.child(filename).write_str(content).unwrap();
  }

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["archive"])
    .current_dir(&dir)
    .assert()
    .success();

  dir
    .child("archive.filepack")
    .assert(predicates::path::exists());

  let archive_content = std::fs::read(dir.child("archive.filepack")).unwrap();
  let mut expected = Vec::new();
  expected.extend_from_slice(b"FILEPACK");

  let mut offset: u64 = 0;
  let manifest_content = std::fs::read_to_string(dir.child("filepack.json")).unwrap();
  let manifest_hash = blake3::hash(manifest_content.as_bytes());
  let manifest_content_length = &u64::try_from(manifest_content.len()).unwrap();

  expected.extend_from_slice(manifest_hash.as_bytes());
  expected.extend_from_slice(&offset.to_le_bytes());
  expected.extend_from_slice(&manifest_content_length.to_le_bytes());
  offset += manifest_content_length;

  for (_filename, content) in &test_files {
    let content_bytes = content.as_bytes();
    let content_length = u64::try_from(content_bytes.len()).unwrap();

    expected.extend_from_slice(blake3::hash(content_bytes).as_bytes());
    expected.extend_from_slice(&offset.to_le_bytes());
    expected.extend_from_slice(&content_length.to_le_bytes());

    offset += content_length;
  }

  expected.extend_from_slice(manifest_content.as_bytes());

  for (_filename, content) in &test_files {
    expected.extend_from_slice(content.as_bytes());
  }

  assert_eq!(archive_content, expected);
}
