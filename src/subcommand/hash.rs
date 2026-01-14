use super::*;

#[derive(Parser)]
pub(crate) struct Hash {
  #[arg(help = "Assert file hash is <HASH>", long, value_name = "HASH")]
  assert: Option<crate::Hash>,
  #[arg(help = "Hash <FILE>, defaulting to standard input")]
  file: Option<Utf8PathBuf>,
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

    if let Some(expected) = self.assert {
      ensure! {
        hash == expected,
        error::Assert {
          actual: hash,
          expected: expected,
        }
      }
    }

    println!("{hash}");

    Ok(())
  }
}
