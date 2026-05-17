use {super::*, clap::ArgGroup};

#[derive(Parser)]
#[command(group(
  ArgGroup::new("input")
    .required(true)
    .args(["file", "package"]),
))]
pub(crate) struct Upload {
  #[arg(help = "Upload file at <PATH>", long, value_name = "PATH")]
  file: Option<Utf8PathBuf>,
  #[arg(help = "Upload file at <PATH>", long, value_name = "PATH")]
  package: Option<Utf8PathBuf>,
  #[arg(help = "Upload to server at <URL>", long, value_name = "URL")]
  server: Url,
}

impl Upload {
  pub(crate) fn run(self, options: Options) -> Result {
    match (&self.file, &self.package) {
      (Some(path), None) => self.upload_file(&path, options),
      (None, Some(path)) => self.upload_package(&path, options),
      (None, None) | (Some(_), Some(_)) => unreachable!(),
    }
  }

  pub(crate) fn upload_file(&self, path: &Utf8Path, options: Options) -> Result {
    let file = options
      .hash_file(&path)
      .context(error::FilesystemIo { path })?;

    let url = self
      .server
      .join(&file.hash.to_string())
      .context(error::UrlParse)?;

    let file = filesystem::open(&path)?;

    let response = Client::new()
      .put(url.clone())
      .body(file)
      .send()
      .with_context(|_| error::Request { url: url.clone() })?;

    if !response.status().is_success() {
      return Err(
        error::ResponseStatus {
          status: response.status(),
          url: url.clone(),
          body: response.text().context(error::ResponseBody { url })?,
        }
        .build(),
      );
    }
    Ok(())
  }

  pub(crate) fn upload_package(&self, path: &Utf8Path, options: Options) -> Result {
    todo!()
  }
}
