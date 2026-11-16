use super::*;

#[derive(Parser)]
pub(crate) struct Signatures {
  #[arg(help = MANIFEST_PATH_HELP, value_hint = ValueHint::AnyPath)]
  path: Option<Utf8PathBuf>,
}

pub(crate) type Input = ();

pub(crate) type Output = BTreeSet<PublicKey>;

impl Signatures {
  pub(crate) fn run(self, options: Options) -> Result {
    let loader = Loader::load(self.path.as_deref())?;

    let fingerprint = loader.fingerprint()?;

    let manifest = loader.unpack()?;

    let signatures = manifest
      .signatures()
      .map(|signature| {
        signature.verify(fingerprint)?;
        Ok(signature.public_key())
      })
      .collect::<Result<Output>>()?;

    let unknown = manifest.unknown_signatures();

    if unknown > 0 && !options.quiet {
      eprintln!("ignored {}", Count::new(unknown, "unrecognized signature"));
    }

    println!("{}", DisplayJson::<Output>(&signatures));

    Ok(())
  }
}
