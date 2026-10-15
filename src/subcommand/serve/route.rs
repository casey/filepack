use {
  super::*,
  axum::{http::Method, routing::MethodRouter},
  axum_extra::either::Either,
};

pub(crate) struct Route {
  pub(crate) endpoint: crate::reference::Endpoint,
  pub(crate) method: Method,
  pub(crate) path: &'static str,
  pub(crate) router: MethodRouter,
}

impl Route {
  pub(crate) fn all(schemas: &mut deco::Registry) -> Vec<Self> {
    vec![
      api_delete_number_route(schemas),
      api_file_route(schemas),
      api_gc_route(schemas),
      api_head_route(schemas),
      api_missing_route(schemas),
      api_number_route(schemas),
      api_numbers_route(schemas),
      api_package_route(schemas),
      api_packages_route(schemas),
      api_upload_file_route(schemas),
      api_verify_directory_route(schemas),
      api_verify_package_route(schemas),
      api_verify_revision_route(schemas),
      artwork_route(schemas),
      artwork_thumbnail_route(schemas),
      directory_route(schemas),
      favicon_route(schemas),
      file_route(schemas),
      file_with_path_route(schemas),
      files_route(schemas),
      home_route(schemas),
      install_script_route(schemas),
      media_audio_item_route(schemas),
      media_image_item_route(schemas),
      media_image_item_thumbnail_route(schemas),
      media_video_item_route(schemas),
      media_video_item_placeholder_route(schemas),
      media_video_item_placeholder_thumbnail_route(schemas),
      mount_route(schemas),
      mount_file_route(schemas),
      mount_redirect_route(schemas),
      package_route(schemas),
      package_history_route(schemas),
      package_item_route(schemas),
      package_media_route(schemas),
      packages_route(schemas),
      static_asset_route(schemas),
    ]
  }
}

#[route(DELETE, "/api/number/{number}")]
pub(crate) async fn api_delete_number(
  _: Authenticated,
  server: ServerExtension,
  Path(number): Path<u64>,
) -> ServerResult {
  block_in_place(|| server.delete_number(number))
}

#[route(GET, "/api/file/{hash}")]
pub(crate) async fn api_file(
  server: ServerExtension,
  hash: Path<Hash>,
  range: Option<TypedHeader<headers::Range>>,
) -> ServerResult<Resource> {
  block_in_place(|| Ok(server.open_file(*hash)?.range(range)))
}

#[route(POST, "/api/gc")]
pub(crate) async fn api_gc(
  _: Authenticated,
  server: ServerExtension,
) -> ServerResult<DecoResponse<api::gc::Response>> {
  block_in_place(|| Ok(DecoResponse(server.gc()?)))
}

#[route(HEAD, "/api/head/{revision}")]
pub(crate) async fn api_head(
  server: ServerExtension,
  Path(revision): Path<Revision>,
) -> ServerResult {
  block_in_place(|| {
    ensure!(
      server.is_head(revision)?,
      server_error::HeadNotFound { revision },
    );
    Ok(())
  })
}

#[route(POST, "/api/missing")]
pub(crate) async fn api_missing(
  server: ServerExtension,
  DecoRequest(request): DecoRequest<api::missing::Request, { MIB }>,
) -> ServerResult<DecoResponse<api::missing::Response>> {
  block_in_place(|| {
    Ok(DecoResponse(api::missing::Response {
      hashes: server.missing(&request.hashes)?.into(),
    }))
  })
}

#[route(GET, "/api/number/{number}")]
pub(crate) async fn api_number(
  server: ServerExtension,
  Path(number): Path<u64>,
) -> ServerResult<DecoResponse<api::number::Response>> {
  block_in_place(|| Ok(DecoResponse(server.head(number)?)))
}

#[route(GET, "/api/numbers")]
pub(crate) async fn api_numbers(
  server: ServerExtension,
) -> ServerResult<DecoResponse<api::numbers::Response>> {
  block_in_place(|| {
    Ok(DecoResponse(api::numbers::Response {
      numbers: server.numbers()?.into(),
    }))
  })
}

#[route(HEAD, "/api/package/{fingerprint}")]
pub(crate) async fn api_package(
  server: ServerExtension,
  Path(fingerprint): Path<Fingerprint>,
) -> ServerResult {
  block_in_place(|| {
    ensure!(
      server.has_package(fingerprint)?,
      server_error::PackageFingerprintNotFound { fingerprint },
    );
    Ok(())
  })
}

#[route(GET, "/api/packages")]
pub(crate) async fn api_packages(
  server: ServerExtension,
) -> ServerResult<DecoResponse<api::packages::Response>> {
  block_in_place(|| {
    Ok(DecoResponse(api::packages::Response {
      packages: server.fingerprints()?.into(),
    }))
  })
}

