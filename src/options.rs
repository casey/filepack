use super::*;

#[derive(Parser)]
pub(crate) struct Options {
  #[arg(long, help = "Memory-map files for hashing")]
  pub(crate) mmap: bool,
  #[arg(long, help = "Memory-map and read files in parallel for hashing")]
  pub(crate) parallel: bool,
}

impl Options {
  pub(crate) fn hash_file(&self, path: &Utf8Path) -> io::Result<Entry> {
    let mut hasher = Hasher::new();

    if self.parallel {
      hasher.update_mmap_rayon(path)?;
    } else if self.mmap {
      hasher.update_mmap(path)?;
    } else {
      let file = File::open(path)?;
      hasher.update_reader(file)?;
    }

    Ok(Entry {
      hash: hasher.finalize().into(),
      size: hasher.count(),
    })
  }
}
