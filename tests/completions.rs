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
fn paths() {
  Test::new()
    .write("foo", "")
    .env("FILEPACK_COMPLETE", "fish")
    .args(["--", "filepack", "hash", "f"])
    .stdout("foo\n")
    .success();
}

#[test]
fn subcommands() {
  Test::new()
    .env("FILEPACK_COMPLETE", "fish")
    .args(["--", "filepack", "ser"])
    .stdout("serve\tStart a filepack server\n")
    .success();
}
