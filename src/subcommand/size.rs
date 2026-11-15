use super::*;

#[derive(Parser)]
pub(crate) struct Size {
  #[arg(help = MANIFEST_PATH_HELP, value_hint = ValueHint::AnyPath)]
  path: Option<Utf8PathBuf>,
}

pub(crate) type Input = ();

pub(crate) type Output = Totals;

impl Size {
  pub(crate) fn run(self) -> Result {
    let (_manifest, totals) = Loader::load(self.path.as_deref())?.unpack_with_totals()?;

    println!(
      "{}",
      serde_json::to_string_pretty::<Output>(&totals).unwrap()
    );

    Ok(())
  }
}
