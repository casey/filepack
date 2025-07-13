use super::*;

const MAGIC_BYTES: &[u8] = b"FILEPACK";

pub(crate) fn run() -> Result {
  let (_manifest_path, manifest_json, manifest) = Manifest::load(None)?;

  let mut files = Vec::new();

  for path in manifest.files.keys() {
    let content = fs::read(path).context(error::Io { path })?;
    files.push(content);
  }

  let archive_path = "archive.filepack";

  let archive = File::create("archive.filepack").context(error::Io { path: archive_path })?;

  let mut writer = BufWriter::new(archive);

  let mut write = |data: &[u8]| {
    writer
      .write_all(data)
      .context(error::Io { path: archive_path })
  };

  write(MAGIC_BYTES)?;

  let mut offset: u64 = 0;

  let manifest_hash = manifest.fingerprint();

  write(manifest_hash.as_bytes())?;
  write(&offset.to_le_bytes())?;

  let manifest_size = u64::try_from(manifest_json.len()).unwrap();

  write(&manifest_size.to_le_bytes())?;

  offset += manifest_size;

  for entry in manifest.files.values() {
    write(entry.hash.as_bytes())?;
    write(&offset.to_le_bytes())?;
    write(&entry.size.to_le_bytes())?;
    offset += entry.size;
  }

  write(&manifest_json.as_bytes())?;

  for file in files {
    write(&file)?;
  }

  Ok(())
}
