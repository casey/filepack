use super::*;

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct Output {
  data_dir: Utf8PathBuf,
  key_dir: Utf8PathBuf,
  keys: BTreeMap<KeyName, PublicKey>,
}

pub(crate) fn run(options: Options) -> Result {
  let keychain = Keychain::load(&options)?;

  let info = Output {
    data_dir: options.data_dir()?,
    key_dir: keychain.path,
    keys: keychain.keys,
  };

  println!("{}", serde_json::to_string_pretty(&info).unwrap());

  Ok(())
}
