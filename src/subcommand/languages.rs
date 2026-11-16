use super::*;

#[derive(Parser)]
pub(crate) struct Languages {
  #[arg(long = "format", default_value_t, help = FORMAT_HELP)]
  format: OutputFormat,
}

pub(crate) type Input = ();

pub(crate) type Output = BTreeMap<Language, String>;

impl Languages {
  pub(crate) fn run(self) -> Result {
    let languages = language::ALL
      .iter()
      .map(|language| (*language, language.name().into()))
      .collect::<Output>();

    match self.format {
      OutputFormat::Json => println!("{}", DisplayJson::<Output>(&languages)),
      OutputFormat::JsonPretty => println!("{:#}", DisplayJson::<Output>(&languages)),
    }

    Ok(())
  }
}
