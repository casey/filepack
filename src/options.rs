use super::*;

#[derive(Parser)]
pub(crate) struct Options {
  #[arg(long, help = "Memory-map files for hashing")]
  pub(crate) mmap: bool,
  #[arg(long, help = "Memory-map and read files in parallel for hashing")]
  pub(crate) parallel: bool,
}

impl Options {
  pub(crate) fn hash_file(&self, path: &Utf8Path) -> Result<Hash> {
    let mut hasher = Hasher::new();

    if self.parallel {
      hasher.update_mmap_rayon(path).context(error::Io { path })?;
    } else if self.mmap {
      hasher.update_mmap(path).context(error::Io { path })?;
    } else {
      let file = File::open(path).context(error::Io { path })?;
      hasher.update_reader(file).context(error::Io { path })?;
    }

    Ok(hasher.finalize().into())
  }
}
