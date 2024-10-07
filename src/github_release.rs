use super::*;

const ERROR: &str = "must be of the form 'OWNER/REPO/TAG'";

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GithubRelease {
  pub(crate) owner: String,
  pub(crate) repo: String,
  pub(crate) tag: String,
}

impl Display for GithubRelease {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}/{}/{}", self.owner, self.repo, self.tag)
  }
}

impl FromStr for GithubRelease {
  type Err = &'static str;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let [owner, repo, tag]: [String; 3] = s
      .split('/')
      .filter_map(|component| (!component.is_empty()).then_some(component.to_string()))
      .collect::<Vec<String>>()
      .try_into()
      .map_err(|_| ERROR)?;
    Ok(Self { owner, repo, tag })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_round_trip() {
    let release = GithubRelease {
      owner: "foo".into(),
      repo: "bar".into(),
      tag: "baz".into(),
    };

    assert_eq!(
      release.to_string().parse::<GithubRelease>().unwrap(),
      release,
    );
  }

  #[test]
  fn err() {
    "".parse::<GithubRelease>().unwrap_err();
    "/".parse::<GithubRelease>().unwrap_err();
    "//".parse::<GithubRelease>().unwrap_err();
    "///".parse::<GithubRelease>().unwrap_err();
    "a//".parse::<GithubRelease>().unwrap_err();
    "/b/".parse::<GithubRelease>().unwrap_err();
    "//c".parse::<GithubRelease>().unwrap_err();
    "a/b/c/d".parse::<GithubRelease>().unwrap_err();
  }
}
