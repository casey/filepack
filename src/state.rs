use super::*;

/// Local package state.
#[derive(Debug, Default, Deserialize, Json, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct State {
  /// State entries, by server URL.
  pub servers: BTreeMap<String, ServerState>,
}

impl State {
  pub(crate) const DIR: &'static str = ".filepack";
  const FILENAME: &'static str = "state.json";

  pub(crate) fn is_published(&self, revision: RevisionHash) -> bool {
    self
      .servers
      .values()
      .any(|server| server.revision == revision)
  }

  pub fn load(root: &Utf8Path) -> Result<Self> {
    let path = root.join_case_sensitive(Self::DIR)?.join(Self::FILENAME);

    let Some(json) = filesystem::read_to_string_opt(&path)? else {
      return Ok(Self::default());
    };

    serde_json::from_str(&json).context(error::DeserializeState { path })
  }

  pub(crate) fn save(&self, root: &Utf8Path) -> Result {
    let dir = root.join_case_sensitive(Self::DIR)?;

    filesystem::create_dir_all(&dir)?;

    let path = dir.join(Self::FILENAME);

    let mut json = serde_json::to_string_pretty(self).unwrap();

    json.push('\n');

    filesystem::write_atomic(&path, json)?;

    Ok(())
  }

  pub(crate) fn server_candidates() -> Vec<CompletionCandidate> {
    Self::load(Utf8Path::new("."))
      .unwrap_or_default()
      .servers
      .into_keys()
      .map(CompletionCandidate::new)
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn is_published() {
    fn state(servers: &[(&str, RevisionHash)]) -> State {
      State {
        servers: servers
          .iter()
          .map(|(server, revision)| {
            (
              (*server).into(),
              ServerState {
                number: 0,
                revision: *revision,
              },
            )
          })
          .collect(),
      }
    }

    let published = RevisionHash::ZERO;
    let unpublished = RevisionHash::EXAMPLE.parse::<RevisionHash>().unwrap();

    assert!(!state(&[]).is_published(published));

    assert!(state(&[("foo", published)]).is_published(published));

    assert!(!state(&[("foo", unpublished)]).is_published(published));

    assert!(state(&[("foo", published), ("bar", unpublished)]).is_published(published));
  }

  #[test]
  fn round_trip() {
    let (_tempdir, root) = tempdir();

    assert_eq!(State::load(&root).unwrap(), State::default());

    let state = State {
      servers: BTreeMap::from([(
        "http://example.com/".into(),
        ServerState {
          number: 1,
          revision: RevisionHash::EXAMPLE.parse().unwrap(),
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
        RevisionHash::EXAMPLE,
      ),
    );

    assert_eq!(State::load(&root).unwrap(), state);
  }
}
