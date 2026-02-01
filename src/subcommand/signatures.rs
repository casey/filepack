use super::*;

#[derive(Parser)]
pub(crate) struct Signatures {
  #[arg(long = "format", default_value_t)]
  format: Format,
  #[arg(help = MANIFEST_PATH_HELP)]
  path: Option<Utf8PathBuf>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct Output {
  public_key: PublicKey,
  timestamp: Option<u64>,
}

impl Signatures {
  pub(crate) fn run(self) -> Result {
    let manifest = Manifest::load(self.path.as_deref())?;

    let signatures = manifest
      .signatures
      .iter()
      .map(|signature| Output {
        public_key: signature.public_key(),
        timestamp: signature.message().time,
      })
      .collect::<Vec<Output>>();

    match self.format {
      Format::Json => println!("{}", serde_json::to_string(&signatures).unwrap()),
      Format::JsonPretty => println!("{}", serde_json::to_string_pretty(&signatures).unwrap()),
      Format::Tsv => {
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
