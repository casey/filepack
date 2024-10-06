use {
  super::*,
  reqwest::{blocking::Client, header},
};

// this shit really is totally separate
// but you don't want to have to download an additional binary

const USER_AGENT: &str = concat!("filepack/", env!("CARGO_PKG_VERSION"));

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GithubRelease {
  owner: String,
  repo: String,
  tag: String,
}

fn var(key: &str) -> Result<Option<String>> {
  match env::var(key) {
    Ok(val) => Ok(Some(val)),
    Err(env::VarError::NotPresent) => Ok(None),
    Err(env::VarError::NotUnicode(_)) => Err(error::EnvVarUnicode { key }.build()),
  }
}

impl GithubRelease {
  pub(crate) fn download(&self, root: &Utf8Path) -> Result {
    let Self { owner, repo, tag } = &self;

    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases/tags/{tag}");

    eprintln!("Downloading {url}");

    let client = Client::new();

    let mut builder = client
      .get(&url)
      .header(header::USER_AGENT, USER_AGENT)
      .header(header::ACCEPT, "application/vnd.github+json")
      .header("x-github-api-version", "2022-11-28");

    for key in ["GH_TOKEN", "GITHUB_TOKEN"] {
      if let Some(token) = var(key)? {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
        break;
      }
    }

    let response = builder.send().context(error::HttpRequest { url: &url })?;

    ensure! {
      response.status().is_success(),
      error::HttpStatus { status: response.status(), url },
    }

    let json = response.text().context(error::HttpRequest { url })?;

    let Release {
      mut assets,
      zipball_url,
      tarball_url,
    } = serde_json::from_str(&json).context(error::GithubReleaseDeserialize {
      release: self.clone(),
    })?;

    assets.push(Asset {
      name: format!("{repo}-{tag}.zip"),
      browser_download_url: zipball_url,
    });

    assets.push(Asset {
      name: format!("{repo}-{tag}.tar.gz"),
      browser_download_url: tarball_url,
    });

    assets.sort_by(|a, b| a.name.cmp(&b.name));

    let mut names = BTreeSet::new();
    for asset in &assets {
      assert!(names.insert(&asset.name));
    }

    for asset in assets {
      eprintln!("{}", asset.name);
      let path = asset.name.parse::<RelativePath>().unwrap();

      let mut response = client
        .get(asset.browser_download_url)
        .header(header::USER_AGENT, USER_AGENT)
        .send()
        .unwrap();

      let mut output_file = File::create(root.join(path)).unwrap();

      std::io::copy(&mut response, &mut output_file).unwrap();
    }

    Ok(())
  }
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
    Ok(Self {
      owner: v[0].into(),
      repo: v[1].into(),
      tag: v[2].into(),
    })
  }
}

#[derive(Deserialize)]
struct Release {
  assets: Vec<Asset>,
  zipball_url: String,
  tarball_url: String,
}

#[derive(Deserialize)]
struct Asset {
  name: String,
  browser_download_url: String,
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
