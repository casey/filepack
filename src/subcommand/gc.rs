use super::*;

#[derive(Parser)]
pub(crate) struct Gc {
  #[arg(help = "Authenticate with key <KEY>", long, value_name = "KEY")]
  auth: Option<KeyName>,
  #[arg(help = "Delete unreferenced data on server at <URL>", long, value_name = "URL", value_parser = CheckedUrl::check)]
  server: Url,
}

impl Gc {
  pub(crate) fn run(self, options: Options) -> Result {
    let response = Client::new(&options, self.server.clone(), self.auth.as_ref())?.gc()?;

    println!(
      "removed {} and {}, freeing {}",
      Count::irregular(response.directories.len(), "directory", "directories"),
      Count::new(response.files.len(), "file"),
      Count::new(response.bytes, "byte"),
    );

    Ok(())
  }
}