#[route(PUT, "/api/file/{hash}")]
pub(crate) async fn api_upload_file(
  _: Authenticated,
  server: ServerExtension,
  hash: Path<Hash>,
  body: Body,
) -> ServerResult {
  server.write_file(*hash, body).await
}

#[route(POST, "/api/directory/{hash}")]
pub(crate) async fn api_verify_directory(
  _: Authenticated,
  server: ServerExtension,
  hash: Path<Hash>,
) -> ServerResult {
  block_in_place(|| server.verify_directory(*hash))
}

#[route(POST, "/api/package/{fingerprint}")]
pub(crate) async fn api_verify_package(
  _: Authenticated,
  server: ServerExtension,
  Path(fingerprint): Path<Fingerprint>,
) -> ServerResult {
  block_in_place(|| server.verify_package(fingerprint))
}

#[route(POST, "/api/revision/{revision}")]
pub(crate) async fn api_verify_revision(
  _: Authenticated,
  server: ServerExtension,
  Path(revision): Path<Revision>,
  DecoRequest(request): DecoRequest<api::revision::Request, { MIB }>,
) -> ServerResult<DecoResponse<api::revision::Response>> {
  block_in_place(|| {
    Ok(DecoResponse(api::revision::Response {
      number: server.verify_revision(revision, request)?,
    }))
  })
}

#[route(GET, "/artwork/{fingerprint}")]
pub(crate) async fn artwork(
  server: ServerExtension,
  fingerprint: Path<Fingerprint>,
  range: Option<TypedHeader<headers::Range>>,
) -> ServerResult<Resource> {
  block_in_place(|| Ok(server.artwork(*fingerprint, false)?.range(range)))
}

#[route(GET, "/artwork/{fingerprint}/thumbnail")]
pub(crate) async fn artwork_thumbnail(
  server: ServerExtension,
  fingerprint: Path<Fingerprint>,
  range: Option<TypedHeader<headers::Range>>,
) -> ServerResult<Resource> {
  block_in_place(|| Ok(server.artwork(*fingerprint, true)?.range(range)))
}

#[route(GET, "/directory/{hash}")]
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

#[route(GET, "/favicon.ico")]
pub(crate) async fn favicon() -> ServerResult<StaticAsset> {
  StaticAsset::get("favicon.svg")
}

#[route(GET, "/file/{hash}")]
pub(crate) async fn file(
  server: ServerExtension,
  hash: Path<Hash>,
  range: Option<TypedHeader<headers::Range>>,
) -> ServerResult<Resource> {
  api_file(server, hash, range).await
}

#[route(GET, "/file/{hash}/{*path}")]
pub(crate) async fn file_with_path(
  server: ServerExtension,
  Path((hash, path)): Path<(Hash, RelativePath)>,
  range: Option<TypedHeader<headers::Range>>,
) -> ServerResult<Either<Resource, Redirect>> {
  block_in_place(|| {
    let Some(resource_type) = ResourceType::from_filename(path.filename()) else {
      return Ok(Either::E2(Redirect::temporary(&format!("/file/{hash}"))));
    };

    Ok(Either::E1(
      server.open_file(hash)?.ty(resource_type).range(range),
    ))
  })
}

#[route(GET, "/files")]
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

#[route(GET, "/")]
pub(crate) async fn home(
  server: ServerExtension,
  server_config: ServerConfigExtension,
) -> PageResult<HomeHtml> {
  block_in_place(|| {
    Ok(
      HomeHtml {
        packages: server.packages(Sort::Title, Order::Ascending)?,
      }
      .page(server_config.url.clone()),
    )
  })
}

#[route(GET, "/install.sh")]
pub(crate) async fn install_script() -> ServerResult<StaticAsset> {
  StaticAsset::get("install.sh")
}

#[route(GET, "/media/audio/{fingerprint}/item/{item}")]
pub(crate) async fn media_audio_item(
  server: ServerExtension,
  Path((fingerprint, Ordinal(item))): Path<(Fingerprint, Ordinal)>,
  range: Option<TypedHeader<headers::Range>>,
) -> ServerResult<Resource> {
  block_in_place(|| {
    Ok(
      server
        .media_item(
          fingerprint,
          item,
          MediaType::Audio,
          MediaItemResource::Original,
        )?
        .range(range),
    )
  })
}

