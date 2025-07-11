use super::*;

use std::{
  fs::{self, File},
  io::{BufWriter, Write},
};

#[derive(Parser)]
pub(crate) struct Archive {}

const MAGIC_BYTES: &[u8] = b"FILEPACK";

impl Archive {
  pub(crate) fn run(self, _options: Options) -> Result {
    let (manifest_path, manifest_json, manifest) = Manifest::load(None)?;

    let mut files = Vec::new();

    for path in manifest.files.keys() {
      let content = fs::read(path).context(error::Io { path })?;
      files.push((path, content));
    }

    let archive_path = "archive.filepack";

    let archive = File::create("archive.filepack").context(error::Io { path: archive_path })?;

    let mut writer = BufWriter::new(archive);

    writer
      .write_all(MAGIC_BYTES)
      .context(error::Io { path: archive_path })?;

    let mut offset: u64 = 0;

    let manifest_hash = manifest.fingerprint();

    writer
      .write_all(manifest_hash.as_bytes())
      .context(error::Io { path: archive_path })?;

    writer
      .write_all(&offset.to_le_bytes())
      .context(error::Io { path: archive_path })?;

    let manifest_size = u64::try_from(manifest_json.len()).unwrap();

    writer
      .write_all(&manifest_size.to_le_bytes())
      .context(error::Io { path: archive_path })?;

    offset += manifest_size;

    for entry in manifest.files.values() {
      writer
        .write_all(entry.hash.as_bytes())
        .context(error::Io { path: archive_path })?;

      writer
        .write_all(&offset.to_le_bytes())
        .context(error::Io { path: archive_path })?;

      writer
        .write_all(&entry.size.to_le_bytes())
        .context(error::Io { path: archive_path })?;

      offset += entry.size;
    }

    writer
      .write_all(&manifest_json.as_bytes())
      .context(error::Io {
        path: &manifest_path,
      })?;

    for (file, content) in files {
      writer
        .write_all(&content)
        .context(error::Io { path: &file })?;
    }

    Ok(())
  }
}
