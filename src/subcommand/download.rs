use {
  super::*,
  reqwest::{blocking::Client, header},
};

const USER_AGENT: &str = concat!("filepack/", env!("CARGO_PKG_VERSION"));

fn asset_filename(name: &str) -> Result<RelativePath> {
  name.parse().context(error::ArtifactName { name })
}

fn var(key: &str) -> Result<Option<String>> {
  match env::var(key) {
    Ok(val) => Ok(Some(val)),
    Err(env::VarError::NotPresent) => Ok(None),
    Err(env::VarError::NotUnicode(_)) => Err(error::EnvironmentVariableUnicode { key }.build()),
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

#[derive(Parser)]
pub(crate) struct Download {
  #[arg(long, help = "Download artifacts for <GITHUB_RELEASE>")]
  github_release: GithubRelease,
  #[arg(help = "Download artifacts to <ROOT> directory, defaults to current directory")]
  root: Option<Utf8PathBuf>,
}

impl Download {
  pub(crate) fn run(self) -> Result {
    let root = if let Some(root) = self.root {
      root
    } else {
      current_dir()?
    };

    let GithubRelease { owner, repo, tag } = &self.github_release;

    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases/tags/{tag}");

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

    let response = builder.send()?.error_for_status()?;

    let json = response.text()?;

    let release = serde_json::from_str::<Release>(&json).context(error::DeserializeRelease {
      release: self.github_release.clone(),
    })?;

    let mut assets = BTreeMap::new();

    assets.insert(
      asset_filename(&format!("{repo}-{tag}.zip"))?,
      release.zipball_url,
    );

    assets.insert(
      asset_filename(&format!("{repo}-{tag}.tar.gz"))?,
      release.tarball_url,
    );

    for asset in release.assets {
      let filename = asset_filename(&asset.name)?;
      ensure! {
        assets.insert(filename.clone(), asset.browser_download_url).is_none(),
        error::DuplicateReleaseAssetFilename { filename },
      }
    }

    for (name, url) in &assets {
      eprintln!("Downloading {name}...");

      let mut response = client
        .get(url)
        .header(header::USER_AGENT, USER_AGENT)
        .send()?
        .error_for_status()?;

      let path = root.join(name);

      let mut file = File::create(&path).context(error::Io { path: &path })?;

      io::copy(&mut response, &mut file).context(error::Download { path, url })?;
    }

    Ok(())
  }
}
