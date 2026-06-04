use {
  super::*,
  redb::{Database, ReadableDatabase, ReadableTable, TableDefinition},
};

const DIRECTORIES: TableDefinition<Hash, ()> = TableDefinition::new("directories");
const METADATA: TableDefinition<DatabaseMetadata, u64> = TableDefinition::new("metadata");
const PACKAGES: TableDefinition<Fingerprint, ()> = TableDefinition::new("packages");
const SCHEMA_VERSION: u64 = 1;

pub(crate) struct Server {
  database: Database,
  files: Utf8PathBuf,
  incoming: Utf8PathBuf,
}

impl Server {
  pub(crate) fn artwork(&self, fingerprint: Fingerprint) -> ServerResult<Resource> {
    let artwork = self
      .package_metadata(fingerprint)?
      .and_then(|metadata| metadata.artwork)
      .context(server_error::ArtworkNotFound { fingerprint })?;

    let hash = self.verified_package_file(fingerprint, &artwork.as_path())?;

    Ok(self.open_file(hash)?.ty(artwork.resource_type()))
  }

  pub(crate) fn directory(&self, hash: Hash) -> ServerResult<Directory> {
    ensure!(
      self
        .database
        .begin_read()?
        .open_table(DIRECTORIES)?
        .get(&hash)?
        .is_some(),
      server_error::DirectoryNotFound { hash },
    );

    self.read_directory(hash)
  }

  fn file_path(&self, hash: Hash) -> Utf8PathBuf {
    self.files.join(hash.to_string())
  }

