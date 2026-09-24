use {
  super::*,
  redb::{Database, ReadOnlyTable, ReadableDatabase, ReadableTable, TableDefinition},
  templates::PackageHtml,
};

const DIRECTORIES: TableDefinition<Hash, ()> = TableDefinition::new("directories");
const METADATA: TableDefinition<DatabaseMetadata, u64> = TableDefinition::new("metadata");
const NUMBERS: TableDefinition<u64, Revision> = TableDefinition::new("numbers");
const PACKAGES: TableDefinition<Fingerprint, u64> = TableDefinition::new("packages");
const REVISIONS: TableDefinition<Revision, ()> = TableDefinition::new("revisions");
const SCHEMA_VERSION: u64 = 4;

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

    let number = tx
      .open_table(PACKAGES)?
      .remove(&fingerprint)?
      .context(server_error::PackageFingerprintNotFound { fingerprint })?
      .value();

    tx.open_table(NUMBERS)?.remove(&number)?;

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
    let context = filesystem_error::Io { path: &self.files };

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

    let mut revisions_removed = BTreeSet::new();

    {
      let mut directories = tx.open_table(DIRECTORIES)?;

      let mut revisions = tx.open_table(REVISIONS)?;

      let mut stack = Vec::new();

      for entry in tx.open_table(NUMBERS)?.iter()? {
        let revision = entry?.1.value();
        marked.insert(revision.into());
        stack.push(self.read_revision(revision)?.package.into());
      }

      for entry in revisions
        .extract_from_if::<Revision, _>(.., |revision, ()| !marked.contains(&revision.into()))?
      {
        revisions_removed.insert(entry?.0.value());
      }

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
        .context(filesystem_error::Io { path: &path })?
        .len();

      files_removed.insert(hash);
    }

    for &hash in &files_removed {
      let path = self.file_path(hash);
      fs::remove_file(&path).context(filesystem_error::Io { path })?;
    }

    tx.commit()?;

    Ok(api::gc::Response {
      bytes,
      directories: directories_removed.into(),
      files: files_removed.into(),
      revisions: revisions_removed.into(),
    })
  }

  pub(crate) fn has_package(&self, fingerprint: Fingerprint) -> ServerResult<bool> {
    Ok(
      self
        .database
        .begin_read()?
        .open_table(PACKAGES)?
        .get(&fingerprint)?
        .is_some(),
    )
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
      .metadata_deco(fingerprint)?
      .map(|metadata| Metadata::decode_from_slice(&metadata))
      .transpose()
      .context(server_error::PackageMetadataCorrupt { fingerprint })
  }

  fn metadata_deco(&self, fingerprint: Fingerprint) -> ServerResult<Option<Vec<u8>>> {
    let directory = self.read_directory(fingerprint.into())?;

    let Some(entry) = directory.entries.get(Metadata::DECO_FILENAME) else {
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
        .context(filesystem_error::Io { path: &path })?
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
        filesystem_error::Io { path: &path }.into_error(err).into()
      }
    })?;

    let content_length = file
      .metadata()
      .context(filesystem_error::Io { path })?
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
      server_error::PackageFingerprintNotFound { fingerprint },
    );

    self
      .resolve_path(fingerprint, path)?
      .context(server_error::PackageFileNotFound { fingerprint, path })
  }

  pub(crate) fn package_html(
    &self,
    package: PackageIdentifier,
    mounts: &HashSet<Fingerprint>,
  ) -> ServerResult<PackageHtml> {
    let (number, fingerprint) = self.resolve(package)?;

    let tx = self.database.begin_read()?;

    let numbers = tx.open_table(NUMBERS)?;

    let next = numbers
      .range((Bound::Excluded(number), Bound::Unbounded))?
      .next()
      .transpose()?
      .map(|(number, _fingerprint)| number.value());

    let prev = numbers
      .range(..number)?
      .next_back()
      .transpose()?
      .map(|(number, _fingerprint)| number.value());

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
      identifier: package,
      metadata,
      mounted: mounts.contains(&fingerprint),
      next,
      number,
      prev,
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
    packages: &ReadOnlyTable<Fingerprint, u64>,
    fingerprint: Fingerprint,
  ) -> ServerResult<Option<Metadata>> {
    ensure!(
      packages.get(&fingerprint)?.is_some(),
      server_error::PackageFingerprintNotFound { fingerprint },
    );

    self.metadata(fingerprint)
  }

  pub(crate) fn packages(&self, sort: Sort, order: Order) -> ServerResult<Vec<PackageSummary>> {
    let tx = self.database.begin_read()?;

    let directories = tx.open_table(DIRECTORIES)?;

    let mut packages = tx
      .open_table(PACKAGES)?
      .iter()?
      .map(|entry| {
        let (fingerprint, number) = entry?;
        let fingerprint = fingerprint.value();
        let number = number.value();

        let totals = self
          .directory_ext(&directories, fingerprint.into())?
          .totals()
          .unwrap();

        Ok(PackageSummary {
          fingerprint,
          metadata: self.metadata(fingerprint)?,
          number,
          totals,
        })
      })
      .collect::<ServerResult<Vec<PackageSummary>>>()?;

    packages.sort_by(|a, b| SortKey::compare(a, b, sort, order));

    Ok(packages)
  }

  fn read_directory(&self, hash: Hash) -> ServerResult<Directory> {
    Directory::decode_from_slice(&self.read_file(hash)?)
      .context(server_error::DirectoryCorrupt { hash })
  }

  fn read_file(&self, hash: Hash) -> ServerResult<Vec<u8>> {
    let path = self.file_path(hash);

    fs::read(&path).map_err(|err| {
      if err.kind() == io::ErrorKind::NotFound {
        server_error::FileNotFound { hash }.into_error(err)
      } else {
        filesystem_error::Io { path }.into_error(err).into()
      }
    })
  }

  fn read_revision(&self, revision: Revision) -> ServerResult<RevisionObject> {
    RevisionObject::decode_from_slice(&self.read_file(revision.into())?)
      .context(server_error::RevisionCorrupt { revision })
  }

  pub(crate) fn resolve(&self, identifier: PackageIdentifier) -> ServerResult<(u64, Fingerprint)> {
    let tx = self.database.begin_read()?;

    match identifier {
      PackageIdentifier::Fingerprint(fingerprint) => Ok((
        tx.open_table(PACKAGES)?
          .get(&fingerprint)?
          .context(server_error::PackageFingerprintNotFound { fingerprint })?
          .value(),
        fingerprint,
      )),
      PackageIdentifier::Number(number) => Ok((
        number,
        self
          .read_revision(
            tx.open_table(NUMBERS)?
              .get(&number)?
              .context(server_error::PackageNumberNotFound { number })?
              .value(),
          )?
          .package,
      )),
    }
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
    let tx = self.database.begin_write()?;

    let directory = Directory::decode_from_slice(&self.read_file(hash)?)
      .context(server_error::DirectoryDecode { hash })?;

    directory
      .totals()
      .context(server_error::DirectoryTotals { hash })?;

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
            filesystem_error::Io { path: &path }
              .into_error(error)
              .into()
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

  pub(crate) fn verify_package(
    &self,
    fingerprint: Fingerprint,
    replace: Option<u64>,
  ) -> ServerResult<u64> {
    let tx = self.database.begin_write()?;

    ensure!(
      tx.open_table(DIRECTORIES)?
        .get(&fingerprint.into())?
        .is_some(),
      server_error::PackageRootUnverified { fingerprint },
    );

    if let Some(metadata) = self.metadata_deco(fingerprint)? {
      let metadata = Metadata::decode_from_slice(&metadata)
        .context(server_error::PackageMetadataDecode { fingerprint })?;

      for path in metadata.files() {
        ensure!(
          self.resolve_path(fingerprint, &path)?.is_some(),
          server_error::PackageMetadataFileMissing { fingerprint, path },
        );
      }
    }

    let number = {
      let mut packages = tx.open_table(PACKAGES)?;
      let mut numbers = tx.open_table(NUMBERS)?;
      let mut revisions = tx.open_table(REVISIONS)?;

      let existing = packages.get(&fingerprint)?.map(|number| number.value());

      let revision_object = RevisionObject {
        version: Version::Zero,
        package: fingerprint,
        previous: None,
      };

      let revision = revision_object.hash();

      match (replace, existing) {
        (Some(number), Some(existing)) => {
          ensure!(
            number == existing,
            server_error::PackageNumberConflict {
              fingerprint,
              number: existing,
            },
          );
          number
        }
        (Some(number), None) => {
          let old = numbers
            .get(&number)?
            .context(server_error::PackageNumberNotFound { number })?
            .value();
          let old = self.read_revision(old)?.package;
          self.write_revision(&revision_object)?;
          revisions.insert(&revision, &())?;
          numbers.insert(&number, &revision)?;
          packages.remove(&old)?;
          packages.insert(&fingerprint, &number)?;
          number
        }
        (None, Some(number)) => number,
        (None, None) => {
          let mut metadata = tx.open_table(METADATA)?;
          let number = metadata.get(DatabaseMetadata::Number)?.unwrap().value();
          metadata.insert(DatabaseMetadata::Number, &(number + 1))?;
          self.write_revision(&revision_object)?;
          revisions.insert(&revision, &())?;
          numbers.insert(&number, &revision)?;
          packages.insert(&fingerprint, &number)?;
          number
        }
      }
    };

    tx.commit()?;

    Ok(number)
  }

  pub(crate) fn with_data_dir(data_dir: &Utf8Path) -> Result<Self> {
    let path = data_dir.join("database.redb");
    let database = Database::create(&path).context(error::DatabaseOpen { path })?;

    let tx = database.begin_write()?;

    if tx.list_tables()?.count() == 0 && tx.list_multimap_tables()?.count() == 0 {
      {
        let mut metadata = tx.open_table(METADATA)?;
        metadata.insert(DatabaseMetadata::Number, &1)?;
        metadata.insert(DatabaseMetadata::Schema, &SCHEMA_VERSION)?;

        tx.open_table(DIRECTORIES)?;
        tx.open_table(NUMBERS)?;
        tx.open_table(PACKAGES)?;
        tx.open_table(REVISIONS)?;
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
    let (file, temp_path) = transfer_tempfile(hash, &self.incoming)?.into_parts();

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
        .context(filesystem_error::Io {
          path: &temp_path_utf8,
        })?;
    }

    writer.flush().await.context(filesystem_error::Io {
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
      .context(filesystem_error::Io { path: &path })?
    {
      return Ok(());
    }

    temp_path
      .persist(&path)
      .map_err(|error| error.error)
      .context(filesystem_error::Io { path: &path })?;

    Ok(())
  }

  fn write_revision(&self, revision_object: &RevisionObject) -> ServerResult {
    let revision = revision_object.hash();

    let path = self.file_path(revision.into());

    if path
      .try_exists()
      .context(filesystem_error::Io { path: &path })?
    {
      return Ok(());
    }

    let mut tempfile = transfer_tempfile(revision.into(), &self.incoming)?;

    tempfile
      .write_all(&revision_object.encode_to_vec())
      .context(filesystem_error::Io {
        path: &self.incoming,
      })?;

    tempfile
      .persist(&path)
      .map_err(|error| error.error)
      .context(filesystem_error::Io { path: &path })?;

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
