use super::*;

#[derive(Parser)]
pub(crate) struct Metadata {
  #[arg(long = "format", default_value_t)]
  format: Format,
  #[arg(
    help = "Load CBOR metadata from <PATH>. May be path to metadata, to directory containing named \
    `metadata.cbor`, or omitted, in which case metadata named `metadata.cbor` in the current \
    directory is loaded."
  )]
  path: Option<Utf8PathBuf>,
}

impl Metadata {
  pub(crate) fn run(self) -> Result {
    let path = if let Some(path) = self.path {
      if path.is_dir() {
        path.join(crate::Metadata::CBOR_FILENAME)
      } else {
        path
      }
    } else {
      crate::Metadata::CBOR_FILENAME.into()
    };

    let bytes = filesystem::read(&path)?;

    let metadata =
      crate::Metadata::decode_from_slice(&bytes).context(error::DecodeMetadataCbor { path })?;

    match self.format {
      Format::Json => println!("{}", serde_json::to_string(&metadata).unwrap()),
      Format::JsonPretty => println!("{}", serde_json::to_string_pretty(&metadata).unwrap()),
      Format::Tsv => return Err(error::MetadataTsv.build()),
    }

    Ok(())
  }
}
