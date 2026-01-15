use super::*;

#[derive(Parser)]
pub(crate) struct Key {
  #[arg(default_value_t = KeyName::DEFAULT, help = "Print public key <KEY>", long)]
  key: KeyName,
}

impl Key {
  pub(crate) fn run(self, options: Options) -> Result {
    let keychain = Keychain::load(&options)?;
    println!("{}", keychain.public_key(&self.key)?);
    Ok(())
  }
}
