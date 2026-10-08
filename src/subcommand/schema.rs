use super::*;

pub(crate) fn run() -> Result {
  let schema = Schema::new();

  serde_json::to_writer_pretty(io::stdout(), &schema).context(error::SerializeStdout)?;

  Ok(())
}
