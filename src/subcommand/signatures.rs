use super::*;

#[derive(Parser)]
pub(crate) struct Signatures {
  #[arg(long = "format", default_value_t)]
  format: OutputFormat,
  #[arg(help = MANIFEST_PATH_HELP)]
  path: Option<Utf8PathBuf>,
}

impl Signatures {
  pub(crate) fn run(self) -> Result {
    let loader = Loader::load(self.path.as_deref())?;

    let fingerprint = loader.fingerprint()?;

    let manifest = loader.unpack()?;

    let signatures = manifest
      .signatures()
      .map(|signature| {
        Ok(SignatureEntry {
          public_key: signature.public_key(),
          timestamp: signature.verify(fingerprint)?.timestamp,
        })
      })
      .collect::<Result<Vec<SignatureEntry>>>()?;

    let unknown = manifest.unknown_signatures();

    if unknown > 0 {
      eprintln!("ignored {}", Count::new(unknown, "unrecognized signature"));
    }

    match self.format {
      OutputFormat::Json => println!("{}", serde_json::to_string(&signatures).unwrap()),
      OutputFormat::JsonPretty => {
        println!("{}", serde_json::to_string_pretty(&signatures).unwrap());
      }
      OutputFormat::Tsv => {
        for signature in &signatures {
          let timestamp = signature
            .timestamp
            .map(|t| t.to_string())
            .unwrap_or_default();
          println!("{}\t{}", signature.public_key, timestamp);
        }
      }
    }

    Ok(())
  }
}
