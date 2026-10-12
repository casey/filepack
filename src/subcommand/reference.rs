use super::*;

pub(crate) fn run() -> Result {
  let reference = Reference::new();

  serde_json::to_writer_pretty(io::stdout(), &reference).context(error::SerializeStdout)?;

  Ok(())
}
