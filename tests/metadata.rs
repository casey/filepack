use super::*;

#[test]
fn create_allows_extra_files_in_web_packages() {
  Test::new()
    .write(
      "metadata.yaml",
      "
        media:
          type: web
      ",
    )
    .touch("static/index.html")
    .touch("static/bar.txt")
    .create_dir("static/foo")
    .arg("create")
    .success();
}

#[test]
fn create_allows_nested_paths() {
  Test::new()
    .write("foo/bar.md", "baz")
    .write("foo/baz.png", image(2, 1, ImageFormat::Png))
    .write(
      "metadata.yaml",
      "
        readme: foo/bar.md
        media:
          type: image
          items:
            - foo/baz.png
      ",
    )
    .arg("create")
    .success()
    .arg("verify")
    .stderr_regex("successfully verified .*")
    .success();
}

#[test]
fn create_checks_metadata() {
  Test::new()
    .write(
      "metadata.yaml",
      "
        title: Foo
        readme: README.md
      ",
    )
    .arg("create")
    .stderr("error: file referenced in metadata missing: `README.md`\n")
    .failure();
}

#[test]
fn create_does_not_write_metadata_cbor_on_failure() {
  Test::new()
    .write(
      "metadata.yaml",
      "
        media:
          type: web
      ",
    )
    .touch("static/index.html")
    .touch("bar.txt")
    .arg("create")
    .stderr(
      "
        error: found 1 extra file not referenced in metadata
               └─ `bar.txt`
      ",
    )
    .failure()
    .remove_file("bar.txt")
    .arg("create")
    .success();
}

#[test]
fn create_extracts_artwork_dimensions() {
  Test::new()
    .write("cover.png", image(2, 2, ImageFormat::Png))
    .write("metadata.yaml", "artwork: cover.png")
    .arg("create")
    .success()
    .arg("metadata")
    .stdout(
      r#"
        {
          "artwork": {
            "alpha": false,
            "bit_depth": 8,
            "color_type": "rgb",
            "dimensions": {
              "height": 2,
              "width": 2
            },
            "orientation": {
              "mirrored": false,
              "rotation": 0
            },
            "path": "cover.png",
            "type": "png"
          }
        }
      "#,
    )
    .success();
}

#[test]
fn create_extracts_image_dimensions() {
  Test::new()
    .write("foo.png", image(2, 1, ImageFormat::Png))
    .write(
      "metadata.yaml",
      "
        media:
          type: image
          items:
            - foo.png
      ",
    )
    .arg("create")
    .success()
    .arg("metadata")
    .stdout(
      r#"
        {
          "media": {
            "type": "image",
            "items": [
              {
                "alpha": false,
                "bit_depth": 8,
                "color_type": "rgb",
                "dimensions": {
                  "height": 1,
                  "width": 2
                },
                "orientation": {
                  "mirrored": false,
                  "rotation": 0
                },
                "path": "foo.png",
                "type": "png"
              }
            ]
          }
        }
      "#,
    )
    .success()
    .arg("verify")
    .stderr_regex("successfully verified .*")
    .success();
}

#[test]
fn create_extracts_track_tags() {
  Test::new()
    .write(
      "foo.flac",
      FlacBuilder::new()
        .tag("ALBUM", "qux")
        .tag("ARTIST", "baz")
        .tag("DISCNUMBER", "1")
        .tag("DISCTOTAL", "1")
        .tag("TITLE", "bar")
        .tag("TRACKNUMBER", "1")
        .tag("TRACKTOTAL", "1")
        .build(),
    )
    .write(
      "metadata.yaml",
      "
        media:
          type: audio
          items:
            - foo.flac
      ",
    )
    .arg("create")
    .success()
    .arg("metadata")
    .stdout(
      r#"
        {
          "media": {
            "type": "audio",
            "items": [
              {
                "album": "qux",
                "artist": "baz",
                "channels": 2,
                "disc": 1,
                "discs": 1,
                "path": "foo.flac",
                "sample_bits": 16,
                "sample_rate": 44100,
                "samples": 44100,
                "size": 1024,
                "title": "bar",
                "track": 1,
                "tracks": 1,
                "type": "flac"
              }
            ]
          }
        }
      "#,
    )
    .success()
    .arg("verify")
    .stderr_regex("successfully verified .*")
    .success();
}

