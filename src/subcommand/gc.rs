use super::*;

#[derive(Parser)]
pub(crate) struct Gc {
  #[arg(help = "Authenticate with key <KEY>", long, value_name = "KEY")]
  auth: Option<KeyName>,
  #[arg(
    help = "Delete unreferenced data on server at <URL>",
    long,
    value_name = "URL"
  )]
  server: ServerUrl,
}

impl Gc {
  pub(crate) fn run(self, options: Options) -> Result {
    let response = Client::new(&options, self.server.clone(), self.auth.as_ref())?.gc()?;

    if !options.quiet {
      eprintln!(
        "removed {}, {}, and {}, freeing {}",
        Count::new(response.revisions.len(), "revision"),
        Count::irregular(response.directories.len(), "directory", "directories"),
        Count::new(response.files.len(), "file"),
        format_size(response.bytes),
      );
    }

    Ok(())
  }
}
