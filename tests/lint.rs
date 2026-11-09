use super::*;

#[test]
fn allow_group() {
  if cfg!(windows) {
    return;
  }

  Test::new()
    .touch(".DS_Store")
    .touch("aux")
    .args(["create", "--deny", "distribution", "--allow", "hygiene"])
    .stderr(
      "
        error: Windows does not allow files named `aux` [windows-reserved-filename]
               └─ `aux`
        error: 1 lint error
      ",
    )
    .failure();
}

#[test]
fn allow_lint() {
  if cfg!(windows) {
    return;
  }

  Test::new().touch("aux").args(["create"]).success();
}

#[test]
fn allow_removes_denied_lint() {
  Test::new()
    .touch(".DS_Store")
    .args(["create", "--deny", "distribution", "--allow", "junk"])
    .success();
}

#[test]
fn deny_artwork_missing() {
  #[track_caller]
  fn case(metadata: Option<&str>) {
    let mut test = Test::new();

    if let Some(metadata) = metadata {
      test = test.write("metadata.yaml", metadata);
    }

    test
      .args(["create", "--deny", "artwork-missing"])
      .stderr(
        "
          error: metadata missing artwork [artwork-missing]
          error: 1 lint error
        ",
      )
      .failure();
  }

  case(None);
  case(Some("title: foo"));
}