#[test]
fn create_extracts_video_metadata() {
  Test::new()
    .write(
      "foo.mp4",
      Mp4Builder::new()
        .duration(1500)
        .frame_count(30)
        .video_track(2, 1)
        .audio_track(0x40)
        .build(),
    )
    .write(
      "metadata.yaml",
      "
        media:
          type: video
          items:
            - foo.mp4
      ",
    )
    .arg("create")
    .success()
    .arg("metadata")
    .stdout(
      r#"
        {
          "media": {
            "type": "video",
            "items": [
              {
                "duration": 1500,
                "path": "foo.mp4",
                "tracks": [
                  {
                    "codec": "h264",
                    "info": {
                      "type": "video",
                      "bit_depth": 8,
                      "chroma_subsampling": "4:2:0",
                      "dimensions": {
                        "height": 1,
                        "width": 2
                      },
                      "frames": 30,
                      "orientation": {
                        "mirrored": false,
                        "rotation": 0
                      }
                    },
                    "size": 30
                  },
                  {
                    "codec": "aac",
                    "info": {
                      "type": "audio",
                      "channels": 2,
                      "sample_rate": 44100
                    },
                    "size": 30
                  }
                ],
                "type": "mp4"
              }
            ]
          }
        }
      "#,
    )
    .success()
    .arg("verify")
    .stderr_regex("successfully verified .*")
    .success();
}

#[test]
fn create_generate_force_overwrites_thumbnails() {
  Test::new()
    .write("foo.jpg", image(1280, 640, ImageFormat::Jpeg))
    .write(
      "metadata.yaml",
      "
        media:
          type: image
          items:
            - foo.jpg
      ",
    )
    .write("thumbnails/foo.jpg", "bar")
    .args(["create", "--generate", "--force"])
    .success()
    .arg("verify")
    .stderr_regex("successfully verified .*")
    .success();
}

#[test]
fn create_generate_rejects_existing_thumbnails() {
  Test::new()
    .write("foo.jpg", image(2, 1, ImageFormat::Jpeg))
    .write(
      "metadata.yaml",
      "
        media:
          type: image
          items:
            - foo.jpg
      ",
    )
    .write("thumbnails/foo.jpg", "bar")
    .args(["create", "--generate"])
    .stderr("error: thumbnail for `foo.jpg` conflicts with `thumbnails/foo.jpg`\n")
    .failure();
}

#[test]
fn create_generates_thumbnails() {
  Test::new()
    .write("foo.jpg", image(1280, 640, ImageFormat::Jpeg))
    .write("bar/baz.png", image_alpha(1280, 640, 128, ImageFormat::Png))
    .write(
      "metadata.yaml",
      "
        media:
          type: image
          items:
            - foo.jpg
            - bar/baz.png
      ",
    )
    .args(["create", "--generate"])
    .success()
    .arg("metadata")
    .stdout(
      r#"
        {
          "media": {
            "type": "image",
            "items": [
              {
                "alpha": false,
                "bit_depth": 8,
                "chroma_subsampling": "4:4:4",
                "color_type": "rgb",
                "dimensions": {
                  "height": 640,
                  "width": 1280
                },
                "orientation": {
                  "mirrored": false,
                  "rotation": 0
                },
                "path": "foo.jpg",
                "type": "jpeg"
              },
              {
                "alpha": true,
                "bit_depth": 8,
                "color_type": "rgb",
                "dimensions": {
                  "height": 640,
                  "width": 1280
                },
                "orientation": {
                  "mirrored": false,
                  "rotation": 0
                },
                "path": "bar/baz.png",
                "type": "png"
              }
            ]
          },
          "thumbnails": {
            "bar/baz.png": {
              "alpha": true,
              "bit_depth": 8,
              "color_type": "rgb",
              "dimensions": {
                "height": 512,
                "width": 1024
              },
              "orientation": {
                "mirrored": false,
                "rotation": 0
              },
              "path": "thumbnails/baz.png",
              "type": "png"
            },
            "foo.jpg": {
              "alpha": false,
              "bit_depth": 8,
              "chroma_subsampling": "4:4:4",
              "color_type": "rgb",
              "dimensions": {
                "height": 512,
                "width": 1024
              },
              "orientation": {
                "mirrored": false,
                "rotation": 0
              },
              "path": "thumbnails/foo.jpg",
              "type": "jpeg"
            }
          }
        }
      "#,
    )
    .success()
    .arg("verify")
    .stderr_regex("successfully verified .*")
    .success();
}

