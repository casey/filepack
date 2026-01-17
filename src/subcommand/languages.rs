use super::*;

#[expect(clippy::unnecessary_wraps)]
pub(crate) fn run() -> Result {
  println!(
    "{}",
    serde_json::to_string_pretty(&*language::CODES).unwrap()
  );

  Ok(())
}
