use super::*;

#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct State {
  pub servers: BTreeMap<String, ServerState>,
}

impl State {
  pub(crate) const DIR: &'static str = ".filepack";
  const FILENAME: &'static str = "state.json";

  pub fn load(root: &Utf8Path) -> Result<Self> {
    let path = Self::path(root);

    let Some(json) = filesystem::read_to_string_opt(&path)? else {
      return Ok(Self::default());
    };

    serde_json::from_str(&json).context(error::DeserializeState { path })
  }

  fn path(root: &Utf8Path) -> Utf8PathBuf {
    root.join(Self::DIR).join(Self::FILENAME)
  }

  pub(crate) fn save(&self, root: &Utf8Path) -> Result {
    let dir = root.join(Self::DIR);

    filesystem::create_dir_all(&dir)?;

    let path = Self::path(root);

    let mut json = serde_json::to_string_pretty(self).unwrap();

    json.push('\n');

    let mut tempfile = tempfile::Builder::new()
      .tempfile_in(&dir)
      .context(error::FilesystemIo { path: &dir })?;

    tempfile
      .write_all(json.as_bytes())
      .context(error::FilesystemIo { path: &dir })?;

    tempfile
      .persist(&path)
      .map_err(|error| error.error)
      .context(error::FilesystemIo { path: &path })?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn round_trip() {
    let (_tempdir, root) = tempdir();

    assert_eq!(State::load(&root).unwrap(), State::default());

    let state = State {
      servers: BTreeMap::from([(
        "http://example.com/".into(),
        ServerState {
          number: 1,
          revision: test::REVISION.parse().unwrap(),
        },
      )]),
    };

    state.save(&root).unwrap();

    assert_eq!(
      fs::read_to_string(root.join(".filepack/state.json")).unwrap(),
      format!(
        r#"{{
  "servers": {{
    "http://example.com/": {{
      "number": 1,
      "revision": "{}"
    }}
  }}
}}
"#,
        test::REVISION,
      ),
    );

    assert_eq!(State::load(&root).unwrap(), state);
  }
}
