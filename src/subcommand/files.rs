use super::*;

#[derive(Parser)]
pub(crate) struct Files {
  #[arg(help = MANIFEST_PATH_HELP, value_hint = ValueHint::AnyPath)]
  path: Option<Utf8PathBuf>,
}

pub(crate) type Input = ();

pub(crate) type Output = BTreeMap<RelativePath, File>;

impl Files {
  pub(crate) fn run(self) -> Result {
    let manifest = Manifest::load(self.path.as_deref())?;

    let files = manifest.files();

    println!("{}", DisplayJson::<Output>(&files));

    Ok(())
  }
}
