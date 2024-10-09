use super::*;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct Manifest {
  pub(crate) files: BTreeMap<RelativePath, Entry>,
}

impl Manifest {
  pub(crate) const FILENAME: &'static str = "filepack.json";
}

#[cfg(test)]
mod tests {
  use {super::*, regex::Regex};

  #[test]
  fn manifests_in_readme_are_valid() {
    let readme = fs::read_to_string("README.md").unwrap();

    let re = Regex::new(r"(?s)```json(.*?)```").unwrap();

    for capture in re.captures_iter(&readme) {
      serde_json::from_str::<Manifest>(&capture[1]).unwrap();
    }
  }
}
