use super::*;

#[derive(Parser)]
pub(crate) struct Options {
  #[arg(
    long,
    help = "Store local data, including private keys, in <DATA_DIR>",
    env = "FILEPACK_DATA_DIR"
  )]
  data_dir: Option<Utf8PathBuf>,
  #[arg(long, help = "Memory-map files for hashing")]
  pub(crate) mmap: bool,
  #[arg(long, help = "Memory-map and read files in parallel for hashing")]
  pub(crate) parallel: bool,
}

impl Options {
  pub(crate) fn hash_file(&self, path: &Utf8Path) -> io::Result<Entry> {
    let mut hasher = Hasher::new();

    if self.parallel {
      // rayon is actually slower for files that are under 128KiB
      // (https://docs.rs/blake3/latest/blake3/struct.Hasher.html#method.update_rayon)
      // Check the file size and only use rayon if it's larger than 128KiB.
      let metadata = fs::metadata(path)?;
      if metadata.len() > 128 * 1024 {
        hasher.update_mmap_rayon(path)?;
      } else {
        hasher.update_reader(File::open(path)?)?;
      }
    } else if self.mmap {
      hasher.update_mmap(path)?;
    } else {
      hasher.update_reader(File::open(path)?)?;
    }

    Ok(Entry {
      hash: hasher.finalize().into(),
      size: hasher.count(),
    })
  }

  pub(crate) fn key_dir(&self) -> Result<Utf8PathBuf> {
    let path = if let Some(path) = &self.data_dir {
      path.into()
    } else {
      let path = dirs::data_local_dir().context(error::DataLocalDir)?;
      Utf8Path::from_path(&path)
        .context(error::PathUnicode { path: &path })?
        .join("filepack")
    };

    Ok(path.join("keys"))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn key_dir_default() {
    assert_eq!(
      Options {
        data_dir: None,
        mmap: false,
        parallel: false,
      }
      .key_dir()
      .unwrap(),
      dirs::data_local_dir()
        .unwrap()
        .join("filepack")
        .join("keys"),
    );
  }

  #[test]
  fn key_dir_set() {
    assert_eq!(
      Options {
        data_dir: Some("hello".into()),
        mmap: false,
        parallel: false,
      }
      .key_dir()
      .unwrap(),
      Utf8Path::new("hello").join("keys"),
    );
  }
}
