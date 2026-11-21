use super::*;

#[test]
fn force_color() {
  Test::new()
    .arg("create")
    .success()
    .env("FORCE_COLOR", "foo")
    .arg("state")
    .stdout("{\n  \u{1b}[34;1m\"servers\"\u{1b}[0m: {}\n}\n")
    .success();
}

#[test]
fn force_color_clap() {
  Test::new()
    .env("FORCE_COLOR", "foo")
    .args(["create", "--deny", "bar"])
    .stderr(
      "
        \u{1b}[1m\u{1b}[31merror:\u{1b}[0m invalid value '\u{1b}[33mbar\u{1b}[0m' for '\u{1b}[1m\u{1b}[34m--deny <LINT>\u{1b}[0m': unknown lint or lint group `bar`

        For more information, try '\u{1b}[1m\u{1b}[34m--help\u{1b}[0m'.
      ",
    )
    .status(2);
}

#[test]
fn force_color_empty() {
  Test::new()
    .arg("create")
    .success()
    .env("FORCE_COLOR", "")
    .arg("state")
    .stdout(json_pretty! {
      servers: {},
    })
    .success();
}

#[test]
fn force_color_log() {
  let server = Test::new().serve().spawn();

  Test::new()
    .arg("create")
    .success()
    .env("FILEPACK_LOG", "filepack=debug")
    .env("FORCE_COLOR", "foo")
    .args(["upload", "--server", &server.address()])
    .stderr_regex(
      r"
        uploading 0 of 0 files
        \x1b\[2m\S+\x1b\[0m \x1b\[34mDEBUG\x1b\[0m \x1b\[2mfilepack::subcommand::upload\x1b\[0m\x1b\[2m:\x1b\[0m uploading directory `\.`
        created package number 0
      ",
    )
    .success();

  server.terminate().success();
}

#[test]
fn force_color_stderr() {
  Test::new()
    .env("FORCE_COLOR", "foo")
    .arg("verify")
    .stderr("\u{1b}[31merror\u{1b}[0m: \u{1b}[1mmanifest `manifest.filepack` not found\u{1b}[0m\n")
    .failure();
}

#[test]
fn no_color() {
  Test::new()
    .arg("create")
    .success()
    .env("FORCE_COLOR", "foo")
    .env("NO_COLOR", "bar")
    .arg("state")
    .stdout(json_pretty! {
      servers: {},
    })
    .success();
}

#[test]
fn no_color_empty() {
  Test::new()
    .arg("create")
    .success()
    .env("FORCE_COLOR", "foo")
    .env("NO_COLOR", "")
    .arg("state")
    .stdout("{\n  \u{1b}[34;1m\"servers\"\u{1b}[0m: {}\n}\n")
    .success();
}
