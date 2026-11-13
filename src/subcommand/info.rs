use super::*;

/// Local configuration.
#[derive(Json, Serialize)]
pub(crate) struct Info {
  /// Path to the data directory.
  data_dir: Utf8PathBuf,
  /// Path to the keychain directory.
  keychain: Utf8PathBuf,
  /// Public keys in the keychain, by name.
  keys: BTreeMap<KeyName, PublicKey>,
  /// Filepack version.
  version: String,
}

pub(crate) type Input = ();

pub(crate) type Output = Info;

pub(crate) fn run(options: Options) -> Result {
  let keychain = Keychain::load(&options)?;

  let info = Output {
    data_dir: options.data_dir()?,
    keychain: keychain.path,
    keys: keychain.keys,
    version: env!("CARGO_PKG_VERSION").into(),
  };

  println!("{}", serde_json::to_string_pretty(&info).unwrap());

  Ok(())
}
