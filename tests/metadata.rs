use super::*;

#[test]
fn create_checks_metadata() {
  Test::new()
    .write("metadata.yaml", "title: Foo\nreadme: README.md")
    .arg("create")
    .stderr("error: file referenced in metadata missing: `README.md`\n")
    .failure();
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
      r#"{
  "artwork": {
    "dimensions": {
      "height": 2,
      "width": 2
    },
    "filename": "cover.png",
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
      "\
media:
  type: image
  images:
    - foo.png
",
    )
    .arg("create")
    .success()
    .arg("metadata")
    .stdout(
      r#"{
  "media": {
    "type": "image",
    "images": [
      {
        "dimensions": {
          "height": 1,
          "width": 2
        },
        "filename": "foo.png",
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
      flac(
        &[
          "ALBUM=qux",
          "ARTIST=baz",
          "DISCNUMBER=1",
          "DISCTOTAL=1",
          "TITLE=bar",
          "TRACKNUMBER=1",
          "TRACKTOTAL=1",
        ],
        44100,
      ),
    )
    .write(
      "metadata.yaml",
      "\
media:
  type: audio
  tracks:
    - foo.flac
",
    )
    .arg("create")
    .success()
    .arg("metadata")
    .stdout(
      r#"{
  "media": {
    "type": "audio",
    "tracks": [
      {
        "album": "qux",
        "artist": "baz",
        "channels": 2,
        "disc": 1,
        "discs": 1,
        "filename": "foo.flac",
        "sample_bits": 16,
        "sample_rate": 44100,
        "samples": 44100,
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
        .video_track(2, 1)
        .audio_track(0x40)
        .build(),
    )
    .write(
      "metadata.yaml",
      "\
media:
  type: video
  videos:
    - foo.mp4
",
    )
    .arg("create")
    .success()
    .arg("metadata")
    .stdout(
      r#"{
  "media": {
    "type": "video",
    "videos": [
      {
        "filename": "foo.mp4",
        "tracks": [
          {
            "codec": "h264",
            "info": {
              "type": "video",
              "dimensions": {
                "height": 1,
                "width": 2
              }
            }
          },
          {
            "codec": "aac",
            "info": {
              "type": "audio"
            }
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
fn create_rejects_extra_files_in_media_packages() {
  Test::new()
    .write(
      "foo.flac",
      flac(
        &[
          "ALBUM=qux",
          "ARTIST=baz",
          "DISCNUMBER=1",
          "DISCTOTAL=1",
          "TITLE=bar",
          "TRACKNUMBER=1",
          "TRACKTOTAL=1",
        ],
        44100,
      ),
    )
    .write(
      "metadata.yaml",
      "\
media:
  type: audio
  tracks:
    - foo.flac
",
    )
    .touch("bar.txt")
    .create_dir("empty")
    .arg("create")
    .stderr(
      "\
error: found 2 extra files not referenced in metadata
       ├─ `bar.txt`
       └─ `empty`
",
    )
    .failure();
}

#[test]
fn create_rejects_invalid_track_positions() {
  Test::new()
    .write(
      "foo.flac",
      flac(
        &[
          "ALBUM=qux",
          "ARTIST=baz",
          "DISCNUMBER=1",
          "DISCTOTAL=1",
          "TITLE=bar",
          "TRACKNUMBER=2",
          "TRACKTOTAL=2",
        ],
        44100,
      ),
    )
    .write(
      "metadata.yaml",
      "\
media:
  type: audio
  tracks:
    - foo.flac
",
    )
    .arg("create")
    .stderr(
      "\
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
      "\
media:
  type: audio
  tracks:
    - foo.flac
",
    )
    .arg("create")
    .stderr_regex(
      "error: failed to decode track `.*foo.flac`
       └─ Ill-formed FLAC stream: .*\n",
    )
    .failure();
}

#[test]
fn create_rejects_invalid_videos() {
  Test::new()
    .write("foo.mp4", "barbar")
    .write(
      "metadata.yaml",
      "\
media:
  type: video
  videos:
    - foo.mp4
",
    )
    .arg("create")
    .stderr_regex(
      "error: invalid video `.*foo.mp4`
       ├─ failed to decode MP4
(       ├─ .*\n)*       └─ .*\n",
    )
    .failure();
}

#[test]
fn create_succeeds_with_valid_metadata() {
  Test::new()
    .touch("content")
    .write("cover.png", image(1, 1, ImageFormat::Png))
    .touch("README.md")
    .write(
      "metadata.yaml",
      "\
title: Foo
date: 2024-01-01
language: en
artwork: cover.png
readme: README.md
package:
  readme: README.md
",
    )
    .arg("create")
    .success()
    .arg("verify")
    .stderr("successfully verified 5 files totaling 246 bytes\n")
    .success();
}

#[test]
fn create_uses_existing_metadata_cbor() {
  let test = Test::new()
    .touch("README.md")
    .write("metadata.yaml", "title: Foo\nreadme: README.md")
    .arg("create")
    .success()
    .remove_file("metadata.yaml")
    .args(["create", "--force"])
    .success();

  let cbor = fs::read(test.path().join("metadata.filemeta")).unwrap();

  let manifest = Manifest::load(Some(&test.path().join("manifest.filepack"))).unwrap();

  assert_eq!(
    manifest.embedded,
    BTreeMap::from([(Hash::bytes(&cbor), cbor)]),
  );

  test
    .remove_file("README.md")
    .args(["create", "--force"])
    .stderr("error: file referenced in metadata missing: `README.md`\n")
    .failure();
}

fn flac(comments: &[&str], samples: u32) -> Vec<u8> {
  let mut bytes = b"fLaC".to_vec();

  bytes.push(if comments.is_empty() { 0x80 } else { 0x00 });
  bytes.extend_from_slice(&34u32.to_be_bytes()[1..]);
  bytes.extend_from_slice(&4096u16.to_be_bytes());
  bytes.extend_from_slice(&4096u16.to_be_bytes());
  bytes.extend_from_slice(&[0; 6]);
  bytes.extend_from_slice(&[0x0a, 0xc4, 0x42, 0xf0]);
  bytes.extend_from_slice(&samples.to_be_bytes());
  bytes.extend_from_slice(&[0; 16]);

  if !comments.is_empty() {
    let mut body = Vec::new();
    body.extend_from_slice(&0u32.to_le_bytes());
    body.extend_from_slice(&u32::try_from(comments.len()).unwrap().to_le_bytes());

    for comment in comments {
      body.extend_from_slice(&u32::try_from(comment.len()).unwrap().to_le_bytes());
      body.extend_from_slice(comment.as_bytes());
    }

    bytes.push(0x84);
    bytes.extend_from_slice(&u32::try_from(body.len()).unwrap().to_be_bytes()[1..]);
    bytes.extend(body);
  }

  bytes
}

fn image(width: u32, height: u32, image_format: ImageFormat) -> Vec<u8> {
  let mut buffer = Cursor::new(Vec::new());
  DynamicImage::new_rgb8(width, height)
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
    .stderr_regex(".*successfully verified.*")
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
      r#"{
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
      r#"{
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
      r#"{
  "title": "Foo"
}
"#,
    )
    .success();
}