#[route(GET, "/media/image/{fingerprint}/item/{item}")]
pub(crate) async fn media_image_item(
  server: ServerExtension,
  Path((fingerprint, Ordinal(item))): Path<(Fingerprint, Ordinal)>,
  range: Option<TypedHeader<headers::Range>>,
) -> ServerResult<Resource> {
  block_in_place(|| {
    Ok(
      server
        .media_item(
          fingerprint,
          item,
          MediaType::Image,
          MediaItemResource::Original,
        )?
        .range(range),
    )
  })
}

#[route(GET, "/media/image/{fingerprint}/item/{item}/thumbnail")]
pub(crate) async fn media_image_item_thumbnail(
  server: ServerExtension,
  Path((fingerprint, Ordinal(item))): Path<(Fingerprint, Ordinal)>,
  range: Option<TypedHeader<headers::Range>>,
) -> ServerResult<Resource> {
  block_in_place(|| {
    Ok(
      server
        .media_item(
          fingerprint,
          item,
          MediaType::Image,
          MediaItemResource::Thumbnail,
        )?
        .range(range),
    )
  })
}

#[route(GET, "/media/video/{fingerprint}/item/{item}")]
pub(crate) async fn media_video_item(
  server: ServerExtension,
  Path((fingerprint, Ordinal(item))): Path<(Fingerprint, Ordinal)>,
  range: Option<TypedHeader<headers::Range>>,
) -> ServerResult<Resource> {
  block_in_place(|| {
    Ok(
      server
        .media_item(
          fingerprint,
          item,
          MediaType::Video,
          MediaItemResource::Original,
        )?
        .range(range),
    )
  })
}

#[route(GET, "/media/video/{fingerprint}/item/{item}/placeholder")]
pub(crate) async fn media_video_item_placeholder(
  server: ServerExtension,
  Path((fingerprint, Ordinal(item))): Path<(Fingerprint, Ordinal)>,
  range: Option<TypedHeader<headers::Range>>,
) -> ServerResult<Resource> {
  block_in_place(|| {
    Ok(
      server
        .media_item(
          fingerprint,
          item,
          MediaType::Video,
          MediaItemResource::Placeholder,
        )?
        .range(range),
    )
  })
}

#[route(GET, "/media/video/{fingerprint}/item/{item}/placeholder/thumbnail")]
pub(crate) async fn media_video_item_placeholder_thumbnail(
  server: ServerExtension,
  Path((fingerprint, Ordinal(item))): Path<(Fingerprint, Ordinal)>,
  range: Option<TypedHeader<headers::Range>>,
) -> ServerResult<Resource> {
  block_in_place(|| {
    Ok(
      server
        .media_item(
          fingerprint,
          item,
          MediaType::Video,
          MediaItemResource::PlaceholderThumbnail,
        )?
        .range(range),
    )
  })
}

#[route(GET, "/mount/{fingerprint}/")]
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

#[route(GET, "/mount/{fingerprint}/{*path}")]
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

#[route(GET, "/mount/{fingerprint}")]
pub(crate) async fn mount_redirect(Path(fingerprint): Path<Fingerprint>) -> Redirect {
  Redirect::permanent(&format!("/mount/{fingerprint}/"))
}

#[route(GET, "/package/{package}")]
pub(crate) async fn package(
  server: ServerExtension,
  server_config: ServerConfigExtension,
  Path(identifier): Path<PackageIdentifier>,
) -> PageResult<PackageHtml> {
  block_in_place(|| {
    Ok(
      server
        .package_html(identifier, &server_config.mounts)?
        .page(server_config.url.clone()),
    )
  })
}

#[route(GET, "/package/{package}/history")]
pub(crate) async fn package_history(
  server: ServerExtension,
  server_config: ServerConfigExtension,
  Path(identifier): Path<PackageIdentifier>,
) -> PageResult<HistoryHtml> {
  block_in_place(|| {
    Ok(
      server
        .history_html(identifier)?
        .page(server_config.url.clone()),
    )
  })
}

#[route(GET, "/package/{package}/item/{item}")]
pub(crate) async fn package_item(
  server: ServerExtension,
  server_config: ServerConfigExtension,
  Path((identifier, Ordinal(index))): Path<(PackageIdentifier, Ordinal)>,
) -> PageResult<ItemHtml> {
  block_in_place(|| {
    let Resolved { fingerprint, .. } = server.resolve(identifier)?;

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
      media.item_count() > index,
      server_error::MediaItemDoesNotExist {
        count: media.item_count(),
        fingerprint,
        index,
        ty,
      },
    }

    Ok(
      ItemHtml {
        fingerprint,
        identifier,
        index,
        metadata,
      }
      .page(server_config.url.clone()),
    )
  })
}

