use {super::*, reqwest::blocking::Response};

struct Context {
  client: Client,
  progress_bar: ProgressBar,
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
  #[arg(help = "Download from server at <URL>", long, value_name = "URL", value_parser = CheckedUrl::check)]
  server: Url,
}

impl Download {
  fn download_file(&self, options: &Options, hash: Hash, path: &Utf8Path) -> Result {
    ensure! {
      !filesystem::exists(path)?,
      error::FileAlreadyExists { path },
    }

    let client = Client::new(options, self.server.clone(), None)?;

    let response = client.file(hash)?;

    let bar = ProgressBar::bytes(options, response.content_length().unwrap_or_default());

    Self::write_response(&client, response, hash, path, &bar, None)?;

    Ok(())
  }

  fn download_package(&self, options: &Options, fingerprint: Fingerprint) -> Result {
    ensure! {
      !filesystem::exists(&self.output)?,
      error::FileAlreadyExists { path: &self.output },
    }

    let client = Client::new(options, self.server.clone(), None)?;

    let mut stack = vec![(Hash::from(fingerprint), self.output.clone(), None)];

    let mut directories = BTreeMap::new();

    let mut files = Vec::new();

    let mut totals = None::<Totals>;

    let mut progress_bar = ProgressBar::items(options, 0, 0, "entries");

    while let Some((hash, path, expected_totals)) = stack.pop() {
      let url = client.file_url(hash);

      let response = client.file(hash)?;

      let deco = response
        .bytes()
        .with_context(|_| error::ResponseBody { url: url.clone() })?;

      let actual = Hash::bytes(&deco);

      ensure! {
        actual == hash,
        error::DownloadHashMismatch { actual, expected: hash },
      }

      let directory =
        Directory::decode_from_slice(&deco).context(error::DecodeResponseDirectory { url })?;

      let actual = directory
        .totals()
        .context(error::DirectoryTotals { hash })?;

      if let Some(expected) = expected_totals {
        actual
          .expect(expected)
          .context(error::DirectoryTotals { hash })?;

        progress_bar.inc(deco.len().into_u64());
      } else {
        assert!(totals.is_none());
        totals = Some(actual);

        progress_bar.set_totals(
          actual.file_size.saturating_add(actual.directory_size),
          actual.files.saturating_add(actual.directories),
        );
      }

      progress_bar.item_done();

      directories.insert(hash, deco.to_vec());

      filesystem::create_dir_all(&path)?;

      for (component, entry) in directory.entries {
        let path = path.join(component);
        match entry {
          Entry::File { hash, size } => files.push((hash, path, size)),
          Entry::Directory { hash, totals, .. } => stack.push((hash, path, Some(totals))),
        }
      }
    }

    let totals = totals.unwrap();

    let mut context = Context {
      client,
      progress_bar,
    };

    for (hash, path, size) in &files {
      Self::download_package_file(&mut context, *hash, path, *size)?;
    }

    let metadata_path = self.output.join(Metadata::DECO_FILENAME);
    if let Some(deco) = filesystem::read_opt(&metadata_path)? {
      let paths = files
        .iter()
        .map(|(_hash, path, _size)| {
          let path = path.strip_prefix(&self.output).unwrap();
          path.try_into().context(error::Path { path })
        })
        .collect::<Result<HashSet<RelativePath>>>()?;

      Metadata::decode_from_slice(&deco)
        .context(error::DecodeMetadataDeco {
          path: metadata_path,
        })?
        .check_files(&paths)?;
    }

    let mut builder = ArchiveBuilder::new();
    builder.files = directories;

    let package = Entry::directory(
      fingerprint.into(),
      builder.files[&fingerprint.into()].len().into_u64(),
      totals,
    );

    let archive = builder.build_package(package, &BTreeSet::new()).unwrap();

    filesystem::write(
      &self.output.join(Manifest::FILENAME),
      archive.encode_to_vec(),
    )?;

    Ok(())
  }

  fn download_package_file(
    context: &mut Context,
    hash: Hash,
    path: &Utf8Path,
    size: u64,
  ) -> Result {
    ensure! {
      !filesystem::exists(path)?,
      error::FileAlreadyExists { path },
    }

    let response = context.client.file(hash)?;

    Self::write_response(
      &context.client,
      response,
      hash,
      path,
      &context.progress_bar,
      Some(size),
    )?;

    context.progress_bar.item_done();

    Ok(())
  }

  pub(crate) fn run(self, options: Options) -> Result {
    if let Some(hash) = self.file {
      self.download_file(&options, hash, &self.output)
    } else {
      self.download_package(&options, self.package.unwrap())
    }
  }

  fn write_response(
    client: &Client,
    mut response: Response,
    hash: Hash,
    path: &Utf8Path,
    bar: &ProgressBar,
    expected_size: Option<u64>,
  ) -> Result {
    let output_directory = path
      .parent()
      .filter(|parent| !parent.as_str().is_empty())
      .unwrap_or(Utf8Path::new("."));

    let tempfile = transfer_tempfile(hash, output_directory).context(error::FilesystemIo {
      path: output_directory,
    })?;

    let mut writer = HashingWriter::new(tempfile);

    let size = response
      .copy_to(&mut bar.wrap_write(&mut writer))
      .with_context(|_| error::ResponseBody {
        url: client.file_url(hash),
      })?;

    let (actual, tempfile) = writer.finalize();

    ensure! {
      actual == hash,
      error::DownloadHashMismatch { actual, expected: hash },
    }

    if let Some(expected) = expected_size {
      ensure! {
        size == expected,
        error::DownloadSizeMismatch { actual: size, expected },
      }
    }

    tempfile
      .persist_noclobber(path)
      .map_err(|error| error.error)
      .context(error::FilesystemIo { path })?;

    Ok(())
  }
}
