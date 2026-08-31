use {
  super::*,
  redb::{Database, ReadOnlyTable, ReadableDatabase, ReadableTable, TableDefinition},
  templates::PackageHtml,
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
  pub(crate) fn artwork(
    &self,
    fingerprint: Fingerprint,
    thumbnail: bool,
  ) -> ServerResult<Resource> {
    let metadata = self.package_metadata(fingerprint)?;

    let artwork = metadata
      .artwork
      .as_ref()
      .context(server_error::ArtworkNotFound { fingerprint })?;

    let image = if thumbnail && let Some(thumbnail) = metadata.thumbnail(&artwork.path) {
      thumbnail
    } else {
      artwork
    };

    let hash = self.verified_package_file(fingerprint, &image.path)?;

    Ok(self.open_file(hash)?.ty(image.resource_type()))
  }

  pub(crate) fn delete_package(&self, fingerprint: Fingerprint) -> ServerResult {
    let tx = self.database.begin_write()?;

    ensure!(
      tx.open_table(PACKAGES)?.remove(&fingerprint)?.is_some(),
      server_error::PackageNotFound { fingerprint },
    );

    tx.commit()?;

    Ok(())
  }

  pub(crate) fn directory(&self, hash: Hash) -> ServerResult<Directory> {
    let tx = self.database.begin_read()?;

    let directories = tx.open_table(DIRECTORIES)?;

    self.directory_ext(&directories, hash)
  }

  pub(crate) fn directory_ext(
    &self,
    directories: &ReadOnlyTable<Hash, ()>,
    hash: Hash,
  ) -> ServerResult<Directory> {
    ensure!(
      directories.get(&hash)?.is_some(),
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

  pub(crate) fn fingerprints(&self) -> ServerResult<BTreeSet<Fingerprint>> {
    let tx = self.database.begin_read()?;

    tx.open_table(PACKAGES)?
      .iter()?
      .map(|entry| Ok(entry?.0.value()))
      .collect()
  }

  pub(crate) fn gc(&self) -> ServerResult<api::gc::Response> {
    let tx = self.database.begin_write()?;

    let mut marked = HashSet::new();

    let mut directories_removed = BTreeSet::new();

    {
      let mut directories = tx.open_table(DIRECTORIES)?;

      let mut stack = tx
        .open_table(PACKAGES)?
        .iter()?
        .map(|entry| Ok(Hash::from(entry?.0.value())))
        .collect::<ServerResult<Vec<Hash>>>()?;

      while let Some(hash) = stack.pop() {
        if !marked.insert(hash) {
          continue;
        }

        let directory = self.read_directory(hash)?;

        for entry in directory.entries.values() {
          match entry {
            Entry::Directory { hash, .. } => stack.push(*hash),
            Entry::File { hash, .. } => {
              marked.insert(*hash);
            }
          }
        }
      }

      for entry in directories.extract_from_if::<Hash, _>(.., |hash, ()| !marked.contains(&hash))? {
        directories_removed.insert(entry?.0.value());
      }
    }

    let mut bytes = 0;

    let mut files_removed = BTreeSet::new();

    for hash in self.files()? {
      if marked.contains(&hash) {
        continue;
      }

      let path = self.file_path(hash);

      bytes += path
        .metadata()
        .context(server_error::FilesystemIo { path: &path })?
        .len();

      files_removed.insert(hash);
    }

    tx.commit()?;

    for &hash in &files_removed {
      let path = self.file_path(hash);
      fs::remove_file(&path).context(server_error::FilesystemIo { path })?;
    }

    Ok(api::gc::Response {
      bytes,
      directories: directories_removed.into(),
      files: files_removed.into(),
    })
  }

  pub(crate) fn media_item(
    &self,
    fingerprint: Fingerprint,
    index: usize,
    ty: MediaType,
    resource: MediaItemResource,
  ) -> ServerResult<Resource> {
    match resource {
      MediaItemResource::Original => {}
      MediaItemResource::Placeholder | MediaItemResource::PlaceholderThumbnail => {
        assert_eq!(ty, MediaType::Video);
      }
      MediaItemResource::Thumbnail => assert_eq!(ty, MediaType::Image),
    }

    let metadata = self.package_metadata(fingerprint)?;

    let media = metadata
      .media
      .as_ref()
      .context(server_error::PackageMediaMetadataNotFound { fingerprint })?;

    ensure! {
      media.ty() == ty,
      server_error::MediaType {
        actual: media.ty(),
        expected: ty,
        fingerprint,
      },
    }

    let item = media
      .item(index)
      .context(server_error::MediaItemDoesNotExist {
        count: media.item_count(),
        fingerprint,
        index,
        ty,
      })?;

    let (path, ty) = match resource {
      MediaItemResource::Original => (item.path(), item.resource_type()),
      MediaItemResource::Placeholder | MediaItemResource::PlaceholderThumbnail => {
        let placeholder = item
          .placeholder()
          .context(server_error::PlaceholderNotFound { fingerprint, index })?;
        let image = if resource == MediaItemResource::PlaceholderThumbnail {
          metadata.thumbnail(&placeholder.path).unwrap_or(placeholder)
        } else {
          placeholder
        };
        (&image.path, image.resource_type())
      }
      MediaItemResource::Thumbnail => match metadata.thumbnail(item.path()) {
        Some(thumbnail) => (&thumbnail.path, thumbnail.resource_type()),
        None => (item.path(), item.resource_type()),
      },
    };

    let hash = self.verified_package_file(fingerprint, path)?;

    Ok(self.open_file(hash)?.ty(ty))
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

    Ok(Some(self.read_file(entry.hash())?))
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
      unsandboxed_content_type: None,
    })
  }

  pub(crate) fn package_file(
    &self,
    fingerprint: Fingerprint,
    path: &RelativePath,
  ) -> ServerResult<Hash> {
    let tx = self.database.begin_read()?;

    let packages = tx.open_table(PACKAGES)?;

    ensure!(
      packages.get(&fingerprint)?.is_some(),
      server_error::PackageNotFound { fingerprint },
    );

    self
      .resolve_path(fingerprint, path)?
      .context(server_error::PackageFileNotFound { fingerprint, path })
  }

  pub(crate) fn package_html(
    &self,
    fingerprint: Fingerprint,
    mounted: bool,
  ) -> ServerResult<PackageHtml> {
    let tx = self.database.begin_read()?;

    let packages = tx.open_table(PACKAGES)?;

    let metadata = self.package_metadata_opt_ext(&packages, fingerprint)?;

    let directories = tx.open_table(DIRECTORIES)?;

    let directory = self.directory_ext(&directories, fingerprint.into())?;

    let totals = directory.totals().unwrap();

    let colophon = if let Some(metadata) = &metadata
      && let Some(package) = &metadata.package
      && let Some(colophon) = &package.colophon
    {
      Some(self.verified_package_file(fingerprint, colophon)?)
    } else {
      None
    };

    let readme = if let Some(metadata) = &metadata
      && let Some(readme) = &metadata.readme
    {
      Some(self.verified_package_file(fingerprint, readme)?)
    } else {
      None
    };

    Ok(PackageHtml {
      colophon,
      directory,
      fingerprint,
      metadata,
      mounted,
      readme,
      totals,
    })
  }

  pub(crate) fn package_metadata(&self, fingerprint: Fingerprint) -> ServerResult<Metadata> {
    self
      .package_metadata_opt(fingerprint)?
      .context(server_error::PackageMetadataNotFound { fingerprint })
  }

  pub(crate) fn package_metadata_opt(
    &self,
    fingerprint: Fingerprint,
  ) -> ServerResult<Option<Metadata>> {
    let tx = self.database.begin_read()?;

    let packages = tx.open_table(PACKAGES)?;

    self.package_metadata_opt_ext(&packages, fingerprint)
  }

  pub(crate) fn package_metadata_opt_ext(
    &self,
    packages: &ReadOnlyTable<Fingerprint, ()>,
    fingerprint: Fingerprint,
  ) -> ServerResult<Option<Metadata>> {
    ensure!(
      packages.get(&fingerprint)?.is_some(),
      server_error::PackageNotFound { fingerprint },
    );

    self.metadata(fingerprint)
  }

  pub(crate) fn packages(
    &self,
    sort: Sort,
    order: Order,
  ) -> ServerResult<Vec<(Fingerprint, Option<Metadata>, Totals)>> {
    let tx = self.database.begin_read()?;

    let directories = tx.open_table(DIRECTORIES)?;

    let mut packages = tx
      .open_table(PACKAGES)?
      .iter()?
      .map(|entry| {
        let fingerprint = entry?.0.value();

        let totals = self
          .directory_ext(&directories, fingerprint.into())?
          .totals()
          .unwrap();

        Ok((fingerprint, self.metadata(fingerprint)?, totals))
      })
      .collect::<ServerResult<Vec<(Fingerprint, Option<Metadata>, Totals)>>>()?;

    packages.sort_by(|a, b| SortKey::compare(a, b, sort, order));

    Ok(packages)
  }

  fn read_directory(&self, hash: Hash) -> ServerResult<Directory> {
    let directory = Directory::decode_from_slice(&self.read_file(hash)?)
      .context(server_error::DirectoryDecode { hash })?;

    Ok(directory)
  }

  fn read_file(&self, hash: Hash) -> ServerResult<Vec<u8>> {
    let path = self.file_path(hash);

    fs::read(&path).map_err(|err| {
      if err.kind() == io::ErrorKind::NotFound {
        server_error::FileNotFound { hash }.into_error(err)
      } else {
        server_error::FilesystemIo { path }.into_error(err)
      }
    })
  }

  fn resolve_path(&self, root: Fingerprint, path: &RelativePath) -> ServerResult<Option<Hash>> {
    let mut components = path.components().peekable();

    let mut directory = self.read_directory(root.into())?;
    while let Some(component) = components.next() {
      let Some(entry) = directory.entries.get(component) else {
        return Ok(None);
      };

      if components.peek().is_none() {
        return Ok((entry.ty() == EntryType::File).then_some(entry.hash()));
      }

      if entry.ty() != EntryType::Directory {
        return Ok(None);
      }

      directory = self.read_directory(entry.hash())?;
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

    directory
      .totals()
      .context(server_error::DirectoryTotals { hash })?;

    let tx = self.database.begin_write()?;

    {
      let mut directories = tx.open_table(DIRECTORIES)?;

      for (name, entry) in &directory.entries {
        let path = self.file_path(entry.hash());

        let metadata = path.metadata().map_err(|error| {
          if error.kind() == io::ErrorKind::NotFound {
            server_error::DirectoryEntryMissing {
              directory: hash,
              hash: entry.hash(),
              name,
              ty: entry.ty(),
            }
            .build()
          } else {
            server_error::FilesystemIo { path: &path }.into_error(error)
          }
        })?;

        ensure! {
          metadata.len() == entry.size(),
          server_error::DirectoryEntrySizeMismatch {
            actual: metadata.len(),
            directory: hash,
            entry: name,
            expected: entry.size(),
          },
        }

        if let Entry::Directory { totals, .. } = entry {
          ensure!(
            directories.get(&entry.hash())?.is_some(),
            server_error::DirectoryUnverified {
              directory: hash,
              subdirectory: entry.hash(),
            },
          );

          self
            .read_directory(entry.hash())?
            .totals()
            .unwrap()
            .expect(*totals)
            .context(server_error::DirectoryEntryTotals {
              directory: hash,
              entry: name,
            })?;
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
