use super::*;

pub(crate) struct Server {
  files: Utf8PathBuf,
  incoming: Utf8PathBuf,
}

impl Server {
  pub(crate) fn new(options: Options) -> Result<Self> {
    let data_dir = options.data_dir()?;

    let files = data_dir.join("files");
    filesystem::create_dir_all(&files)?;

    let incoming = data_dir.join("incoming");
    filesystem::create_dir_all(&incoming)?;

    Ok(Self { files, incoming })
  }

  pub(crate) fn read_file(&self, hash: Hash) -> ServerResult<Vec<u8>> {
    let path = self.files.join(hash.to_string());
    match fs::read(&path) {
      Err(err) => {
        if err.kind() == io::ErrorKind::NotFound {
          Err(server_error::FileNotFound { hash }.into_error(err))
        } else {
          Err(server_error::FilesystemIo { path }.into_error(err))
        }
      }
      Ok(file) => Ok(file),
    }
  }

  pub(crate) fn write_file(&self, hash: Hash, contents: &[u8]) -> ServerResult {
    let actual = Hash::bytes(contents);

    ensure!(
      actual == hash,
      server_error::UploadHashMismatch {
        actual,
        expected: hash,
      },
    );

    let mut tempfile = tempfile::Builder::new()
      .prefix(&format!("{hash}-"))
      .tempfile_in(&self.incoming)
      .context(server_error::FilesystemIo {
        path: &self.incoming,
      })?;

    let tempfile_path = Utf8Path::from_path(tempfile.path()).unwrap().to_owned();

    tempfile
      .write_all(contents)
      .with_context(|_| server_error::FilesystemIo {
        path: &tempfile_path,
      })?;

    tempfile
      .keep()
      .map_err(|error| error.error)
      .with_context(|_| server_error::FilesystemIo {
        path: &tempfile_path,
      })?;

    let path = self.files.join(hash.to_string());

    fs::rename(tempfile_path, &path).context(server_error::FilesystemIo { path: &path })?;

    Ok(())
  }
}
