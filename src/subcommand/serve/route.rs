use super::*;

pub(crate) async fn artwork(
  server: ServerExtension,
  fingerprint: Path<Fingerprint>,
  range: Option<TypedHeader<headers::Range>>,
) -> ServerResult<Resource> {
  Ok(block_in_place(|| server.artwork(*fingerprint))?.range(range))
}

pub(crate) async fn directory(
  server: ServerExtension,
  Path(hash): Path<Hash>,
) -> PageResult<DirectoryHtml> {
  Ok(
    DirectoryHtml {
      directory: block_in_place(|| server.directory(hash))?,
      hash,
    }
    .into(),
  )
}

pub(crate) async fn fallback(uri: Uri) -> ServerResult<Response> {
  if let Some(component) = uri.path().strip_prefix('/')
    && !component.contains('/')
    && component.to_ascii_lowercase().starts_with("package1")
  {
    let fingerprint = component
      .parse::<Fingerprint>()
      .context(server_error::FingerprintParse)?;

    return Ok(Redirect::permanent(&format!("/package/{fingerprint}")).into_response());
  }

  Ok(
    StaticAsset::get("404.html")?
      .status(StatusCode::NOT_FOUND)
      .into_response(),
  )
}

pub(crate) async fn favicon() -> ServerResult<StaticAsset> {
  StaticAsset::get("favicon.png")
}

pub(crate) async fn file(
  server: ServerExtension,
  hash: Path<Hash>,
  range: Option<TypedHeader<headers::Range>>,
) -> ServerResult<Resource> {
  Ok(block_in_place(|| server.open_file(*hash))?.range(range))
}

pub(crate) async fn file_with_name(
  server: ServerExtension,
  Path((hash, name)): Path<(Hash, ComponentBuf)>,
  range: Option<TypedHeader<headers::Range>>,
) -> ServerResult<Response> {
  let Some(resource_type) = ResourceType::from_filename(&name) else {
    return Ok(Redirect::temporary(&format!("/file/{hash}")).into_response());
  };

  Ok(
    block_in_place(|| server.open_file(hash))?
      .ty(resource_type)
      .range(range)
      .into_response(),
  )
}

pub(crate) async fn files(server: ServerExtension) -> PageResult<FilesHtml> {
  Ok(
    FilesHtml {
      files: block_in_place(|| server.files())?,
    }
    .into(),
  )
}

pub(crate) async fn home() -> ServerResult<StaticAsset> {
  StaticAsset::get("index.html")
}

pub(crate) async fn install_script() -> ServerResult<StaticAsset> {
  StaticAsset::get("install.sh")
}

pub(crate) async fn media_audio_track(
  server: ServerExtension,
  Path((fingerprint, Ordinal(track))): Path<(Fingerprint, Ordinal)>,
  range: Option<TypedHeader<headers::Range>>,
) -> ServerResult<Resource> {
  Ok(block_in_place(|| server.media_audio_track(fingerprint, track))?.range(range))
}

pub(crate) async fn media_image_image(
  server: ServerExtension,
  Path((fingerprint, Ordinal(image))): Path<(Fingerprint, Ordinal)>,
  range: Option<TypedHeader<headers::Range>>,
) -> ServerResult<Resource> {
  Ok(block_in_place(|| server.media_image_image(fingerprint, image))?.range(range))
}

pub(crate) async fn missing(
  _: Authenticated,
  server: ServerExtension,
  Cbor(request): Cbor<api::missing::Request, { MIB }>,
) -> ServerResult<Vec<u8>> {
  let missing = block_in_place(|| server.missing(&request.hashes))?;

  Ok(
    api::missing::Response {
      hashes: missing.into(),
    }
    .encode_to_vec(),
  )
}

pub(crate) async fn package(
  server: ServerExtension,
  Path(fingerprint): Path<Fingerprint>,
) -> PageResult<PackageHtml> {
  Ok(
    PackageHtml {
      fingerprint,
      metadata: block_in_place(|| server.package_metadata(fingerprint))?,
    }
    .into(),
  )
}

pub(crate) async fn packages(server: ServerExtension) -> PageResult<PackagesHtml> {
  Ok(
    PackagesHtml {
      packages: block_in_place(|| server.packages())?,
    }
    .into(),
  )
}
pub(crate) async fn static_asset(path: Path<String>) -> ServerResult<StaticAsset> {
  StaticAsset::get(&path)
}

pub(crate) async fn upload_file(
  _: Authenticated,
  server: ServerExtension,
  hash: Path<Hash>,
  body: Body,
) -> ServerResult {
  server.write_file(*hash, body).await
}

pub(crate) async fn verify_directory(
  _: Authenticated,
  server: ServerExtension,
  hash: Path<Hash>,
) -> ServerResult {
  block_in_place(|| server.verify_directory(*hash))
}

pub(crate) async fn verify_package(
  _: Authenticated,
  server: ServerExtension,
  Path(fingerprint): Path<Fingerprint>,
) -> ServerResult {
  block_in_place(|| server.verify_package(fingerprint))
}
