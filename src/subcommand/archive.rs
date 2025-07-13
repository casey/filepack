use super::*;

const MAGIC_BYTES: &[u8] = b"FILEPACK";

// todo:
// - come up with better magic bytes
//   - call it a file signature instead? more self-explanatory
//   - include non text characters
//   - add more entropy
// - assert that content has correct size and hash

pub(crate) fn run() -> Result {
  let (_manifest_path, manifest_json, manifest) = Manifest::load(None)?;

  let mut files = Vec::new();

  files.push((
    Hash::bytes(manifest_json.as_bytes()),
    manifest_json.clone().into(),
  ));

  for (path, entry) in &manifest.files {
    let content = fs::read(path).context(error::Io { path })?;
    files.push((entry.hash, content));
  }

  files.sort_by_key(|(hash, _content)| *hash);

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

  for (hash, file) in &files {
    write(hash.as_bytes())?;
    write(&offset.to_le_bytes())?;
    let size = file.len().into_u64();
    write(&size.to_le_bytes())?;
    offset += size;
  }

  for (_hash, file) in files {
    write(&file)?;
  }

  Ok(())
}
