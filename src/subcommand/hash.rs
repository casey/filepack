use super::*;

#[derive(Parser)]
pub(crate) struct Hash {
  #[arg(help = "Hash <FILE>, defaulting to standard input")]
  file: Option<Utf8PathBuf>,
  #[arg(help = "Assert file hash is <HASH>", long, value_name = "HASH")]
  assert: Option<crate::Hash>,
}

impl Hash {
  pub(crate) fn run(self, options: Options) -> Result {
    let hash = if let Some(path) = self.file {
      options
        .hash_file(&path)
        .context(error::FilesystemIo { path })?
        .hash
    } else {
      let mut hasher = Hasher::new();

      hasher
        .update_reader(io::stdin())
        .context(error::StandardInputIo)?;

      hasher.finalize().into()
    };

    if let Some(assert) = self.assert {
      ensure! {
        hash == assert,
        error::Assert {
          actual: hash,
          expected: assert,
        }
      }
    }

    println!("{hash}");

    Ok(())
  }
}
