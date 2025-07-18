use super::*;

#[test]
fn creates_archive_for_multiple_files() {
  let tempdir = TempDir::new().unwrap();

  let files = [("sub/foo.txt", "hello"), ("sub/bar.txt", "goodbye")];

  for (path, content) in files {
    tempdir.child(path).write_str(content).unwrap();
  }

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "sub"])
    .current_dir(&tempdir)
    .assert()
    .success();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["archive", "sub", "--output", "output.filepack"])
    .current_dir(&tempdir)
    .assert()
    .success();

  let mut expected = Vec::new();

  expected.extend_from_slice(b"FILEPACK");

  let mut offset = 0u64;
  let manifest = std::fs::read_to_string(tempdir.child("sub/filepack.json")).unwrap();

  expected.extend_from_slice(blake3::hash(manifest.as_bytes()).as_bytes());

  let mut files = iter::once(manifest.as_str())
    .chain(files.iter().map(|&(_path, content)| content))
    .map(|content| (blake3::hash(content.as_bytes()), content.into()))
    .collect::<Vec<(Hash, Vec<u8>)>>();

  files.sort_by_key(|(hash, _content)| *hash.as_bytes());

  expected.extend_from_slice(&u64::try_from(files.len()).unwrap().to_le_bytes());

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

  let actual = std::fs::read(tempdir.child("output.filepack")).unwrap();
  assert_eq!(actual, expected);
}

#[test]
fn hash_mismatch_error() {
  let tempdir = TempDir::new().unwrap();

  let files = [("foo.txt", "hello"), ("bar.txt", "goodbye")];

  for (path, content) in files {
    tempdir.child(path).write_str(content).unwrap();
  }

  Command::cargo_bin("filepack")
    .unwrap()
    .arg("create")
    .current_dir(&tempdir)
    .assert()
    .success();

  tempdir.child("foo.txt").write_str("bazzz").unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["archive", "--output", "archive.filepack"])
    .current_dir(&tempdir)
    .assert()
    .failure()
    .stderr(is_match(
      "error: file `.*(/|\\\\)foo.txt` hash .* does not match manifest hash .*\n",
    ));
}

#[test]
fn size_mismatch_error() {
  let tempdir = TempDir::new().unwrap();

  let files = [("foo.txt", "hello"), ("bar.txt", "goodbye")];

  for (path, content) in files {
    tempdir.child(path).write_str(content).unwrap();
  }

  Command::cargo_bin("filepack")
    .unwrap()
    .arg("create")
    .current_dir(&tempdir)
    .assert()
    .success();

  tempdir.child("foo.txt").write_str("baz").unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["archive", "--output", "archive.filepack"])
    .current_dir(&tempdir)
    .assert()
    .failure()
    .stderr(is_match(
      "error: file `.*(/|\\\\)foo.txt` size 3 does not match manifest size 5\n",
    ));
}
