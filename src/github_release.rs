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
    let components = s
      .split('/')
      .filter(|component| !component.is_empty())
      .collect::<Vec<&str>>();

    let [owner, repo, tag] = components[..] else {
      return Err(ERROR);
    };

    Ok(Self {
      owner: owner.into(),
      repo: repo.into(),
      tag: tag.into(),
    })
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
