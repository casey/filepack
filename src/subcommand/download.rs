use {super::*, reqwest::blocking::Response};

struct Context {
  client: Client,
  files: u64,
  files_downloaded: u64,
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
  #[arg(help = "Download from server at <URL>", long, value_name = "URL", value_parser = parse_server_url)]
  server: Url,
}

impl Download {
  fn download_file(&self, options: &Options, hash: Hash, path: &Utf8Path) -> Result {
    ensure! {
      !filesystem::exists(path)?,
      error::FileAlreadyExists { path },
    }

    let client = Client::new();

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

    let client = Client::new();

    let mut stack = vec![(Hash::from(fingerprint), self.output.clone())];

    let mut directories = BTreeMap::new();

    let mut files = Vec::new();

    let mut bytes = 0u64;

    while let Some((hash, path)) = stack.pop() {
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

      directories.insert(hash, cbor.to_vec());

      filesystem::create_dir_all(&path)?;

      for (component, entry) in directory.entries {
        let path = path.join(component);
        match entry.ty {
          EntryType::Directory => stack.push((entry.hash, path)),
          EntryType::File => {
            bytes = bytes.saturating_add(entry.size);
            files.push((entry.hash, path));
          }
        }
      }
    }

    let file_count = files.len().into_u64();

    let progress_bar = progress_bar::with_files(options, bytes, file_count);

    let mut context = Context {
      client,
      files: file_count,
      files_downloaded: 0,
      progress_bar,
    };

    for (hash, path) in &files {
      self.download_package_file(&mut context, *hash, path)?;
    }

    let mut builder = ArchiveBuilder::new();
    builder.files = directories;

    let package = Entry {
      ty: EntryType::Directory,
      hash: fingerprint.into(),
      size: builder.files[&fingerprint.into()].len().into_u64(),
    };

    let archive = builder.build_package(package, &BTreeSet::new());

    filesystem::write(
      &self.output.join(Manifest::FILENAME),
      archive.encode_to_vec(),
    )?;

    context.progress_bar.finish();

    Ok(())
  }

  fn download_package_file(&self, context: &mut Context, hash: Hash, path: &Utf8Path) -> Result {
    ensure! {
      !filesystem::exists(path)?,
      error::FileAlreadyExists { path },
    }

    let response = self.get_file(&context.client, hash)?;

    self.write_response(response, hash, path, &context.progress_bar)?;

    context.files_downloaded += 1;

    context
      .progress_bar
      .set_message(progress_bar::file_progress_message(
        context.files_downloaded,
        context.files,
      ));

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
