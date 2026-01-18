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
    source: serde_yaml::Error,
  },
  #[snafu(display("unknown fields in metadata at `{path}`: {unknown}"))]
  DeserializeMetadataStrict {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
    unknown: Ticked,
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
  #[snafu(display("note {first} and {second} have the same digest"))]
  DuplicateNote {
    backtrace: Option<Backtrace>,
    first: Index,
    second: Index,
  },
  #[snafu(display("note {first} and {second} both have signatures from key {key}"))]
  DuplicateSignature {
    backtrace: Option<Backtrace>,
    first: Index,
    key: PublicKey,
    second: Index,
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
  #[snafu(display("keychain directory `{path}` has insecure permissions {mode}"))]
  KeychainPermissions {
    backtrace: Option<Backtrace>,
    mode: Mode,
    path: DisplayPath,
  },
  #[snafu(display("unexpected directory in keychain directory: `{path}`"))]
  KeychainUnexpectedDirectory {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
  },
  #[snafu(display("unexpected file in keychain directory: `{path}`"))]
  KeychainUnexpectedFile {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
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
  #[snafu(display("cannot use `--manifest` when `{path}` exists"))]
  ManifestInPackage {
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
  #[snafu(display("file referenced in metadata missing: `{filename}`"))]
  MissingMetadataFile {
    backtrace: Option<Backtrace>,
    filename: RelativePath,
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
  #[snafu(display("manifest has already been signed by key `{key}`"))]
  SignatureAlreadyExists {
    backtrace: Option<Backtrace>,
    key: PublicKey,
  },
  #[snafu(display("invalid signature for key `{key}`"))]
  SignatureInvalid {
    backtrace: Option<Backtrace>,
    key: PublicKey,
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
  #[snafu(display("Failed to get curent time"))]
  Time {
    backtrace: Option<Backtrace>,
    source: SystemTimeError,
  },
  #[snafu(display("note {index} is unsigned"))]
  UnsignedNote {
    backtrace: Option<Backtrace>,
    index: Index,
  },
  #[snafu(transparent)]
  WalkDir {
    backtrace: Option<Backtrace>,
    source: walkdir::Error,
  },
}
