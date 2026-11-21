use super::*;

#[test]
fn completion_scripts() {
  for shell in ["bash", "elvish", "fish", "powershell", "zsh"] {
    Test::new()
      .args(["completions", shell])
      .stdout_regex("(?s).*FILEPACK_COMPLETE.*")
      .success();
  }
}

#[test]
fn lints() {
  Test::new()
    .env("FILEPACK_COMPLETE", "fish")
    .args(["--", "filepack", "create", "--allow", "t"])
    .stdout("time-missing\tMetadata has a time.\ntitle-missing\tMetadata has a title.\n")
    .success();
}

#[test]
fn paths() {
  Test::new()
    .write("foo", "")
    .env("FILEPACK_COMPLETE", "fish")
    .args(["--", "filepack", "hash", "f"])
    .stdout("foo\n")
    .success();
}

#[test]
fn servers() {
  for subcommand in ["forget", "upload"] {
    Test::new()
      .write(
        ".filepack/state.json",
        json! {
          servers: {
            "https://foo/": {
              number: 0,
              revision: "revision12c7922354e5c2496ffc5e87ee0b29cc8767dc35a7c0e31e9d239014df5b44583"
            }
          }
        },
      )
      .env("FILEPACK_COMPLETE", "fish")
      .args(["--", "filepack", subcommand, "--server", ""])
      .stdout("https://foo/\n")
      .success();
  }
}

#[test]
fn subcommands() {
  Test::new()
    .env("FILEPACK_COMPLETE", "fish")
    .args(["--", "filepack", "ser"])
    .stdout("serve\tStart Filepack server\n")
    .success();
}
