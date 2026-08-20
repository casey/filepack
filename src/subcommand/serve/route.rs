use super::*;

pub(crate) async fn api_delete_package(
  _: Authenticated,
  server: ServerExtension,
  Path(fingerprint): Path<Fingerprint>,
) -> ServerResult {
  block_in_place(|| server.delete_package(fingerprint))
}

pub(crate) async fn api_gc(
  _: Authenticated,
  server: ServerExtension,
) -> ServerResult<CborResponse<api::gc::Response>> {
  block_in_place(|| Ok(CborResponse(server.gc()?)))
}

pub(crate) async fn api_missing(
  server: ServerExtension,
  Cbor(request): Cbor<api::missing::Request, { MIB }>,
) -> ServerResult<CborResponse<api::missing::Response>> {
  block_in_place(|| {
    Ok(CborResponse(api::missing::Response {
      hashes: server.missing(&request.hashes)?.into(),
    }))
  })
}

pub(crate) async fn api_packages(
  server: ServerExtension,
) -> ServerResult<CborResponse<api::packages::Response>> {
  block_in_place(|| {
    Ok(CborResponse(api::packages::Response {
      packages: server.fingerprints()?.into(),
    }))
  })
}

pub(crate) async fn api_verify_directory(
  _: Authenticated,
  server: ServerExtension,
  hash: Path<Hash>,
) -> ServerResult {
  block_in_place(|| server.verify_directory(*hash))
}

pub(crate) async fn api_verify_package(
  _: Authenticated,
  server: ServerExtension,
  Path(fingerprint): Path<Fingerprint>,
) -> ServerResult {
  block_in_place(|| server.verify_package(fingerprint))
}

pub(crate) async fn artwork(
  server: ServerExtension,
  fingerprint: Path<Fingerprint>,
  range: Option<TypedHeader<headers::Range>>,
) -> ServerResult<Resource> {
  block_in_place(|| Ok(server.artwork(*fingerprint)?.range(range)))
}

