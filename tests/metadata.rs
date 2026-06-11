use super::*;

const EMPTY_MD5: [u8; 16] = [
  0xd4, 0x1d, 0x8c, 0xd9, 0x8f, 0x00, 0xb2, 0x04, 0xe9, 0x80, 0x09, 0x98, 0xec, 0xf8, 0x42, 0x7e,
];

#[test]
fn create_checks_metadata() {
  Test::new()
    .write("metadata.yaml", "title: Foo\nreadme: README.md")
    .arg("create")
    .stderr("error: file referenced in metadata missing: `README.md`\n")
    .failure();
}

#[test]
fn create_extracts_track_titles() {
  Test::new()
    .write("foo.flac", flac(EMPTY_MD5, &["TITLE=bar"]))
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
        "filename": "foo.flac",
        "title": "bar",
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
      "error: failed to decode FLAC track `.*foo.flac`\n       └─ Ill-formed FLAC stream: .*\n",
    )
    .failure();
}

#[test]
fn create_succeeds_with_valid_metadata() {
  Test::new()
    .touch("content")
    .write("cover.png", image(1, 1, ImageFormat::Png))
    .touch("info.nfo")
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
  nfo: info.nfo
",
    )
    .arg("create")
    .success()
    .arg("verify")
    .stderr("successfully verified 6 files totaling 231 bytes\n")
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

  let cbor = fs::read(test.path().join("metadata.filepack")).unwrap();

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

fn flac(md5sum: [u8; 16], comments: &[&str]) -> Vec<u8> {
  let mut bytes = b"fLaC".to_vec();

  bytes.push(if comments.is_empty() { 0x80 } else { 0x00 });
  bytes.extend_from_slice(&34u32.to_be_bytes()[1..]);
  bytes.extend_from_slice(&4096u16.to_be_bytes());
  bytes.extend_from_slice(&4096u16.to_be_bytes());
  bytes.extend_from_slice(&[0; 6]);
  bytes.extend_from_slice(&[0x0a, 0xc4, 0x42, 0xf0]);
  bytes.extend_from_slice(&[0; 4]);
  bytes.extend_from_slice(&md5sum);

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
    .touch("metadata.filepack")
    .arg("create")
    .stderr_regex("error: metadata `.*metadata.filepack` already exists\n")
    .failure();
}

#[test]
fn metadata_cbor_force() {
  Test::new()
    .write("metadata.yaml", "title: Foo")
    .touch("metadata.filepack")
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
    .args(["metadata", "pkg/metadata.filepack"])
    .stdout(
      r#"{
  "title": "Foo"
}
"#,
    )
    .success();
}
