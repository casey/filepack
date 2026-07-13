use {super::*, reqwest::blocking::Response};

struct Context {
  entries: u64,
  entries_downloaded: u64,
  progress_bar: ProgressBar,
  totals: Totals,
}

impl Context {
  fn entry_downloaded(&mut self) {
    self.entries_downloaded += 1;
    self
      .progress_bar
      .set_message(progress_bar::entry_progress_message(
        self.entries_downloaded,
        self.entries,
      ));
  }

  fn new(options: &Options, totals: Totals) -> Self {
    let entries = totals.files.saturating_add(totals.directories);

    Self {
      entries,
      entries_downloaded: 0,
      progress_bar: progress_bar::with_message(
        options,
        totals.file_size.saturating_add(totals.directory_size),
        progress_bar::entry_progress_message(0, entries),
      ),
      totals,
    }
  }
}

#[derive(Parser)]
#[command(group = ArgGroup::new("target").required(true))]
pub(crate) struct Download {
  #[arg(
    group = "target",
    help = "Download file with <HASH>",
    long,
    value_name = "HASH"
  )]
  file: Option<Hash>,
  #[arg(help = "Download to <PATH>", value_name = "PATH")]
  output: Utf8PathBuf,
  #[arg(
    group = "target",
    help = "Download package with <FINGERPRINT>",
    long,
    value_name = "FINGERPRINT"
  )]
  package: Option<Fingerprint>,
  #[arg(help = "Download from server at <URL>", long, value_name = "URL", value_parser = parse_server_url)]
  server: Url,
}

impl Download {
  fn download_file(&self, options: &Options, hash: Hash, path: &Utf8Path) -> Result {
    ensure! {
      !filesystem::exists(path)?,
      error::FileAlreadyExists { path },
    }

    let client = client()?;

    let response = self.get_file(&client, hash)?;

    let bar = progress_bar::new(options, response.content_length().unwrap_or(0));

    self.write_response(response, hash, path, &bar)?;

    bar.finish();

    Ok(())
  }

  fn download_package(&self, options: &Options, fingerprint: Fingerprint) -> Result {
    ensure! {
      !filesystem::exists(&self.output)?,
      error::FileAlreadyExists { path: &self.output },
    }

    let client = client()?;

    let mut stack = vec![(Hash::from(fingerprint), self.output.clone(), None)];

    let mut directories = BTreeMap::new();

    let mut files = Vec::new();

    let mut context = None::<Context>;

    while let Some((hash, path, expected_totals)) = stack.pop() {
      let url = self.file_url(hash);

      let response = self.get_file(&client, hash)?;

      let cbor = response
        .bytes()
        .with_context(|_| error::ResponseBody { url: url.clone() })?;

      let actual = Hash::bytes(&cbor);

      ensure! {
        actual == hash,
        error::DownloadHashMismatch { actual, expected: hash },
      }

      let directory =
        Directory::decode_from_slice(&cbor).context(error::DecodeResponseDirectory { url })?;

      let actual = directory
        .totals()
        .context(error::DirectoryTotals { hash })?;

      if let Some(expected) = expected_totals {
        actual
          .expect(expected)
          .context(error::DirectoryTotals { hash })?;

        let context = context.as_mut().unwrap();

        context.progress_bar.inc(cbor.len().into_u64());

        context.entry_downloaded();
      } else {
        assert!(context.is_none());
        context = Some(Context::new(options, actual));
      }

      directories.insert(hash, cbor.to_vec());

      filesystem::create_dir_all(&path)?;

      for (component, entry) in directory.entries {
        let path = path.join(component);
        match entry {
          Entry::File { hash, .. } => files.push((hash, path)),
          Entry::Directory { hash, totals, .. } => stack.push((hash, path, Some(totals))),
        }
      }
    }

    let mut context = context.unwrap();

    for (hash, path) in &files {
      self.download_package_file(&client, &mut context, *hash, path)?;
    }

    let metadata_path = self.output.join(Metadata::CBOR_FILENAME);
    if let Some(cbor) = filesystem::read_opt(&metadata_path)? {
      let paths = files
        .iter()
        .map(|(_hash, path)| {
          let path = path.strip_prefix(&self.output).unwrap();
          path.try_into().context(error::Path { path })
        })
        .collect::<Result<HashSet<RelativePath>>>()?;

      Metadata::decode_from_slice(&cbor)
        .context(error::DecodeMetadataCbor {
          path: metadata_path,
        })?
        .check_files(&paths)?;
    }

    let mut builder = ArchiveBuilder::new();
    builder.files = directories;

    let package = Entry::directory(
      fingerprint.into(),
      builder.files[&fingerprint.into()].len().into_u64(),
      context.totals,
    );

    let archive = builder.build_package(package, &BTreeSet::new()).unwrap();

    filesystem::write(
      &self.output.join(Manifest::FILENAME),
      archive.encode_to_vec(),
    )?;

    context.progress_bar.finish();

    Ok(())
  }

  fn download_package_file(
    &self,
    client: &Client,
    context: &mut Context,
    hash: Hash,
    path: &Utf8Path,
  ) -> Result {
    ensure! {
      !filesystem::exists(path)?,
      error::FileAlreadyExists { path },
    }

    let response = self.get_file(client, hash)?;

    self.write_response(response, hash, path, &context.progress_bar)?;

    context.entry_downloaded();

    Ok(())
  }

  fn file_url(&self, hash: Hash) -> Url {
    self.server.join(&format!("file/{hash}")).unwrap()
  }

  fn get_file(&self, client: &Client, hash: Hash) -> Result<Response> {
    let url = self.file_url(hash);

    let response = client.get(url).send().check_status()?;

    Ok(response)
  }

  pub(crate) fn run(self, options: Options) -> Result {
    if let Some(hash) = self.file {
      self.download_file(&options, hash, &self.output)
    } else {
      self.download_package(&options, self.package.unwrap())
    }
  }

  fn write_response(
    &self,
    mut response: Response,
    hash: Hash,
    path: &Utf8Path,
    bar: &ProgressBar,
  ) -> Result {
    let output_directory = path
      .parent()
      .filter(|parent| !parent.as_str().is_empty())
      .unwrap_or(Utf8Path::new("."));

    let tempfile = transfer_tempfile(hash, output_directory).context(error::FilesystemIo {
      path: output_directory,
    })?;

    let mut writer = HashingWriter::new(tempfile);

    response
      .copy_to(&mut bar.wrap_write(&mut writer))
      .with_context(|_| error::ResponseBody {
        url: self.file_url(hash),
      })?;

    let (actual, tempfile) = writer.finalize();

    ensure! {
      actual == hash,
      error::DownloadHashMismatch { actual, expected: hash },
    }

    tempfile
      .persist_noclobber(path)
      .map_err(|error| error.error)
      .context(error::FilesystemIo { path })?;

    Ok(())
  }
}
