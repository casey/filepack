use super::*;

#[test]
fn creates_archive_for_multiple_files() {
  let tempdir = TempDir::new().unwrap();

  let files = [("foo", "hello"), ("bar", "goodbye")];

  for (path, content) in files {
    tempdir.child(path).write_str(content).unwrap();
  }

  Command::cargo_bin("filepack")
    .unwrap()
    .arg("create")
    .current_dir(&tempdir)
    .assert()
    .success();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["archive", "--output", "archive.filepack"])
    .current_dir(&tempdir)
    .assert()
    .success();

  let mut expected = Vec::new();

  expected.extend_from_slice(b"FILEPACK");

  let mut offset = 0u64;
  let manifest = std::fs::read_to_string(tempdir.child("filepack.json")).unwrap();

  let mut files = iter::once(manifest.as_str())
    .chain(files.iter().map(|&(_path, content)| content))
    .map(|content| (blake3::hash(content.as_bytes()), content.into()))
    .collect::<Vec<(Hash, Vec<u8>)>>();

  files.sort_by_key(|(hash, _content)| *hash.as_bytes());

  for (hash, content) in &files {
    let size = u64::try_from(content.len()).unwrap();
    expected.extend_from_slice(hash.as_bytes());
    expected.extend_from_slice(&offset.to_le_bytes());
    expected.extend_from_slice(&size.to_le_bytes());
    offset += size;
  }

  for (_hash, content) in &files {
    expected.extend_from_slice(content);
  }

  let actual = std::fs::read(tempdir.child("archive.filepack")).unwrap();
  assert_eq!(actual, expected);
}