pub(crate) async fn directory(
  server: ServerExtension,
  server_config: ServerConfigExtension,
  Path(hash): Path<Hash>,
) -> PageResult<DirectoryHtml> {
  block_in_place(|| {
    Ok(
      DirectoryHtml {
        directory: server.directory(hash)?,
        hash,
      }
      .page(server_config.url.clone()),
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

pub(crate) async fn file_with_path(
  server: ServerExtension,
  Path((hash, path)): Path<(Hash, RelativePath)>,
  range: Option<TypedHeader<headers::Range>>,
) -> ServerResult<Response> {
  block_in_place(|| {
    let Some(resource_type) = ResourceType::from_filename(path.filename()) else {
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

pub(crate) async fn files(
  server: ServerExtension,
  server_config: ServerConfigExtension,
) -> PageResult<FilesHtml> {
  block_in_place(|| {
    Ok(
      FilesHtml {
        files: server.files()?,
      }
      .page(server_config.url.clone()),
    )
  })
}

pub(crate) async fn home() -> ServerResult<StaticAsset> {
  StaticAsset::get("index.html")
}

pub(crate) async fn install_script() -> ServerResult<StaticAsset> {
  StaticAsset::get("install.sh")
}

pub(crate) async fn media_audio_item(
  server: ServerExtension,
  Path((fingerprint, Ordinal(item))): Path<(Fingerprint, Ordinal)>,
  range: Option<TypedHeader<headers::Range>>,
) -> ServerResult<Resource> {
  block_in_place(|| {
    Ok(
      server
        .media_item(fingerprint, item, MediaType::Audio)?
        .range(range),
    )
  })
}

pub(crate) async fn media_image_item(
  server: ServerExtension,
  Path((fingerprint, Ordinal(item))): Path<(Fingerprint, Ordinal)>,
  range: Option<TypedHeader<headers::Range>>,
) -> ServerResult<Resource> {
  block_in_place(|| {
    Ok(
      server
        .media_item(fingerprint, item, MediaType::Image)?
        .range(range),
    )
  })
}

pub(crate) async fn media_video_item(
  server: ServerExtension,
  Path((fingerprint, Ordinal(item))): Path<(Fingerprint, Ordinal)>,
  range: Option<TypedHeader<headers::Range>>,
) -> ServerResult<Resource> {
  block_in_place(|| {
    Ok(
      server
        .media_item(fingerprint, item, MediaType::Video)?
        .range(range),
    )
  })
}

pub(crate) async fn mount(
  server: ServerExtension,
  server_config: ServerConfigExtension,
  Path(fingerprint): Path<Fingerprint>,
  range: Option<TypedHeader<headers::Range>>,
) -> ServerResult<Resource> {
  mount_file(
    server,
    server_config,
    Path((fingerprint, "index.html".parse().unwrap())),
    range,
  )
  .await
}

pub(crate) async fn mount_file(
  server: ServerExtension,
  server_config: ServerConfigExtension,
  Path((fingerprint, path)): Path<(Fingerprint, RelativePath)>,
  range: Option<TypedHeader<headers::Range>>,
) -> ServerResult<Resource> {
  block_in_place(|| {
    ensure! {
      server_config.mounts.contains(&fingerprint),
      server_error::PackageNotMounted { fingerprint },
    }

    let path = format!("static/{path}").parse::<RelativePath>().unwrap();

    let hash = server.package_file(fingerprint, &path)?;

    let content_type = mime_guess::from_path(&path).first_or_octet_stream();

    Ok(
      server
        .open_file(hash)?
        .range(range)
        .unsandboxed_content_type(content_type),
    )
  })
}

pub(crate) async fn mount_redirect(Path(fingerprint): Path<Fingerprint>) -> Redirect {
  Redirect::permanent(&format!("/mount/{fingerprint}/"))
}

pub(crate) async fn package(
  server: ServerExtension,
  server_config: ServerConfigExtension,
  Path(fingerprint): Path<Fingerprint>,
) -> PageResult<PackageHtml> {
  block_in_place(|| {
    Ok(
      server
        .package_html(fingerprint, server_config.mounts.contains(&fingerprint))?
        .page(server_config.url.clone()),
    )
  })
}

pub(crate) async fn package_item(
  server: ServerExtension,
  server_config: ServerConfigExtension,
  Path((fingerprint, Ordinal(index))): Path<(Fingerprint, Ordinal)>,
) -> ServerResult<Response> {
  block_in_place(|| {
    let metadata = server.package_metadata(fingerprint)?;

    let media = metadata
      .media
      .as_ref()
      .context(server_error::PackageMediaMetadataNotFound { fingerprint })?;

    let ty = media.discriminant();

    ensure! {
      ty.has_items(),
      server_error::MediaTypeDoesNotHaveItems {
        ty,
      }
    }

    ensure! {
      media.items() > index,
      server_error::MediaItemDoesNotExist {
        count: media.items(),
        fingerprint,
        index,
        ty,
      },
    }

    match media {
      Media::Audio { .. } => Ok(
        AudioHtml {
          audio: index,
          fingerprint,
          metadata,
        }
        .page(server_config.url.clone())
        .into_response(),
      ),
      Media::Image { .. } => Ok(
        ImageHtml {
          fingerprint,
          image: index,
          metadata,
        }
        .page(server_config.url.clone())
        .into_response(),
      ),
      Media::Video { .. } => Ok(
        VideoHtml {
          fingerprint,
          video: index,
        }
        .page(server_config.url.clone())
        .into_response(),
      ),
      Media::Web => unreachable!(),
    }
  })
}

pub(crate) async fn package_media(
  server: ServerExtension,
  server_config: ServerConfigExtension,
  Path(fingerprint): Path<Fingerprint>,
) -> PageResult<MediaHtml> {
  block_in_place(|| {
    let metadata = server.package_metadata(fingerprint)?;

    ensure! {
      metadata.media.is_some(),
      server_error::PackageMediaMetadataNotFound { fingerprint },
    }

    Ok(
      MediaHtml {
        fingerprint,
        metadata,
      }
      .page(server_config.url.clone()),
    )
  })
}

pub(crate) async fn packages(
  server: ServerExtension,
  server_config: ServerConfigExtension,
  Query(query): Query<PackagesQuery>,
) -> PageResult<PackagesHtml> {
  block_in_place(|| {
    Ok(
      PackagesHtml {
        packages: server.packages()?,
        view: query.view.unwrap_or_default(),
      }
      .page(server_config.url.clone()),
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
