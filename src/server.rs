use super::*;

pub(crate) struct Server {
  files: Utf8PathBuf,
}

impl Server {
  pub(crate) fn new(options: Options) -> Result<Self> {
    let files = options.data_dir()?.join("files");

    filesystem::create_dir_all(&files)?;

    Ok(Self { files })
  }

  pub(crate) fn read_file(&self, hash: Hash) -> ServerResult<Vec<u8>> {
    let path = self.files.join(hash.to_string());
    fs::read(&path).context(server_error::FilesystemIo { path })
  }

  pub(crate) fn write_file(&self, hash: Hash, contents: &[u8]) -> ServerResult {
    let path = self.files.join(hash.to_string());

    let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
      Ok(file) => file,
      Err(err) => {
        if err.kind() == io::ErrorKind::AlreadyExists {
          return Ok(());
        }

        return Err(server_error::FilesystemIo { path }.into_error(err));
      }
    };

    file
      .write_all(contents)
      .context(server_error::FilesystemIo { path })
  }
}
