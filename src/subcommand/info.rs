use super::*;

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct Info {
  data_dir: Utf8PathBuf,
  key_dir: Utf8PathBuf,
  keys: BTreeSet<KeyName>,
}

fn keys(dir: &Utf8Path) -> Result<BTreeSet<KeyName>> {
  let mut keys = BTreeSet::new();

  for entry in WalkDir::new(&dir) {
    let entry = entry?;

    if entry.depth() == 0 {
      continue;
    }

    if !entry.file_type().is_file() {
      panic!();
    }

    let path = decode_path(entry.path())?;

    let Some(extension) = path.extension() else {
      panic!();
    };

    if extension == "private" {
    } else if extension == "public" {
    } else {
      panic!()
    }

    let name = path.file_stem().unwrap().parse::<KeyName>().unwrap();

    keys.insert(name);
  }

  Ok(keys)
}

pub(crate) fn run(options: Options) -> Result {
  let key_dir = options.key_dir()?;
  let keys = keys(&key_dir)?;

  let info = Info {
    data_dir: options.data_dir()?,
    key_dir,
    keys,
  };

  println!("{}", serde_json::to_string_pretty(&info).unwrap());

  Ok(())
}