#[test]
fn create_rejects_extra_files_in_media_packages() {
  Test::new()
    .write(
      "foo.flac",
      FlacBuilder::new()
        .tag("ALBUM", "qux")
        .tag("ARTIST", "baz")
        .tag("DISCNUMBER", "1")
        .tag("DISCTOTAL", "1")
        .tag("TITLE", "bar")
        .tag("TRACKNUMBER", "1")
        .tag("TRACKTOTAL", "1")
        .build(),
    )
    .write(
      "metadata.yaml",
      "
        media:
          type: audio
          items:
            - foo.flac
      ",
    )
    .touch("bar.txt")
    .create_dir("empty")
    .arg("create")
    .stderr(
      "
        error: found 2 extra files not referenced in metadata
               ├─ `bar.txt`
               └─ `empty`
      ",
    )
    .failure();
}

#[test]
fn create_rejects_extra_files_in_web_packages() {
  Test::new()
    .write(
      "metadata.yaml",
      "
        media:
          type: web
      ",
    )
    .touch("static/index.html")
    .touch("bar.txt")
    .arg("create")
    .stderr(
      "
        error: found 1 extra file not referenced in metadata
               └─ `bar.txt`
      ",
    )
    .failure();
}

#[test]
fn create_rejects_invalid_track_positions() {
  Test::new()
    .write(
      "foo.flac",
      FlacBuilder::new()
        .tag("ALBUM", "qux")
        .tag("ARTIST", "baz")
        .tag("DISCNUMBER", "1")
        .tag("DISCTOTAL", "1")
        .tag("TITLE", "bar")
        .tag("TRACKNUMBER", "2")
        .tag("TRACKTOTAL", "2")
        .build(),
    )
    .write(
      "metadata.yaml",
      "
        media:
          type: audio
          items:
            - foo.flac
      ",
    )
    .arg("create")
    .stderr(
      "
        error: invalid track position
               └─ track `foo.flac` is disc 1 track 2 but expected disc 1 track 1
      ",
    )
    .failure();
}

#[test]
fn create_rejects_invalid_tracks() {
  Test::new()
    .write("foo.flac", "barbar")
    .write(
      "metadata.yaml",
      "
        media:
          type: audio
          items:
            - foo.flac
      ",
    )
    .arg("create")
    .stderr_regex(
      "
        error: invalid audio track `.*foo.flac`
               ├─ failed to decode FLAC
               └─ Ill-formed FLAC stream: .*
      ",
    )
    .failure();
}

#[test]
fn create_rejects_invalid_videos() {
  Test::new()
    .write("foo.mp4", "barbar")
    .write(
      "metadata.yaml",
      "
        media:
          type: video
          items:
            - foo.mp4
      ",
    )
    .arg("create")
    .stderr_regex(
      "
        error: invalid video `.*foo.mp4`
               ├─ failed to decode MP4
               ├─ failed to fill whole buffer
               └─ failed to fill whole buffer
      ",
    )
    .failure();
}

