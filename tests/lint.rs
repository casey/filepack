use super::*;

#[test]
fn allow_lint() {
  if cfg!(windows) {
    return;
  }

  Test::new().touch("aux").args(["create", "."]).success();
}

#[test]
fn deny_case_insensitive_filesystem_path_conflict() {
  if cfg!(windows) || cfg!(target_os = "macos") {
    return;
  }

  Test::new()
    .touch("foo")
    .touch("FOO")
    .args(["create", "--deny", "distribution", "."])
    .stderr(
      "\
error: paths would conflict on case-insensitive filesystem:
       ├─ `FOO`
       └─ `foo`
error: 1 lint error
",
    )
    .failure();
}

#[test]
fn deny_compatibility_ignores_junk() {
  if cfg!(windows) {
    return;
  }

  Test::new()
    .touch("aux")
    .touch(".DS_Store")
    .args(["create", "--deny", "compatibility", "."])
    .stderr(
      "\
error: path failed lint: `aux`
       └─ Windows does not allow files named `aux`
error: 1 lint error
",
    )
    .failure();
}

#[test]
fn deny_distribution_catches_both() {
  if cfg!(windows) {
    return;
  }

  Test::new()
    .touch(".DS_Store")
    .touch("aux")
    .args(["create", "--deny", "distribution", "."])
    .stderr(
      "\
error: path failed lint: `.DS_Store`
       └─ possible junk file
error: path failed lint: `aux`
       └─ Windows does not allow files named `aux`
error: 2 lint errors
",
    )
    .failure();
}

#[test]
fn deny_junk_ignores_compatibility() {
  if cfg!(windows) {
    return;
  }

  Test::new()
    .touch("aux")
    .touch(".DS_Store")
    .args(["create", "--deny", "junk", "."])
    .stderr(
      "\
error: path failed lint: `.DS_Store`
       └─ possible junk file
error: 1 lint error
",
    )
    .failure();
}

#[test]
fn deny_lint() {
  if cfg!(windows) {
    return;
  }

  Test::new()
    .touch("aux")
    .args(["create", "--deny", "distribution", "."])
    .stderr(
      "\
error: path failed lint: `aux`
       └─ Windows does not allow files named `aux`
error: 1 lint error
",
    )
    .failure();
}
