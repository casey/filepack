use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum Error {
  #[snafu(display("artwork `{path}` is {dimensions} but must be square"))]
  ArtworkDimensions {
    backtrace: Option<Backtrace>,
    dimensions: Dimensions,
    path: DisplayPath,
  },
  #[snafu(display("file hash {actual} not equal to expected {expected}"))]
  Assert {
    actual: Hash,
    backtrace: Option<Backtrace>,
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
    actual: bech32::Fe32,
    backtrace: Option<Backtrace>,
    expected: bech32::Fe32,
  },
  #[snafu(display("bech32 version character missing"))]
  Bech32VersionMissing { backtrace: Option<Backtrace> },
  #[snafu(display("failed to bind listener to {address}"))]
  BindListener {
    address: String,
    backtrace: Option<Backtrace>,
    source: io::Error,
  },
  #[snafu(display("failed to build HTTP client"))]
  ClientBuild {
    backtrace: Option<Backtrace>,
    source: reqwest::Error,
  },
  #[snafu(display("failed to get current directory"))]
  CurrentDir {
    backtrace: Option<Backtrace>,
    source: io::Error,
  },
  #[snafu(display("failed to get local data directory"))]
  DataLocalDir { backtrace: Option<Backtrace> },
  #[snafu(display("failed to commit to database"))]
  DatabaseCommit {
    backtrace: Option<Backtrace>,
    source: redb::CommitError,
  },
  #[snafu(display("error opening database at `{path}`"))]
  DatabaseOpen {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
    source: redb::DatabaseError,
  },
  #[snafu(display("database schema version `{actual}` does not match expected `{expected}`"))]
  DatabaseSchemaVersionMismatch {
    actual: u64,
    backtrace: Option<Backtrace>,
    expected: u64,
  },
  #[snafu(display("database schema version missing"))]
  DatabaseSchemaVersionMissing { backtrace: Option<Backtrace> },
  #[snafu(display("database storage error"))]
  DatabaseStorage {
    backtrace: Option<Backtrace>,
    source: redb::StorageError,
  },
  #[snafu(display("failed to open database table"))]
  DatabaseTableOpen {
    backtrace: Option<Backtrace>,
    source: redb::TableError,
  },
  #[snafu(display("database transaction error"))]
  DatabaseTransaction {
    backtrace: Option<Backtrace>,
    source: redb::TransactionError,
  },
  #[snafu(display("failed to decode manifest at `{path}`"))]
  DecodeManifest {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
    source: DecodeError,
  },
  #[snafu(display("failed to decode metadata at `{path}`"))]
  DecodeMetadataCbor {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
    source: DecodeError,
  },
  #[snafu(display("failed to decode response from `{url}`"))]
  DecodeResponse {
    backtrace: Option<Backtrace>,
    source: DecodeError,
    url: Url,
  },
  #[snafu(display("failed to decode downloaded directory from `{url}`"))]
  DecodeResponseDirectory {
    backtrace: Option<Backtrace>,
    source: DecodeError,
    url: Url,
  },
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
  #[snafu(display("downloaded file hash mismatch: expected {expected} but got {actual}"))]
  DownloadHashMismatch {
    actual: Hash,
    backtrace: Option<Backtrace>,
    expected: Hash,
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
  #[snafu(display("`{path}` already exists"))]
  FileAlreadyExists {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
  },
  #[snafu(display("file did not match manifest entry"))]
  FileMismatch {
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
  #[snafu(display("failed to decode JPEG image `{path}`"))]
  ImageDecodeJpeg {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
    source: zune_jpeg::errors::DecodeErrors,
  },
  #[snafu(display("failed to decode PNG image `{path}`"))]
  ImageDecodePng {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
    source: png::DecodingError,
  },
  #[snafu(display("image `{path}` is {actual} but metadata dimensions are {expected}"))]
  ImageDimensionsMismatch {
    actual: Dimensions,
    backtrace: Option<Backtrace>,
    expected: Dimensions,
    path: DisplayPath,
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
  #[snafu(display("failed to convert tokio socket into standard socket"))]
  ListenerIntoStandard {
    backtrace: Option<Backtrace>,
    source: io::Error,
  },
  #[snafu(display("failed to get socket address"))]
  LocalAddress {
    backtrace: Option<Backtrace>,
    source: io::Error,
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
  #[snafu(display("manifest cannot be formatted as TSV"))]
  ManifestTsv { backtrace: Option<Backtrace> },
  #[snafu(display("metadata `{path}` already exists"))]
  MetadataAlreadyExists {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
  },
  #[snafu(display("metadata cannot be formatted as TSV"))]
  MetadataTsv { backtrace: Option<Backtrace> },
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
    source: PrivateKeyError,
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
  #[snafu(display("readme `{readme}` must end in `.md`",))]
  ReadmeExtension {
    backtrace: Option<Backtrace>,
    readme: ComponentBuf,
  },
  #[snafu(display("failed to write listening port to `{address}`"))]
  ReadyAddress {
    address: SocketAddr,
    backtrace: Option<Backtrace>,
    source: io::Error,
  },
  #[snafu(display("redirect domain `{domain}` is the canonical domain"))]
  RedirectDomainCanonical {
    backtrace: Option<Backtrace>,
    domain: String,
  },
  #[snafu(display("request failed"))]
  Request {
    backtrace: Option<Backtrace>,
    source: reqwest::Error,
  },
  #[snafu(display("failed to read body from response from {url}"))]
  ResponseBody {
    backtrace: Option<Backtrace>,
    source: reqwest::Error,
    url: Url,
  },
  #[snafu(display("response from {url} failed with status {status}: {body}"))]
  ResponseStatus {
    backtrace: Option<Backtrace>,
    body: String,
    status: StatusCode,
    url: Url,
  },
  #[snafu(display("failed to install rustls ring crypto provider"))]
  RustlsProvider { backtrace: Option<Backtrace> },
  #[snafu(display("server failed"))]
  Serve {
    backtrace: Option<Backtrace>,
    source: io::Error,
  },
  #[snafu(display("failed to build server runtime"))]
  ServerRuntime {
    backtrace: Option<Backtrace>,
    source: io::Error,
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
  #[snafu(display("failed to get current time"))]
  Time {
    backtrace: Option<Backtrace>,
    source: SystemTimeError,
  },
  #[snafu(display("authentication tokens may only be used over HTTPS or loopback"))]
  TokenOverHttp { backtrace: Option<Backtrace> },
  #[snafu(display("failed to decode FLAC track `{path}`"))]
  TrackDecode {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
    source: claxon::Error,
  },
  #[snafu(display("FLAC track `{path}` has empty title"))]
  TrackTitleEmpty {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
  },
  #[snafu(display("FLAC track `{path}` has multiple titles"))]
  TrackTitleMultiple {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
  },
  #[snafu(display("failed to unarchive manifest"))]
  UnarchiveManifest {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
    source: ArchiveError,
  },
  #[snafu(display("manifest `{path}` contains unexpected embedded files: {unexpected}"))]
  UnexpectedEmbeddedFiles {
    backtrace: Option<Backtrace>,
    path: DisplayPath,
    unexpected: Ticked<RelativePath>,
  },
  #[snafu(display("error walking directory"))]
  WalkDir {
    backtrace: Option<Backtrace>,
    source: walkdir::Error,
  },
}

impl From<redb::CommitError> for Error {
  fn from(source: redb::CommitError) -> Self {
    DatabaseCommit {}.into_error(source)
  }
}

impl From<redb::StorageError> for Error {
  fn from(source: redb::StorageError) -> Self {
    DatabaseStorage {}.into_error(source)
  }
}

impl From<redb::TableError> for Error {
  fn from(source: redb::TableError) -> Self {
    DatabaseTableOpen {}.into_error(source)
  }
}

impl From<redb::TransactionError> for Error {
  fn from(source: redb::TransactionError) -> Self {
    DatabaseTransaction {}.into_error(source)
  }
}

impl From<walkdir::Error> for Error {
  fn from(source: walkdir::Error) -> Self {
    WalkDir {}.into_error(source)
  }
}