#[test]
fn create_rejects_metadata_cbor_without_yaml() {
  Test::new()
    .touch("README.md")
    .write(
      "metadata.yaml",
      "
        title: Foo
        readme: README.md
      ",
    )
    .arg("create")
    .success()
    .remove_file("metadata.yaml")
    .args(["create", "--force"])
    .stderr_regex("error: metadata `.*metadata.filemeta` already exists\n")
    .failure();
}

#[test]
fn create_requires_index_html_in_web_packages() {
  Test::new()
    .write(
      "metadata.yaml",
      "
        media:
          type: web
      ",
    )
    .arg("create")
    .stderr("error: file referenced in metadata missing: `static/index.html`\n")
    .failure();
}

#[test]
fn create_succeeds_with_valid_metadata() {
  Test::new()
    .touch("content")
    .write("cover.png", image(1, 1, ImageFormat::Png))
    .touch("README.md")
    .touch("COLOPHON.md")
    .write(
      "metadata.yaml",
      "
        title: Foo
        time: 2024-01-01
        language: en
        artwork: cover.png
        readme: README.md
        package:
          colophon: COLOPHON.md
      ",
    )
    .arg("create")
    .success()
    .arg("verify")
    .stderr("successfully verified 6 files totaling 260 bytes\n")
    .success();
}

fn image(width: u32, height: u32, image_format: ImageFormat) -> Vec<u8> {
  let mut buffer = Cursor::new(Vec::new());
  gradient(width, height)
    .write_to(&mut buffer, image_format)
    .unwrap();
  buffer.into_inner()
}

fn image_alpha(width: u32, height: u32, alpha: u8, image_format: ImageFormat) -> Vec<u8> {
  let mut buffer = Cursor::new(Vec::new());
  gradient_alpha(width, height, alpha)
    .write_to(&mut buffer, image_format)
    .unwrap();
  buffer.into_inner()
}

#[test]
fn metadata_cbor_already_exists() {
  Test::new()
    .write("metadata.yaml", "title: Foo")
    .touch("metadata.filemeta")
    .arg("create")
    .stderr_regex("error: metadata `.*metadata.filemeta` already exists\n")
    .failure();
}

#[test]
fn metadata_cbor_force() {
  Test::new()
    .write("metadata.yaml", "title: Foo")
    .touch("metadata.filemeta")
    .args(["create", "--force"])
    .success()
    .arg("verify")
    .stderr_regex("successfully verified .*")
    .success();
}

#[test]
fn metadata_subcommand_default() {
  Test::new()
    .write("metadata.yaml", "title: Foo")
    .arg("create")
    .success()
    .arg("metadata")
    .stdout(
      r#"
        {
          "title": "Foo"
        }
      "#,
    )
    .success();
}

#[test]
fn metadata_subcommand_format_json() {
  Test::new()
    .write("metadata.yaml", "title: Foo")
    .arg("create")
    .success()
    .args(["metadata", "--format", "json"])
    .stdout("{\"title\":\"Foo\"}\n")
    .success();
}

#[test]
fn metadata_subcommand_format_tsv_error() {
  Test::new()
    .write("metadata.yaml", "title: Foo")
    .arg("create")
    .success()
    .args(["metadata", "--format", "tsv"])
    .stderr("error: metadata cannot be formatted as TSV\n")
    .failure();
}

#[test]
fn metadata_subcommand_path_is_directory() {
  Test::new()
    .write("pkg/metadata.yaml", "title: Foo")
    .args(["create", "pkg"])
    .success()
    .args(["metadata", "pkg"])
    .stdout(
      r#"
        {
          "title": "Foo"
        }
      "#,
    )
    .success();
}

#[test]
fn metadata_subcommand_path_is_file() {
  Test::new()
    .write("pkg/metadata.yaml", "title: Foo")
    .args(["create", "pkg"])
    .success()
    .args(["metadata", "pkg/metadata.filemeta"])
    .stdout(
      r#"
        {
          "title": "Foo"
        }
      "#,
    )
    .success();
}
