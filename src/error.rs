use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub(crate) enum Error {
  #[snafu(display("unexpected end of archive"))]
  ArchiveTruncated { path: PathBuf, source: io::Error },
  #[snafu(display("failed to get current directory"))]
  CurrentDir { source: io::Error },
  #[snafu(display("failed to get local data directory"))]
  DataLocalDir { backtrace: Option<Backtrace> },
  #[snafu(display("failed to deserialize manifest at `{path}`"))]
  DeserializeManifest {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
    source: serde_json::Error,
  },
  #[snafu(display("failed to deserialize metadata at `{path}`"))]
  DeserializeMetadata {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
    source: serde_json::Error,
  },
  #[snafu(display("failed to deserialize metadata template at `{path}`"))]
  DeserializeMetadataTemplate {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
    source: serde_yaml::Error,
  },
  #[snafu(
    display(
      "empty director{} {}",
      if paths.len() == 1 { "y" } else { "ies" },
      List::and_ticked(paths),
    )
  )]
  EmptyDirectory { paths: Vec<DisplayPath> },
  #[snafu(display("{count} mismatched file{}", if *count == 1 { "" } else { "s" }))]
  EntryMismatch {
    backtrace: Option<Backtrace>,
    count: usize,
  },
  #[snafu(display("extraneous file not in manifest: `{path}`"))]
  ExtraneousFile {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
  },
  #[snafu(display("I/O error at `{path}`"))]
  FilesystemIo {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
    source: io::Error,
  },
  #[snafu(display("fingerprint mismatch"))]
  FingerprintMismatch { backtrace: Option<Backtrace> },
  #[snafu(display("file `{path}` hash {actual} does not match manifest hash {expected}"))]
  HashMismatch {
    actual: Hash,
    backtrace: Option<Backtrace>,
    expected: Hash,
    path: DisplayPath,
  },
  #[snafu(display("public key `{public_key}` doesn't match private key `{private_key}`"))]
  KeyMismatch {
    backtrace: Option<Backtrace>,
    public_key: DisplayPath,
    private_key: DisplayPath,
  },
  #[snafu(display("{count} lint error{}", if *count == 1 { "" } else { "s" }))]
  Lint {
    backtrace: Option<Backtrace>,
    count: u64,
  },
  #[snafu(display("manifest `{path}` already exists"))]
  ManifestAlreadyExists {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
  },
  #[snafu(display("manifest `{path}` not found"))]
  ManifestNotFound {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
  },
  #[snafu(display("metadata `{path}` already exists"))]
  MetadataAlreadyExists {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
  },
  #[snafu(display("metadata template `{path}` should not be included in package"))]
  MetadataTemplateIncluded {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
  },
  #[snafu(display("file missing: `{path}`"))]
  MissingFile {
    backtrace: Option<Backtrace>,
    path: RelativePath,
  },
  #[snafu(display("invalid path `{path}`"))]
  Path {
    path: DisplayPath,
    source: relative_path::Error,
  },
  #[snafu(display("path not valid unicode: `{}`", path.display()))]
  PathUnicode {
    backtrace: Option<Backtrace>,
    path: PathBuf,
  },
  #[snafu(display("private key already exists: `{}`", path.display()))]
  PrivateKeyAlreadyExists {
    backtrace: Option<Backtrace>,
    path: PathBuf,
  },
  #[snafu(display("invalid private key `{path}`"))]
  PrivateKeyLoad {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
    source: private_key::Error,
  },
  #[snafu(display("private key not found: `{path}`"))]
  PrivateKeyNotFound {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
  },
  #[snafu(display("public key already exists: `{}`", path.display()))]
  PublicKeyAlreadyExists {
    backtrace: Option<Backtrace>,
    path: PathBuf,
  },
  #[snafu(display("invalid public key `{path}`"))]
  PublicKeyLoad {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
    source: public_key::Error,
  },
  #[snafu(display("public key not found: `{path}`"))]
  PublicKeyNotFound {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
  },
  #[snafu(display("failed to bind server to `{address}:{port}`"))]
  ServerBind {
    address: String,
    backtrace: Option<Backtrace>,
    port: u16,
    source: io::Error,
  },
  #[snafu(display("failed to initialize server runtime"))]
  ServerRuntime {
    backtrace: Option<Backtrace>,
    source: io::Error,
  },
  #[snafu(display("failed to serve server"))]
  ServerServe {
    backtrace: Option<Backtrace>,
    source: io::Error,
  },
  #[snafu(display("manifest has already been signed by public key `{public_key}`"))]
  SignatureAlreadyExists {
    backtrace: Option<Backtrace>,
    public_key: PublicKey,
  },
  #[snafu(display("invalid signature for public key `{public_key}`"))]
  SignatureInvalid {
    backtrace: Option<Backtrace>,
    public_key: PublicKey,
    source: SignatureError,
  },
  #[snafu(display("no signature found for key {key}"))]
  SignatureMissing {
    backtrace: Option<Backtrace>,
    key: PublicKey,
  },
  #[snafu(display("file `{path}` size {actual} does not match manifest size {expected}"))]
  SizeMismatch {
    actual: u64,
    backtrace: Option<Backtrace>,
    expected: u64,
    path: DisplayPath,
  },
  #[snafu(display("I/O error reading standard input"))]
  StandardInputIo {
    backtrace: Option<Backtrace>,
    source: io::Error,
  },
  #[snafu(display("symlink at `{path}`"))]
  Symlink {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
  },
  #[snafu(transparent)]
  WalkDir {
    backtrace: Option<Backtrace>,
    source: walkdir::Error,
  },
}