#[route(GET, "/package/{package}/media")]
pub(crate) async fn package_media(
  server: ServerExtension,
  server_config: ServerConfigExtension,
  Path(identifier): Path<PackageIdentifier>,
) -> PageResult<MediaHtml> {
  block_in_place(|| {
    let Resolved {
      fingerprint,
      number,
      revision,
    } = server.resolve(identifier)?;

    let metadata = server.package_metadata(fingerprint)?;

    ensure! {
      metadata.media.is_some(),
      server_error::PackageMediaMetadataNotFound { fingerprint },
    }

    Ok(
      MediaHtml {
        fingerprint,
        identifier,
        metadata,
        number,
        revision,
      }
      .page(server_config.url.clone()),
    )
  })
}

#[route(GET, "/packages")]
pub(crate) async fn packages(
  server: ServerExtension,
  server_config: ServerConfigExtension,
  Query(query): Query<PackagesQuery>,
) -> PageResult<PackagesHtml> {
  let order = query.order.unwrap_or_default();
  let sort = query.sort.unwrap_or_default();

  block_in_place(|| {
    Ok(
      PackagesHtml {
        order,
        packages: server.packages(sort, order)?,
        sort,
      }
      .page(server_config.url.clone()),
    )
  })
}

#[route(GET, "/static/{*path}")]
pub(crate) async fn static_asset(path: Path<String>) -> ServerResult<StaticAsset> {
  StaticAsset::get(&path)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn endpoint() {
    let mut schemas = BTreeMap::new();

    assert!(api_gc_route(&mut schemas).endpoint.authenticated);
    assert!(!api_file_route(&mut schemas).endpoint.authenticated);

    assert!(api_file_route(&mut schemas).endpoint.range);
    assert!(!api_gc_route(&mut schemas).endpoint.range);

    assert_eq!(
      api_file_route(&mut schemas).endpoint.parameters,
      [crate::reference::Parameter::Hash]
    );

    assert_eq!(
      media_audio_item_route(&mut schemas).endpoint.parameters,
      [
        crate::reference::Parameter::Fingerprint,
        crate::reference::Parameter::Ordinal
      ],
    );

    assert_eq!(api_gc_route(&mut schemas).endpoint.query, []);

    assert_eq!(
      packages_route(&mut schemas).endpoint.query,
      [
        crate::reference::QueryParameter {
          default: "ascending".into(),
          name: "order".into(),
          values: vec!["ascending".into(), "descending".into()],
        },
        crate::reference::QueryParameter {
          default: "title".into(),
          name: "sort".into(),
          values: vec![
            "title".into(),
            "creator".into(),
            "year".into(),
            "media".into(),
            "files".into(),
            "size".into(),
          ],
        },
      ],
    );

    assert_eq!(api_gc_route(&mut schemas).endpoint.request, None);

    assert_eq!(
      api_upload_file_route(&mut schemas).endpoint.request,
      Some(crate::reference::Request::Bytes),
    );

    assert_eq!(
      api_missing_route(&mut schemas).endpoint.request,
      Some(crate::reference::Request::Deco {
        limit: 1_048_576,
        schema: deco::Schema::Struct {
          fields: vec![deco::Field {
            n: 1,
            name: "hashes".into(),
            optional: false,
            schema: "SortedSet<Hash>".into(),
          }],
          magic: None,
          strict: false,
        },
      }),
    );

    assert!(!schemas.contains_key("Request"));
    assert!(schemas.contains_key("SortedSet<Hash>"));

    assert_eq!(
      api_numbers_route(&mut schemas).endpoint.response,
      Some(crate::reference::Response::Deco {
        schema: deco::Schema::Struct {
          fields: vec![deco::Field {
            n: 1,
            name: "numbers".into(),
            optional: false,
            schema: "SortedSet<UnsignedInteger>".into(),
          }],
          magic: None,
          strict: false,
        },
      }),
    );

    assert_eq!(
      api_delete_number_route(&mut schemas).endpoint.response,
      None,
    );

    assert_eq!(
      api_file_route(&mut schemas).endpoint.response,
      Some(crate::reference::Response::File),
    );

    assert_eq!(
      home_route(&mut schemas).endpoint.response,
      Some(crate::reference::Response::Html),
    );

    assert_eq!(
      favicon_route(&mut schemas).endpoint.response,
      Some(crate::reference::Response::Static),
    );

    assert_eq!(
      mount_redirect_route(&mut schemas).endpoint.response,
      Some(crate::reference::Response::Redirect),
    );

    assert_eq!(
      file_with_path_route(&mut schemas).endpoint.response,
      Some(crate::reference::Response::File),
    );

    assert!(!schemas.contains_key("Response"));
    assert!(schemas.contains_key("SortedSet<UnsignedInteger>"));
  }
}
