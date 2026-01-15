use super::*;

#[derive(Parser)]
pub(crate) struct Keygen {
  #[arg(default_value_t = KeyName::DEFAULT, help = "Generate key named <NAME>", long)]
  name: KeyName,
}

impl Keygen {
  pub(crate) fn run(self, options: Options) -> Result {
    Keychain::load(&options)?.generate_key(&self.name)
  }
}