#[test]
fn deny_audio_embedded_artwork_aspect_ratio() {
  #[track_caller]
  fn case(path: &str, data: Vec<u8>, square: bool) {
    let test = Test::new()
      .write(path, data)
      .write(
        "metadata.yaml",
        format!(
          "
            creator: baz
            title: qux
            media:
              type: audio
              items:
                - path: {path}
          "
        ),
      )
      .args(["create", "--deny", "audio-embedded-artwork-aspect-ratio"]);

    if square {
      test.success();
    } else {
      test
        .stderr(
          format!(
            "
              error: audio file embedded front cover art is 2×1 but must be square [audio-embedded-artwork-aspect-ratio]
                     └─ `{path}`
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

  fn m4a() -> Mp4Builder {
    Mp4Builder::new()
      .tag(*b"\xa9alb", "qux")
      .tag(*b"\xa9ART", "baz")
      .tag(*b"\xa9nam", "bar")
      .pair_tag(*b"disk", 1, 1)
      .pair_tag(*b"trkn", 1, 1)
      .frame_count(1)
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

  let square = PngBuilder::new().build();
  let wide = PngBuilder::new().width(2).build();

  case("foo.flac", flac().build(), true);
  case("foo.flac", flac().picture(3, &square).build(), true);
  case("foo.flac", flac().picture(3, &wide).build(), false);
  case("foo.flac", flac().picture(4, &wide).build(), true);
  case("foo.mp3", mp3().build(), true);
  case("foo.mp3", mp3().picture(3, &square).build(), true);
  case("foo.mp3", mp3().picture(3, &wide).build(), false);
  case("foo.mp3", mp3().picture(4, &wide).build(), true);
  case("foo.m4a", m4a().audio_track(0x40).build(), true);
  case(
    "foo.m4a",
    m4a().picture(14, &square).audio_track(0x40).build(),
    true,
  );
  case(
    "foo.m4a",
    m4a().picture(14, &wide).audio_track(0x40).build(),
    false,
  );
}

#[test]
fn deny_audio_embedded_artwork_missing() {
  #[track_caller]
  fn case(path: &str, data: Vec<u8>, cover: bool) {
    let test = Test::new()
      .write(path, data)
      .write(
        "metadata.yaml",
        format!(
          "
            creator: baz
            title: qux
            media:
              type: audio
              items:
                - path: {path}
          "
        ),
      )
      .args(["create", "--deny", "audio-embedded-artwork-missing"]);

    if cover {
      test.success();
    } else {
      test
        .stderr(
          format!(
            "
              error: audio file missing embedded front cover art [audio-embedded-artwork-missing]
                     └─ `{path}`
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

  fn m4a() -> Mp4Builder {
    Mp4Builder::new()
      .tag(*b"\xa9alb", "qux")
      .tag(*b"\xa9ART", "baz")
      .tag(*b"\xa9nam", "bar")
      .pair_tag(*b"disk", 1, 1)
      .pair_tag(*b"trkn", 1, 1)
      .frame_count(1)
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
  case("foo.flac", flac().picture(3, b"foo").build(), true);
  case("foo.flac", flac().picture(4, b"foo").build(), false);
  case("foo.mp3", mp3().build(), false);
  case("foo.mp3", mp3().picture(3, b"foo").build(), true);
  case("foo.mp3", mp3().picture(4, b"foo").build(), false);
  case("foo.m4a", m4a().audio_track(0x40).build(), false);
  case(
    "foo.m4a",
    m4a().picture(13, b"foo").audio_track(0x40).build(),
    true,
  );
}

#[test]
fn deny_case_conflict() {
  if cfg!(windows) || cfg!(target_os = "macos") {
    return;
  }

  Test::new()
    .touch("foo")
    .touch("FOO")
    .args(["create", "--deny", "case-conflict"])
    .stderr(
      "
        error: paths would conflict on case-insensitive filesystem [case-conflict]
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
    .args(["create", "--deny", "compatibility"])
    .stderr(
      "
        error: Windows does not allow files named `aux` [windows-reserved-filename]
               └─ `aux`
        error: 1 lint error
      ",
    )
    .failure();
}

#[test]
fn deny_creator_missing() {
  Test::new()
    .write("metadata.yaml", "title: foo")
    .args(["create", "--deny", "creator-missing"])
    .stderr(
      "
        error: metadata missing creator [creator-missing]
        error: 1 lint error
      ",
    )
    .failure();
}

#[test]
fn deny_derived_assets_missing() {
  Test::new()
    .write("foo.png", PngBuilder::new().build())
    .write("metadata.yaml", "artwork: foo.png")
    .args(["create", "--deny", "derived-assets-missing"])
    .stderr(
      "
        error: derived assets not generated, pass `--generate` [derived-assets-missing]
        error: 1 lint error
      ",
    )
    .failure();
}

#[test]
fn deny_derived_assets_missing_ignores_metadata_without_assets() {
  Test::new()
    .write("metadata.yaml", "title: foo")
    .args(["create", "--deny", "derived-assets-missing"])
    .success();
}

#[test]
fn deny_derived_assets_missing_ignores_missing_metadata() {
  Test::new()
    .args(["create", "--deny", "derived-assets-missing"])
    .success();
}

#[test]
fn deny_distribution_catches_both() {
  if cfg!(windows) {
    return;
  }

  Test::new()
    .touch(".DS_Store")
    .touch("aux")
    .args(["create", "--deny", "distribution"])
    .stderr(
      "
        error: possible junk file [junk]
               └─ `.DS_Store`
        error: Windows does not allow files named `aux` [windows-reserved-filename]
               └─ `aux`
        error: 2 lint errors
      ",
    )
    .failure();
}

#[test]
fn deny_hygiene_ignores_compatibility() {
  if cfg!(windows) {
    return;
  }

  Test::new()
    .touch("aux")
    .touch(".DS_Store")
    .args(["create", "--deny", "hygiene"])
    .stderr(
      "
        error: possible junk file [junk]
               └─ `.DS_Store`
        error: 1 lint error
      ",
    )
    .failure();
}

#[test]
fn deny_item_title_missing() {
  Test::new()
    .write("foo.png", PngBuilder::new().build())
    .write("bar.png", PngBuilder::new().text("Title", "baz").build())
    .write(
      "metadata.yaml",
      "
        media:
          type: image
          items:
            - path: foo.png
            - path: bar.png
      ",
    )
    .args(["create", "--deny", "item-title-missing"])
    .stderr(
      "
        error: media item missing title [item-title-missing]
               └─ `foo.png`
        error: 1 lint error
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
    .args(["create", "--deny", "junk"])
    .stderr(
      "
        error: possible junk file [junk]
               └─ `.DS_Store`
        error: 1 lint error
      ",
    )
    .failure();
}

#[test]
fn deny_media_items_missing() {
  Test::new()
    .write(
      "metadata.yaml",
      "
        media:
          type: audio
          items: []
      ",
    )
    .args(["create", "--deny", "media-items-missing"])
    .stderr(
      "
        error: metadata media missing items [media-items-missing]
        error: 1 lint error
      ",
    )
    .failure();

  Test::new()
    .unstable()
    .write(
      "metadata.yaml",
      "
        media:
          type: web
      ",
    )
    .touch("static/index.html")
    .args(["create", "--deny", "media-items-missing"])
    .success();
}

#[test]
fn deny_media_missing() {
  Test::new()
    .write("metadata.yaml", "title: foo")
    .args(["create", "--deny", "media-missing"])
    .stderr(
      "
        error: metadata missing media [media-missing]
        error: 1 lint error
      ",
    )
    .failure();
}

#[test]
fn deny_metadata_missing() {
  Test::new()
    .args(["create", "--deny", "metadata-missing"])
    .stderr(
      "
        error: package missing metadata [metadata-missing]
        error: 1 lint error
      ",
    )
    .failure();
}

#[test]
fn deny_mixed_formats() {
  let mut jpeg = Cursor::new(Vec::new());
  gradient(1, 1)
    .write_to(&mut jpeg, ImageFormat::Jpeg)
    .unwrap();

  Test::new()
    .write("foo.png", PngBuilder::new().build())
    .write("bar.jpg", jpeg.into_inner())
    .write(
      "metadata.yaml",
      "
        media:
          type: image
          items:
            - path: foo.png
            - path: bar.jpg
      ",
    )
    .args(["create", "--deny", "mixed-formats"])
    .stderr(
      "
        error: media items have mixed formats: PNG, JPEG [mixed-formats]
        error: 1 lint error
      ",
    )
    .failure();
}

#[test]
fn deny_multiple() {
  if cfg!(windows) {
    return;
  }

  Test::new()
    .touch(".DS_Store")
    .touch("aux")
    .args([
      "create",
      "--deny",
      "junk",
      "--deny",
      "windows-reserved-filename",
    ])
    .stderr(
      "
        error: possible junk file [junk]
               └─ `.DS_Store`
        error: Windows does not allow files named `aux` [windows-reserved-filename]
               └─ `aux`
        error: 2 lint errors
      ",
    )
    .failure();
}

#[test]
fn deny_package_creator_missing() {
  Test::new()
    .write("metadata.yaml", "package: {}")
    .args(["create", "--deny", "package-creator-missing"])
    .stderr(
      "
        error: metadata package missing creator [package-creator-missing]
        error: 1 lint error
      ",
    )
    .failure();
}

#[test]
fn deny_package_missing() {
  Test::new()
    .write("metadata.yaml", "title: foo")
    .args(["create", "--deny", "package-missing"])
    .stderr(
      "
        error: metadata missing package [package-missing]
        error: 1 lint error
      ",
    )
    .failure();
}

#[test]
fn deny_time_missing() {
  Test::new()
    .write("metadata.yaml", "title: foo")
    .args(["create", "--deny", "time-missing"])
    .stderr(
      "
        error: metadata missing time [time-missing]
        error: 1 lint error
      ",
    )
    .failure();
}

#[test]
fn deny_title_missing() {
  Test::new()
    .write("metadata.yaml", "creator: foo")
    .args(["create", "--deny", "title-missing"])
    .stderr(
      "
        error: metadata missing title [title-missing]
        error: 1 lint error
      ",
    )
    .failure();
}

#[test]
fn deny_unknown_lint() {
  Test::new()
    .args(["create", "--deny", "foo"])
    .stderr(
      "
        error: invalid value 'foo' for '--deny <LINT>': unknown lint or lint group `foo`

        For more information, try '--help'.
      ",
    )
    .status(2);
}

#[test]
fn deny_video_placeholder_dimension_mismatch() {
  let mut placeholder = Cursor::new(Vec::new());
  gradient(1, 1)
    .write_to(&mut placeholder, ImageFormat::Png)
    .unwrap();

  Test::new()
    .write("foo.mp4", Mp4Builder::new().video_track(2, 1).build())
    .write("bar.png", placeholder.into_inner())
    .write(
      "metadata.yaml",
      "
        media:
          type: video
          items:
            - path: foo.mp4
              placeholder: bar.png
      ",
    )
    .args(["create", "--deny", "video-placeholder-dimension-mismatch"])
    .stderr(
      "
        error: placeholder image is 1×1 but video is 2×1 [video-placeholder-dimension-mismatch]
               └─ `foo.mp4`
        error: 1 lint error
      ",
    )
    .failure();
}

#[test]
fn deny_video_placeholder_missing() {
  Test::new()
    .write("foo.mp4", Mp4Builder::new().video_track(2, 1).build())
    .write(
      "metadata.yaml",
      "
        media:
          type: video
          items:
            - path: foo.mp4
      ",
    )
    .args(["create", "--deny", "video-placeholder-missing"])
    .stderr(
      "
        error: video missing placeholder image [video-placeholder-missing]
               └─ `foo.mp4`
        error: 1 lint error
      ",
    )
    .failure();
}

#[test]
fn deny_windows_reserved_filename() {
  if cfg!(windows) {
    return;
  }

  Test::new()
    .touch("aux")
    .args(["create", "--deny", "windows-reserved-filename"])
    .stderr(
      "
        error: Windows does not allow files named `aux` [windows-reserved-filename]
               └─ `aux`
        error: 1 lint error
      ",
    )
    .failure();
}

#[test]
fn yaml_lints_fail_before_load() {
  let test = Test::new()
    .touch(".DS_Store")
    .write("foo.png", PngBuilder::new().build())
    .write("metadata.yaml", "artwork: foo.png")
    .args([
      "create",
      "--generate",
      "--deny",
      "title-missing",
      "--deny",
      "junk",
    ])
    .stderr(
      "
        error: metadata missing title [title-missing]
        error: 1 lint error
      ",
    )
    .failure();

  assert!(!test.path().join("thumbnails").exists());
}
