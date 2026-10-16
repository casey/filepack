use super::*;

#[derive(Parser)]
pub(crate) struct Languages {
  #[arg(long = "format", default_value_t)]
  format: OutputFormat,
}

pub(crate) type Input = ();

pub(crate) type Output = BTreeMap<Language, String>;

impl Languages {
  #[expect(clippy::unnecessary_wraps)]
  pub(crate) fn run(self) -> Result {
    let languages = language::ALL
      .iter()
      .map(|language| (*language, language.name().into()))
      .collect::<Output>();

    match self.format {
      OutputFormat::Json => println!("{}", serde_json::to_string(&languages).unwrap()),
      OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&languages).unwrap()),
      OutputFormat::Tsv => {
        for (code, language) in &languages {
          println!("{code}\t{language}");
        }
      }
    }

    Ok(())
  }
}
