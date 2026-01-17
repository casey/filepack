use super::*;

#[test]
fn files() {
  Test::new()
    .touch("content")
    .touch("cover.png")
    .touch("info.nfo")
    .touch("README.md")
    .write(
      "metadata.yaml",
      "\
title: Foo
artwork: cover.png
nfo: info.nfo
readme: README.md
",
    )
    .arg("create")
    .success()
    .arg("verify")
    .stderr("successfully verified 5 files totaling 62 bytes\n")
    .success();
}

#[test]
fn files_missing() {
  #[track_caller]
  fn case(metadata: &str, stderr: &str) {
    Test::new()
      .write("metadata.yaml", metadata)
      .arg("create")
      .stderr_regex(stderr)
      .failure();
  }

  case(
    "title: Foo\nartwork: cover.png",
    "error: file referenced in metadata missing: `cover.png`\n",
  );

  case(
    "title: Foo\nnfo: info.nfo",
    "error: file referenced in metadata missing: `info.nfo`\n",
  );

  case(
    "title: Foo\nreadme: README.md",
    "error: file referenced in metadata missing: `README.md`\n",
  );
}

#[test]
fn files_wrong_extension() {
  #[track_caller]
  fn case(metadata: &str, file: &str, stderr: &str) {
    Test::new()
      .touch(file)
      .write("metadata.yaml", metadata)
      .arg("create")
      .stderr_regex(stderr)
      .failure();
  }

  case(
    "title: Foo\nartwork: cover.jpg",
    "cover.jpg",
    ".*component must end in `.png`.*",
  );

  case(
    "title: Foo\nnfo: info.txt",
    "info.txt",
    ".*component must end in `.nfo`.*",
  );

  case(
    "title: Foo\nreadme: README.txt",
    "README.txt",
    ".*component must end in `.md`.*",
  );
}

#[test]
fn invalid_language() {
  Test::new()
    .write("metadata.yaml", "title: Foo\nlanguage: ac")
    .arg("create")
    .stderr_regex(".*unknown language code `ac`.*")
    .failure();
}

#[test]
fn language() {
  Test::new()
    .touch("content")
    .write("metadata.yaml", "title: Foo\nlanguage: en")
    .arg("create")
    .success()
    .arg("verify")
    .stderr("successfully verified 2 files totaling 23 bytes\n")
    .success();
}

#[test]
fn unknown_keys() {
  Test::new()
    .write("metadata.yaml", "title: Foo\nbar: baz")
    .arg("create")
    .stderr_regex(".*unknown fields in metadata at `.*metadata.yaml`: `bar`\n")
    .failure();
}

#[test]
fn dates() {
  Test::new()
    .touch("content")
    .write(
      "metadata.yaml",
      "
title: Foo
package-date: 2024-06-15T12:30:00+05:00
release-date: 2024-01-01
",
    )
    .arg("create")
    .success()
    .arg("verify")
    .stderr("successfully verified 2 files totaling 77 bytes\n")
    .success();
}

#[test]
fn invalid_package_date() {
  Test::new()
    .write("metadata.yaml", "title: Foo\npackage-date: not-a-date")
    .arg("create")
    .stderr_regex(".*invalid characters.*")
    .failure();
}

#[test]
fn invalid_release_date() {
  Test::new()
    .write("metadata.yaml", "title: Foo\nrelease-date: 2024/06/15")
    .arg("create")
    .stderr_regex(".*invalid characters.*")
    .failure();
}
