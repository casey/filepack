use super::*;

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct Info {
  data_dir: Utf8PathBuf,
  key_dir: Utf8PathBuf,
  keys: BTreeMap<KeyName, PublicKey>,
}

pub(crate) fn run(options: Options) -> Result {
  let key_dir = options.key_dir()?;
  let keys = Keys::load(&key_dir)?;

  let info = Info {
    data_dir: options.data_dir()?,
    key_dir,
    keys: keys.names,
  };

  println!("{}", serde_json::to_string_pretty(&info).unwrap());

  Ok(())
}
