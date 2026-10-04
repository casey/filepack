use super::*;

#[derive(Parser)]
pub(crate) struct Languages {
  #[arg(long = "format", default_value_t)]
  format: OutputFormat,
}

impl Languages {
  #[expect(clippy::unnecessary_wraps)]
  pub(crate) fn run(self) -> Result {
    let codes = &*language::CODES;

    match self.format {
      OutputFormat::Json => println!("{}", serde_json::to_string(codes).unwrap()),
      OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(codes).unwrap()),
      OutputFormat::Tsv => {
        for (code, language) in codes {
          println!("{code}\t{language}");
        }
      }
    }

    Ok(())
  }
}
