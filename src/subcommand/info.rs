use super::*;

/// Local configuration.
#[derive(Json, Serialize)]
pub(crate) struct Info {
  /// Path to the data directory.
  data: Utf8PathBuf,
  /// Path to the keychain directory.
  keychain: Utf8PathBuf,
  /// Public keys in the keychain, by name.
  keys: BTreeMap<KeyName, PublicKey>,
}

pub(crate) type Input = ();

pub(crate) type Output = Info;

pub(crate) fn run(options: Options) -> Result {
  let keychain = Keychain::load(&options)?;

  let info = Output {
    data: options.data_dir()?,
    keychain: keychain.path,
    keys: keychain.keys,
  };

  println!("{}", serde_json::to_string_pretty(&info).unwrap());

  Ok(())
}