  pub(crate) fn files(&self) -> ServerResult<Vec<Hash>> {
    let context = server_error::FilesystemIo { path: &self.files };

    let mut files = Vec::new();

    for entry in fs::read_dir(&self.files).context(context)? {
      let entry = entry.context(context)?;

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

  pub(crate) fn media_audio_track(
    &self,
    fingerprint: Fingerprint,
    track: usize,
  ) -> ServerResult<Resource> {
    let metadata = self
      .package_metadata(fingerprint)?
      .context(server_error::PackageMetadataNotFound { fingerprint })?;

    let media = metadata
      .media
      .context(server_error::PackageMediaMetadataNotFound { fingerprint })?;

    match media {
      Media::Audio { tracks } => {
        let track = tracks
          .get(track)
          .context(server_error::MediaAudioTrackDoesNotExist {
            fingerprint,
            track,
            tracks: tracks.len(),
          })?;

        let path = track.as_path();

        let hash = self.verified_package_file(fingerprint, &path)?;

        Ok(self.open_file(hash)?.ty(track.resource_type()))
      }
    }
  }

  fn metadata(&self, fingerprint: Fingerprint) -> ServerResult<Option<Metadata>> {
    self
      .metadata_cbor(fingerprint)?
      .map(|metadata| Metadata::decode_from_slice(&metadata))
      .transpose()
      .context(server_error::PackageMetadataCorrupt { fingerprint })
  }

  fn metadata_cbor(&self, fingerprint: Fingerprint) -> ServerResult<Option<Vec<u8>>> {
    let directory = self.read_directory(fingerprint.into())?;

    let Some(entry) = directory.entries.get(Metadata::CBOR_FILENAME) else {
      return Ok(None);
    };

    let path = self.file_path(entry.hash);

    fs::read(&path)
      .map_err(|err| {
        if err.kind() == io::ErrorKind::NotFound {
          server_error::FileNotFound { hash: entry.hash }.into_error(err)
        } else {
          server_error::FilesystemIo { path }.into_error(err)
        }
      })
      .map(Some)
  }

  pub(crate) fn missing(&self, hashes: &[Hash]) -> ServerResult<BTreeSet<Hash>> {
    let mut missing = BTreeSet::new();

    for &hash in hashes {
      let path = self.file_path(hash);

      if !path
        .try_exists()
        .context(server_error::FilesystemIo { path: &path })?
      {
        missing.insert(hash);
      }
    }

    Ok(missing)
  }

  pub(crate) fn open_file(&self, hash: Hash) -> ServerResult<Resource> {
    let path = self.file_path(hash);

    let file = fs::File::open(&path).map_err(|err| {
      if err.kind() == io::ErrorKind::NotFound {
        server_error::FileNotFound { hash }.into_error(err)
      } else {
        server_error::FilesystemIo { path: &path }.into_error(err)
      }
    })?;

    let content_length = file
      .metadata()
      .context(server_error::FilesystemIo { path })?
      .len();

    Ok(Resource {
      content_length,
      file,
      hash,
      range: None,
      ty: ResourceType::Binary,
    })
  }

  pub(crate) fn package_metadata(
    &self,
    fingerprint: Fingerprint,
  ) -> ServerResult<Option<Metadata>> {
    ensure!(
      self
        .database
        .begin_read()?
        .open_table(PACKAGES)?
        .get(&fingerprint)?
        .is_some(),
      server_error::PackageNotFound { fingerprint },
    );

    self.metadata(fingerprint)
  }

  pub(crate) fn packages(&self) -> ServerResult<Vec<(Fingerprint, Option<ComponentBuf>)>> {
    self
      .database
      .begin_read()?
      .open_table(PACKAGES)?
      .iter()?
      .map(|entry| {
        let fingerprint = entry?.0.value();
        Ok((
          fingerprint,
          self
            .metadata(fingerprint)?
            .and_then(|metadata| metadata.title),
        ))
      })
      .collect()
  }

  fn read_directory(&self, hash: Hash) -> ServerResult<Directory> {
    let path = self.file_path(hash);

    let cbor = fs::read(&path).map_err(|err| {
      if err.kind() == io::ErrorKind::NotFound {
        server_error::FileNotFound { hash }.into_error(err)
      } else {
        server_error::FilesystemIo { path }.into_error(err)
      }
    })?;

    Directory::decode_from_slice(&cbor).context(server_error::DirectoryDecode { hash })
  }

  fn resolve_path(&self, root: Fingerprint, path: &RelativePath) -> ServerResult<Option<Hash>> {
    let mut components = path.components().peekable();

    let mut directory = self.read_directory(root.into())?;
    while let Some(component) = components.next() {
      let Some(entry) = directory.entries.get(component) else {
        return Ok(None);
      };

      if components.peek().is_none() {
        return Ok((entry.ty == EntryType::File).then_some(entry.hash));
      }

      if entry.ty != EntryType::Directory {
        return Ok(None);
      }

      directory = self.read_directory(entry.hash)?;
    }

    Ok(None)
  }

  fn verified_package_file(
    &self,
    fingerprint: Fingerprint,
    path: &RelativePath,
  ) -> ServerResult<Hash> {
    self
      .resolve_path(fingerprint, path)?
      .context(server_error::PackageFileMissing { fingerprint, path })
  }

  pub(crate) fn verify_directory(&self, hash: Hash) -> ServerResult {
    let directory = self.read_directory(hash)?;

    for entry in directory.entries.values() {
      if entry.ty == EntryType::File {
        let path = self.file_path(entry.hash);
        ensure!(
          path
            .try_exists()
            .context(server_error::FilesystemIo { path: &path })?,
          server_error::DirectoryFileMissing {
            directory: hash,
            file: entry.hash,
          },
        );
      }
    }

    let tx = self.database.begin_write()?;

    {
      let mut directories = tx.open_table(DIRECTORIES)?;

      for entry in directory.entries.values() {
        if entry.ty == EntryType::Directory {
          ensure!(
            directories.get(&entry.hash)?.is_some(),
            server_error::DirectoryUnverified {
              directory: hash,
              subdirectory: entry.hash,
            },
          );
        }
      }

      directories.insert(&hash, &())?;
    }

    tx.commit()?;

    Ok(())
  }

  pub(crate) fn verify_package(&self, fingerprint: Fingerprint) -> ServerResult {
    ensure!(
      self
        .database
        .begin_read()?
        .open_table(DIRECTORIES)?
        .get(&fingerprint.into())?
        .is_some(),
      server_error::PackageRootUnverified { fingerprint },
    );

    if let Some(metadata) = self.metadata_cbor(fingerprint)? {
      let metadata = Metadata::decode_from_slice(&metadata)
        .context(server_error::PackageMetadataDecode { fingerprint })?;

      for path in metadata.files() {
        ensure!(
          self.resolve_path(fingerprint, &path)?.is_some(),
          server_error::PackageMetadataFileMissing { fingerprint, path },
        );
      }
    }

    let tx = self.database.begin_write()?;

    tx.open_table(PACKAGES)?.insert(&fingerprint, &())?;

    tx.commit()?;

    Ok(())
  }

  pub(crate) fn with_data_dir(data_dir: &Utf8Path) -> Result<Self> {
    let path = data_dir.join("database.redb");
    let database = Database::create(&path).context(error::DatabaseOpen { path })?;

    let tx = database.begin_write()?;

    if tx.list_tables()?.count() == 0 && tx.list_multimap_tables()?.count() == 0 {
      {
        tx.open_table(METADATA)?
          .insert(DatabaseMetadata::Schema, &SCHEMA_VERSION)?;

        tx.open_table(DIRECTORIES)?;
        tx.open_table(PACKAGES)?;
      }

      tx.commit()?;
    } else {
      let actual = tx
        .open_table(METADATA)?
        .get(DatabaseMetadata::Schema)?
        .context(error::DatabaseSchemaVersionMissing)?
        .value();

      ensure!(
        actual == SCHEMA_VERSION,
        error::DatabaseSchemaVersionMismatch {
          actual,
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn database_schema_version_mismatch() {
    let (_tempdir, data_dir) = tempdir();

    {
      let database = Database::create(data_dir.join("database.redb")).unwrap();
      let tx = database.begin_write().unwrap();
      tx.open_table(METADATA)
        .unwrap()
        .insert(DatabaseMetadata::Schema, &SCHEMA_VERSION + 1)
        .unwrap();
      tx.commit().unwrap();
    }

    assert_matches!(
      Server::with_data_dir(&data_dir).map(drop),
      Err(Error::DatabaseSchemaVersionMismatch {
        actual,
        backtrace: _,
        expected: SCHEMA_VERSION,
      }) if actual == SCHEMA_VERSION + 1,
    );
  }

  #[test]
  fn database_schema_version_missing() {
    let (_tempdir, data_dir) = tempdir();

    {
      let database = Database::create(data_dir.join("database.redb")).unwrap();
      let tx = database.begin_write().unwrap();
      tx.open_table(DIRECTORIES).unwrap();
      tx.commit().unwrap();
    }

    assert_matches!(
      Server::with_data_dir(&data_dir).map(drop),
      Err(Error::DatabaseSchemaVersionMissing { backtrace: _ }),
    );
  }
}
