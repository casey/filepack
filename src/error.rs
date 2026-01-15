use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum Error {
  #[snafu(display("file hash {actual} not equal to expected {expected}"))]
  Assert {
    backtrace: Option<Backtrace>,
    actual: Hash,
    expected: Hash,
  },
  #[snafu(display("failed to get current directory"))]
  CurrentDir {
    backtrace: Option<Backtrace>,
    source: io::Error,
  },
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
  #[snafu(display(
    "duplicate key: {}",
    if first == second {
      format!("`{first}`")
    } else {
      format!("`{first}` and `{second}`")
    },
  ))]
  DuplicateKey {
    backtrace: Option<Backtrace>,
    first: KeyIdentifier,
    second: KeyIdentifier,
  },
  #[snafu(display("{count} mismatched file{}", if *count == 1 { "" } else { "s" }))]
  EntryMismatch {
    backtrace: Option<Backtrace>,
    count: usize,
  },
  #[snafu(display("extraneous directory not in manifest: `{path}`"))]
  ExtraneousDirectory {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
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
  #[snafu(display("internal error, this may indicate a bug in filepack: {message}"))]
  Internal {
    backtrace: Option<Backtrace>,
    message: String,
  },
  #[snafu(display("key directory `{path}` has insecure permissions {mode}"))]
  KeyDirPermissions {
    backtrace: Option<Backtrace>,
    mode: Mode,
    path: DisplayPath,
  },
  #[snafu(display("unexpected directory in key directory: `{path}`"))]
  KeyDirUnexpectedDirectory {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
  },
  #[snafu(display("unexpected file in key directory: `{path}`"))]
  KeyDirUnexpectedFile {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
  },
  #[snafu(display(
    "public key `{}` doesn't match private key `{}`",
    key.public_key_filename(),
    key.private_key_filename(),
  ))]
  KeyMismatch {
    backtrace: Option<Backtrace>,
    key: crate::KeyName,
  },
  #[snafu(display("invalid key name: `{path}`"))]
  KeyName {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
    source: PublicKeyError,
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
  #[snafu(display("directory missing: `{path}`"))]
  MissingDirectory {
    backtrace: Option<Backtrace>,
    path: RelativePath,
  },
  #[snafu(display("file missing: `{path}`"))]
  MissingFile {
    backtrace: Option<Backtrace>,
    path: RelativePath,
  },
  #[snafu(display("invalid path `{path}`"))]
  Path {
    path: DisplayPath,
    source: PathError,
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
  #[snafu(display("private key `{path}` has insecure permissions {mode}"))]
  PrivateKeyPermissions {
    backtrace: Option<Backtrace>,
    mode: Mode,
    path: DisplayPath,
  },
  #[snafu(display("public key already exists: `{}`", path.display()))]
  PublicKeyAlreadyExists {
    backtrace: Option<Backtrace>,
    path: PathBuf,
  },
  #[snafu(display("invalid public key: `{path}`"))]
  PublicKeyLoad {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
    source: PublicKeyError,
  },
  #[snafu(display("public key not found: `{path}`"))]
  PublicKeyNotFound {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
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
  #[snafu(display("no signature found for key `{identifier}`"))]
  SignatureMissing {
    backtrace: Option<Backtrace>,
    identifier: KeyIdentifier,
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
