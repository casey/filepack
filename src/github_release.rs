use super::*;

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
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let v = s.split('/').collect::<Vec<&str>>();
    assert_eq!(v.len(), 3);
    assert!(!v[0].is_empty());
    assert!(!v[1].is_empty());
    assert!(!v[2].is_empty());
    Ok(Self {
      owner: v[0].into(),
      repo: v[1].into(),
      tag: v[2].into(),
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
}
