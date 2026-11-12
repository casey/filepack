use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum MetadataError {
  #[snafu(display("artwork `{path}` is {dimensions} but must be square"))]
  ArtworkAspectRatio {
    dimensions: Dimensions,
    path: DisplayPath,
  },
  #[snafu(display("colophon `{colophon}` must end in `.md`"))]
  ColophonExtension { colophon: RelativePath },
  #[snafu(display("failed to decode metadata at `{path}`"))]
  DecodeMetadataDeco {
    path: DisplayPath,
    source: DecodeError,
  },
  #[snafu(display("duplicate item in metadata: `{path}`"))]
  DuplicateItem { path: RelativePath },
  #[snafu(display("found {} not referenced in metadata", Count::new(paths.len(), "extra file")))]
  ExtraFiles { paths: Vec<RelativePath> },
  #[snafu(transparent)]
  Filesystem { source: FilesystemError },
  #[snafu(display("file referenced in metadata missing: `{path}`"))]
  MissingMetadataFile { path: RelativePath },
  #[snafu(display("readme `{readme}` must end in `.md`"))]
  ReadmeExtension { readme: RelativePath },
  #[snafu(display("thumbnail for `{image}` conflicts with `{path}`"))]
  ThumbnailAlreadyExists {
    image: RelativePath,
    path: DisplayPath,
  },
  #[snafu(display("thumbnail for `{second}` conflicts with thumbnail for `{first}`"))]
  ThumbnailCollision {
    first: RelativePath,
    second: RelativePath,
  },
}
