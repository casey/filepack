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
      "
        error: paths would conflict on case-insensitive filesystem
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
      "
        error: path failed lint: `aux`
               └─ Windows does not allow files named `aux`
        error: 1 lint error
      ",
    )
    .failure();
}

#[test]
fn deny_content_ignores_generate_without_metadata() {
  Test::new()
    .args(["create", "--deny", "content", "."])
    .stderr(
      "
        error: package missing artwork
        error: 1 lint error
      ",
    )
    .failure();
}

#[test]
fn deny_content_requires_artwork() {
  #[track_caller]
  fn case(metadata: Option<&str>) {
    let mut test = Test::new();

    if let Some(metadata) = metadata {
      test = test.write("metadata.yaml", metadata);
    }

    test
      .args(["create", "--deny", "content", "--generate", "."])
      .stderr(
        "
          error: package missing artwork
          error: 1 lint error
        ",
      )
      .failure();
  }

  case(None);
  case(Some("title: foo"));
}

#[test]
fn deny_content_requires_embedded_cover_art() {
  #[track_caller]
  fn case(path: &str, data: Vec<u8>, cover: bool) {
    let test = Test::new()
      .write(path, data)
      .write("bar.png", PngBuilder::new().build())
      .write(
        "metadata.yaml",
        format!(
          "
            artwork: bar.png
            media:
              type: audio
              items:
                - {path}
          "
        ),
      )
      .args(["create", "--deny", "content", "--generate", "."]);

    if cover {
      test.success();
    } else {
      test
        .stderr(
          format!(
            "
              error: path failed lint: `{path}`
                     └─ audio file missing embedded front cover art
              error: 1 lint error
            "
          )
          .as_str(),
        )
        .failure();
    }
  }

  fn flac() -> FlacBuilder {
    FlacBuilder::new()
      .tag("ALBUM", "qux")
      .tag("ARTIST", "baz")
      .tag("DISCNUMBER", "1")
      .tag("DISCTOTAL", "1")
      .tag("TITLE", "bar")
      .tag("TRACKNUMBER", "1")
      .tag("TRACKTOTAL", "1")
  }

  fn mp3() -> Mp3Builder {
    Mp3Builder::new()
      .tag("TALB", "qux")
      .tag("TIT2", "bar")
      .tag("TPE1", "baz")
      .tag("TPOS", "1/1")
      .tag("TRCK", "1/1")
      .frames(1)
  }

  case("foo.flac", flac().build(), false);
  case("foo.flac", flac().picture(3).build(), true);
  case("foo.flac", flac().picture(4).build(), false);
  case("foo.mp3", mp3().build(), false);
  case("foo.mp3", mp3().picture(3).build(), true);
  case("foo.mp3", mp3().picture(4).build(), false);
}

#[test]
fn deny_content_requires_generate() {
  Test::new()
    .write("foo.png", PngBuilder::new().build())
    .write("metadata.yaml", "artwork: foo.png")
    .args(["create", "--deny", "content", "."])
    .stderr(
      "
        error: derived assets not generated, pass `--generate`
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
      "
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
      "
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
      "
        error: path failed lint: `aux`
               └─ Windows does not allow files named `aux`
        error: 1 lint error
      ",
    )
    .failure();
}
