use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum ManifestError {
  #[snafu(display("failed to decode manifest"))]
  Decode { source: DecodeError },
  #[snafu(display("failed to decode manifest at `{path}`"))]
  DecodeManifest {
    path: DisplayPath,
    source: DecodeError,
  },
  #[snafu(display("failed to deserialize manifest at `{path}`"))]
  DeserializeManifest {
    path: DisplayPath,
    source: serde_json::Error,
  },
  #[snafu(display(
    "embedded file in manifest `{path}` should have hash `{expected}` but has hash `{actual}`"
  ))]
  EmbeddedFileHashMismatch {
    actual: Hash,
    expected: Hash,
    path: DisplayPath,
  },
  #[snafu(display(
    "embedded file `{hash}` in manifest `{path}` should have size {expected} but has size {actual}"
  ))]
  EmbeddedFileSizeMismatch {
    actual: u64,
    expected: u64,
    hash: Hash,
    path: DisplayPath,
  },
  #[snafu(transparent)]
  Filesystem { source: FilesystemError },
  #[snafu(display("internal error, this may indicate a bug in Filepack: {message}"))]
  Internal { message: String },
  #[snafu(display("manifest `{path}` not found"))]
  ManifestNotFound { path: DisplayPath },
  #[snafu(display("manifest `{path}` totals error"))]
  ManifestTotals {
    path: DisplayPath,
    source: TotalsError,
  },
  #[snafu(display("package contains reserved filename `{}` at `{path}`", path.filename()))]
  ReservedFilename { path: RelativePath },
  #[snafu(display("manifest totals error"))]
  Totals { source: TotalsError },
  #[snafu(display("failed to unarchive manifest"))]
  Unarchive { source: ArchiveError },
  #[snafu(display("failed to unarchive manifest"))]
  UnarchiveManifest {
    path: DisplayPath,
    source: ArchiveError,
  },
  #[snafu(display("manifest `{path}` unknown entry `{name}` conflicts with known entry"))]
  UnknownEntryConflict {
    name: ComponentBuf,
    path: DisplayPath,
  },
  #[snafu(display("manifest `{path}` contains unreferenced embedded files: {hashes}"))]
  UnreferencedEmbeddedFiles {
    hashes: Ticked<Hash>,
    path: DisplayPath,
  },
}
