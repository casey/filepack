use super::*;

#[derive(Parser)]
pub(crate) struct Metadata {
  #[arg(long = "format", default_value_t)]
  format: Format,
  #[arg(help = "Path to metadata.cbor or directory containing it")]
  path: Option<Utf8PathBuf>,
}

impl Metadata {
  pub(crate) fn run(self) -> Result {
    let path = if let Some(path) = self.path {
      if path.is_dir() {
        path.join("metadata.cbor")
      } else {
        path
      }
    } else {
      Utf8PathBuf::from("metadata.cbor")
    };

    let bytes = filesystem::read(&path)?;

    let mut decoder = Decoder::new(bytes);

    let metadata = crate::metadata::Metadata::decode(&mut decoder).map_err(|err| {
      error::DecodeMetadataCbor {
        message: err.to_string(),
        path: &path,
      }
      .build()
    })?;

    decoder.finish().map_err(|err| {
      error::DecodeMetadataCbor {
        message: err.to_string(),
        path: &path,
      }
      .build()
    })?;

    match self.format {
      Format::Json => println!("{}", serde_json::to_string(&metadata).unwrap()),
      Format::JsonPretty => println!("{}", serde_json::to_string_pretty(&metadata).unwrap()),
      Format::Tsv => return Err(error::MetadataTsv.build()),
    }

    Ok(())
  }
}
