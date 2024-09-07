use super::*;

#[derive(Parser)]
pub(crate) struct Hash {
  #[arg(help = "Hash file at <PATH>, defaulting to standard input")]
  path: Option<Utf8PathBuf>,
}

impl Hash {
  pub(crate) fn run(self, options: Options) -> Result {
    let hash = if let Some(path) = self.path {
      options.hash_file(&path)?.hash
    } else {
      let mut hasher = Hasher::new();

      hasher
        .update_reader(io::stdin())
        .context(error::StandardInputIo)?;

      hasher.finalize().into()
    };

    println!("{hash}");

    Ok(())
  }
}
