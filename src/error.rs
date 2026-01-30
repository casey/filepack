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
  #[snafu(display("failed to decode bech32 `{bech32}`"))]
  Bech32Decode {
    backtrace: Option<Backtrace>,
    bech32: String,
    source: CheckedHrpstringError,
  },
  #[snafu(display("failed to encode bech32"))]
  Bech32Encode {
    backtrace: Option<Backtrace>,
    source: bech32::EncodeError,
  },
  #[snafu(display("failed to parse bech32 human-readable part"))]
  Bech32Hrp {
    backtrace: Option<Backtrace>,
    source: bech32::primitives::hrp::Error,
  },
  #[snafu(display("invalid bech32 prefix character `{character}`"))]
  Bech32Prefix {
    backtrace: Option<Backtrace>,
    character: char,
    source: bech32::primitives::gf32::FromCharError,
  },
  #[snafu(display("bech32 prefix missing"))]
  Bech32PrefixMissing { backtrace: Option<Backtrace> },
  #[snafu(display("invalid bech32 version character `{version}`"))]
  Bech32Version {
    backtrace: Option<Backtrace>,
    source: bech32::primitives::gf32::FromCharError,
    version: char,
  },
  #[snafu(display("bech32 version `{actual}` does not match expected `{expected}`"))]
  Bech32VersionMismatch {
    backtrace: Option<Backtrace>,
    actual: bech32::Fe32,
    expected: bech32::Fe32,
  },
  #[snafu(display("bech32 version character missing"))]
  Bech32VersionMissing { backtrace: Option<Backtrace> },
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
  #[snafu(display("manifest does not contain file with hash `{hash}`"))]
  FileNotFound {
    backtrace: Option<Backtrace>,
    hash: Hash,
  },
  #[snafu(display("I/O error at `{path}`"))]
  FilesystemIo {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
    source: io::Error,
  },
  #[snafu(display("fingerprint mismatch"))]
  FingerprintMismatch { backtrace: Option<Backtrace> },
  #[snafu(display("failed to parse hexadecimal `{hex}`"))]
  Hex {
    backtrace: Option<Backtrace>,
    hex: String,
    source: hex::FromHexError,
  },
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
    source: Bech32Error,
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
  #[snafu(display(
    "signature fingerprint `{signature}` does not match package fingerprint `{package}`"
  ))]
  SignatureFingerprintMismatch {
    backtrace: Option<Backtrace>,
    package: Fingerprint,
    signature: Fingerprint,
  },
  #[snafu(display("invalid signature for key `{public_key}`"))]
  SignatureInvalid {
    backtrace: Option<Backtrace>,
    public_key: PublicKey,
    source: DalekSignatureError,
  },
  #[snafu(display("no signature found for key `{identifier}`"))]
  SignatureMissing {
    backtrace: Option<Backtrace>,
    identifier: KeyIdentifier,
  },
  #[snafu(display(
    "file with hash `{hash}` has size {manifest} in manifest but size {disk} on disk"
  ))]
  SizeMismatch {
    backtrace: Option<Backtrace>,
    disk: u64,
    hash: Hash,
    manifest: u64,
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
  #[snafu(transparent)]
  WalkDir {
    backtrace: Option<Backtrace>,
    source: walkdir::Error,
  },
}
