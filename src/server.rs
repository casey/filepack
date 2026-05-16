use super::*;

pub(crate) struct Server {
  files: Utf8PathBuf,
  incoming: Utf8PathBuf,
}

impl Server {
  pub(crate) fn new(options: Options) -> Result<Self> {
    Self::with_data_dir(&options.data_dir()?)
  }

  pub(crate) async fn open_file(&self, hash: Hash) -> ServerResult<(tokio::fs::File, u64)> {
    let path = self.files.join(hash.to_string());

    let file = match tokio::fs::File::open(&path).await {
      Err(err) => {
        return if err.kind() == io::ErrorKind::NotFound {
          Err(server_error::FileNotFound { hash }.into_error(err))
        } else {
          Err(server_error::FilesystemIo { path }.into_error(err))
        };
      }
      Ok(file) => file,
    };

    let len = file
      .metadata()
      .await
      .context(server_error::FilesystemIo { path })?
      .len();

    Ok((file, len))
  }

  pub(crate) fn with_data_dir(data_dir: &Utf8Path) -> Result<Self> {
    let files = data_dir.join("files");
    filesystem::create_dir_all(&files)?;

    let incoming = data_dir.join("incoming");
    filesystem::create_dir_all(&incoming)?;

    Ok(Self { files, incoming })
  }

  pub(crate) async fn write_file(&self, hash: Hash, body: Body) -> ServerResult {
    let (file, temp_path) = tempfile::Builder::new()
      .prefix(&format!("{hash}-"))
      .tempfile_in(&self.incoming)
      .context(server_error::FilesystemIo {
        path: &self.incoming,
      })?
      .into_parts();

    let temp_path_utf8 = Utf8Path::from_path(&temp_path).unwrap().to_owned();

    let mut writer = tokio::io::BufWriter::new(tokio::fs::File::from_std(file));

    let mut hasher = Hasher::new();

    let mut stream = body.into_data_stream();

    while let Some(chunk) = stream.next().await {
      let chunk = chunk.context(server_error::UploadBodyRead)?;

      hasher.update(&chunk);

      writer
        .write_all(&chunk)
        .await
        .context(server_error::FilesystemIo {
          path: &temp_path_utf8,
        })?;
    }

    writer.flush().await.context(server_error::FilesystemIo {
      path: &temp_path_utf8,
    })?;

    let actual = Hash::from(hasher.finalize());

    ensure!(
      actual == hash,
      server_error::UploadHashMismatch {
        actual,
        expected: hash,
      },
    );

    let path = self.files.join(hash.to_string());

    if tokio::fs::try_exists(&path)
      .await
      .context(server_error::FilesystemIo { path: &path })?
    {
      return Ok(());
    }

    temp_path
      .persist(&path)
      .map_err(|error| error.error)
      .context(server_error::FilesystemIo { path: &path })?;

    Ok(())
  }
}
