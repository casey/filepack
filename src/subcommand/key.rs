use super::*;

#[derive(Parser)]
pub(crate) struct Key {
  #[arg(default_value_t = KeyName::DEFAULT, help = "Print public key <KEY>", long)]
  key: KeyName,
}

impl Key {
  pub(crate) fn run(self, options: Options) -> Result {
    let key_dir = options.key_dir()?;

    println!(
      "{}",
      PublicKey::load(&key_dir.join(self.key.public_key_filename()))?,
    );

    Ok(())
  }
}
