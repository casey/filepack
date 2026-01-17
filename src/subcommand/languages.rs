use super::*;

#[derive(Parser)]
pub(crate) struct Languages {
  #[arg(long = "format", default_value_t)]
  format: Format,
}

impl Languages {
  #[expect(clippy::unnecessary_wraps)]
  pub(crate) fn run(self) -> Result {
    let codes = &*crate::language::CODES;

    match self.format {
      Format::Json => println!("{}", serde_json::to_string(codes).unwrap()),
      Format::JsonPretty => println!("{}", serde_json::to_string_pretty(codes).unwrap()),
      Format::Tsv => {
        for (code, language) in codes {
          println!("{code}\t{language}");
        }
      }
    }

    Ok(())
  }
}
