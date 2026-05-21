use {
  super::*,
  redb::{Database, ReadableDatabase, ReadableTable, TableDefinition},
};

const SCHEMA_VERSION: u64 = 0;

const DIRECTORIES: TableDefinition<Hash, ()> = TableDefinition::new("directories");
const METADATA: TableDefinition<u64, u64> = TableDefinition::new("metadata");

#[derive(Copy, Clone)]
pub(crate) enum MetadataKey {
  Schema = 0,
}

impl MetadataKey {
  fn key(self) -> u64 {
    self as u64
  }
}

pub(crate) struct Server {
  database: Database,
  files: Utf8PathBuf,
  incoming: Utf8PathBuf,
}

impl Server {
  fn file_path(&self, hash: Hash) -> Utf8PathBuf {
    self.files.join(hash.to_string())
  }

  pub(crate) async fn files(&self) -> ServerResult<Vec<Hash>> {
    let context = server_error::FilesystemIo { path: &self.files };

    let mut entries = tokio::fs::read_dir(&self.files).await.context(context)?;

    let mut files = Vec::new();

    while let Some(entry) = entries.next_entry().await.context(context)? {
      let Ok(name) = entry.file_name().into_string() else {
        continue;
      };

      if let Ok(hash) = name.parse() {
        files.push(hash);
      }
    }

    files.sort();

    Ok(files)
  }

  pub(crate) async fn open_file(&self, hash: Hash) -> ServerResult<(tokio::fs::File, u64)> {
    let path = self.file_path(hash);

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

  pub(crate) async fn verify_directory(&self, hash: Hash) -> ServerResult {
    let path = self.file_path(hash);

    let cbor = match tokio::fs::read(&path).await {
      Err(err) => {
        return if err.kind() == io::ErrorKind::NotFound {
          Err(server_error::FileNotFound { hash }.into_error(err))
        } else {
          Err(server_error::FilesystemIo { path }.into_error(err))
        };
      }
      Ok(cbor) => cbor,
    };

    let directory =
      Directory::decode_from_slice(&cbor).context(server_error::DirectoryDecode { hash })?;

    {
      let tx = self.database.begin_read()?;

      let directories = tx.open_table(DIRECTORIES)?;

      for entry in directory.entries.values() {
        match entry.ty {
          EntryType::File => {
            let file_path = self.file_path(entry.hash);
            let exists = tokio::fs::try_exists(&file_path)
              .await
              .context(server_error::FilesystemIo { path: &file_path })?;
            ensure!(
              exists,
              server_error::DirectoryFileMissing {
                directory: hash,
                file: entry.hash,
              },
            );
          }
          EntryType::Directory => {
            let verified = directories.get(&entry.hash)?.is_some();
            ensure!(
              verified,
              server_error::DirectorySubdirectoryUnverified {
                directory: hash,
                subdirectory: entry.hash,
              },
            );
          }
        }
      }
    }

    let tx = self.database.begin_write()?;

    {
      let mut directories = tx.open_table(DIRECTORIES)?;

      directories.insert(&hash, &())?;
    }

    tx.commit()?;

    Ok(())
  }

  pub(crate) fn with_data_dir(data_dir: &Utf8Path) -> Result<Self> {
    let database = Database::create(data_dir.join("database.redb"))?;

    let tx = database.begin_write()?;

    if tx.list_tables()?.count() == 0 {
      {
        let mut metadata = tx.open_table(METADATA)?;

        metadata.insert(&MetadataKey::Schema.key(), &SCHEMA_VERSION)?;

        tx.open_table(DIRECTORIES)?;
      }

      tx.commit()?;
    } else {
      let schema_version = tx
        .open_table(METADATA)?
        .get(&MetadataKey::Schema.key())?
        .context(error::DatabaseSchemaVersionMissing)?
        .value();

      ensure!(
        schema_version == SCHEMA_VERSION,
        error::DatabaseSchemaVersionMismatch {
          actual: schema_version,
          expected: SCHEMA_VERSION,
        },
      );

      drop(tx);
    }

    let files = data_dir.join("files");
    filesystem::create_dir_all(&files)?;

    let incoming = data_dir.join("incoming");
    filesystem::create_dir_all(&incoming)?;

    Ok(Self {
      database,
      files,
      incoming,
    })
  }

  pub(crate) async fn write_file(&self, hash: Hash, body: Body) -> ServerResult {
    let (file, temp_path) = transfer_tempfile(hash, &self.incoming)
      .context(server_error::FilesystemIo {
        path: &self.incoming,
      })?
      .into_parts();

    let temp_path_utf8 = Utf8Path::from_path(&temp_path).unwrap().to_owned();

    let mut writer = tokio::io::BufWriter::new(tokio::fs::File::from_std(file));

    let mut hasher = Hasher::new();

    let mut stream = body.into_data_stream();

    while let Some(chunk) = stream.next().await {
      let chunk = chunk.context(server_error::UploadBodyRead { hash })?;

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

    let path = self.file_path(hash);

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
