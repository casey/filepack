use super::*;

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct Output {
  data: Utf8PathBuf,
  keychain: Utf8PathBuf,
  keys: BTreeMap<KeyName, PublicKey>,
}

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
