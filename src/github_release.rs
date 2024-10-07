use super::*;

const ERROR: &str = "must be of the form '<OWNER>/<REPO>/<TAG>'";

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
      .map(|component| {
        if component.is_empty() {
          Err(ERROR)
        } else {
          Ok(component)
        }
      })
      .collect::<Result<Vec<&str>, &str>>()?;

    if components.len() != 3 {
      return Err(ERROR);
    }

    Ok(Self {
      owner: components[0].into(),
      repo: components[1].into(),
      tag: components[2].into(),
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
    "///".parse::<GithubRelease>().unwrap_err();
    "a//".parse::<GithubRelease>().unwrap_err();
    "/b/".parse::<GithubRelease>().unwrap_err();
    "/b/c".parse::<GithubRelease>().unwrap_err();
    "a/b/c/d".parse::<GithubRelease>().unwrap_err();
  }
}
