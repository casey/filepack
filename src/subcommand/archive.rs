use super::*;

#[derive(Parser)]
pub(crate) struct Archive {
  #[arg(
    help = "Load manifest from <MANIFEST>. May be path to manifest, to directory containing manifest \
    named `filepack.json`, or omitted, in which case manifest named `filepack.json` in the current \
    directory is loaded."
  )]
  manifest: Option<Utf8PathBuf>,
  #[arg(help = "Write archive to <OUTPUT>.", long)]
  output: Utf8PathBuf,
}

impl Archive {
  pub(crate) fn run(self) -> Result {
    let (path, json, manifest) = Manifest::load(self.manifest.as_deref())?;

    let root = path.parent().unwrap();

    let mut files = Vec::new();

    let manifest_hash = Hash::bytes(json.as_bytes());

    files.push((manifest_hash, json.into()));

    for (path, entry) in &manifest.files {
      let path = root.join(path);
      let content = fs::read(&path).context(error::FilesystemIo { path: &path })?;

      let size = content.len().into_u64();
      if size != entry.size {
        return Err(
          error::SizeMismatch {
            path,
            actual: size,
            expected: entry.size,
          }
          .build(),
        );
      }

      let hash = Hash::bytes(&content);

      if hash != entry.hash {
        return Err(
          error::HashMismatch {
            path,
            actual: hash,
            expected: entry.hash,
          }
          .build(),
        );
      }

      files.push((entry.hash, content));
    }

    files.sort_by_key(|(hash, _content)| *hash);

    let output = File::create(&self.output).context(error::FilesystemIo { path: &self.output })?;

    let mut writer = BufWriter::new(output);

    let mut write = |data: &[u8]| {
      writer
        .write_all(data)
        .context(error::FilesystemIo { path: &self.output })
    };

    write(crate::Archive::FILE_SIGNATURE)?;

    write(manifest_hash.as_bytes())?;
    write(&files.len().into_u64().to_le_bytes())?;

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
}
