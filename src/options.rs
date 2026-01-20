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
  #[arg(long, help = "Suppress output")]
  pub(crate) quiet: bool,
}

impl Options {
  pub(crate) fn data_dir(&self) -> Result<Utf8PathBuf> {
    if let Some(path) = &self.data_dir {
      Ok(path.into())
    } else {
      let dir = if let Some(dir) = env::var_os("XDG_DATA_HOME")
        && !dir.is_empty()
      {
        dir.into()
      } else {
        dirs::data_local_dir().context(error::DataLocalDir)?
      };

      Ok(decode_path(&dir)?.join("filepack"))
    }
  }

  pub(crate) fn hash_file(&self, path: &Utf8Path) -> io::Result<File> {
    let mut hasher = Hasher::new();

    if self.parallel {
      hasher.update_mmap_rayon(path)?;
    } else if self.mmap {
      hasher.update_mmap(path)?;
    } else {
      hasher.update_reader(fs::File::open(path)?)?;
    }

    Ok(File {
      hash: hasher.finalize().into(),
      size: hasher.count(),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn data_dir_default() {
    assert_eq!(
      Options {
        data_dir: None,
        mmap: false,
        parallel: false,
        quiet: false,
      }
      .data_dir()
      .unwrap(),
      dirs::data_local_dir().unwrap().join("filepack")
    );
  }

  #[test]
  fn data_dir_set() {
    assert_eq!(
      Options {
        data_dir: Some("hello".into()),
        mmap: false,
        parallel: false,
        quiet: false,
      }
      .data_dir()
      .unwrap(),
      Utf8Path::new("hello"),
    );
  }
}
