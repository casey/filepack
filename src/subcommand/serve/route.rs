use super::*;

pub(crate) async fn artwork(
  server: ServerExtension,
  fingerprint: Path<Fingerprint>,
  range: Option<TypedHeader<headers::Range>>,
) -> ServerResult<Resource> {
  block_in_place(|| Ok(server.artwork(*fingerprint)?.range(range)))
}

pub(crate) async fn directory(
  server: ServerExtension,
  Path(hash): Path<Hash>,
) -> PageResult<DirectoryHtml> {
  block_in_place(|| {
    Ok(
      DirectoryHtml {
        directory: server.directory(hash)?,
        hash,
      }
      .into(),
    )
  })
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
  block_in_place(|| Ok(server.open_file(*hash)?.range(range)))
}

pub(crate) async fn file_with_name(
  server: ServerExtension,
  Path((hash, name)): Path<(Hash, ComponentBuf)>,
  range: Option<TypedHeader<headers::Range>>,
) -> ServerResult<Response> {
  block_in_place(|| {
    let Some(resource_type) = ResourceType::from_filename(&name) else {
      return Ok(Redirect::temporary(&format!("/file/{hash}")).into_response());
    };

    Ok(
      server
        .open_file(hash)?
        .ty(resource_type)
        .range(range)
        .into_response(),
    )
  })
}

pub(crate) async fn files(server: ServerExtension) -> PageResult<FilesHtml> {
  block_in_place(|| {
    Ok(
      FilesHtml {
        files: server.files()?,
      }
      .into(),
    )
  })
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
  block_in_place(|| Ok(server.media_audio_track(fingerprint, track)?.range(range)))
}

pub(crate) async fn media_image_image(
  server: ServerExtension,
  Path((fingerprint, Ordinal(image))): Path<(Fingerprint, Ordinal)>,
  range: Option<TypedHeader<headers::Range>>,
) -> ServerResult<Resource> {
  block_in_place(|| Ok(server.media_image_image(fingerprint, image)?.range(range)))
}

pub(crate) async fn media_video_video(
  server: ServerExtension,
  Path((fingerprint, Ordinal(video))): Path<(Fingerprint, Ordinal)>,
  range: Option<TypedHeader<headers::Range>>,
) -> ServerResult<Resource> {
  block_in_place(|| Ok(server.media_video_video(fingerprint, video)?.range(range)))
}

pub(crate) async fn missing(
  _: Authenticated,
  server: ServerExtension,
  Cbor(request): Cbor<api::missing::Request, { MIB }>,
) -> ServerResult<Vec<u8>> {
  block_in_place(|| {
    Ok(
      api::missing::Response {
        hashes: server.missing(&request.hashes)?.into(),
      }
      .encode_to_vec(),
    )
  })
}

pub(crate) async fn package(
  server: ServerExtension,
  fingerprint: Path<Fingerprint>,
) -> PageResult<PackageHtml> {
  block_in_place(|| Ok(server.package_html(*fingerprint)?.into()))
}

pub(crate) async fn package_item(
  server: ServerExtension,
  Path((fingerprint, Ordinal(index))): Path<(Fingerprint, Ordinal)>,
) -> ServerResult<Response> {
  block_in_place(|| {
    let metadata = server.package_metadata(fingerprint)?;

    let media = metadata
      .media
      .as_ref()
      .context(server_error::PackageMediaMetadataNotFound { fingerprint })?;

    ensure! {
      media.items() > index,
      server_error::MediaItemDoesNotExist {
        count: media.items(),
        fingerprint,
        index,
        ty: media.discriminant(),
      },
    }

    match media {
      Media::Audio { .. } => Ok(
        AudioHtml {
          audio: index,
          fingerprint,
          metadata,
        }
        .page()
        .into_response(),
      ),
      Media::Image { .. } => Ok(
        ImageHtml {
          fingerprint,
          image: index,
          metadata,
        }
        .page()
        .into_response(),
      ),
      Media::Video { .. } => Ok(
        VideoHtml {
          fingerprint,
          video: index,
        }
        .page()
        .into_response(),
      ),
    }
  })
}

pub(crate) async fn packages(server: ServerExtension) -> PageResult<PackagesHtml> {
  block_in_place(|| {
    Ok(
      PackagesHtml {
        packages: server.packages()?,
      }
      .into(),
    )
  })
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
