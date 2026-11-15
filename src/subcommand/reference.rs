use super::*;

pub(crate) type Input = ();

pub(crate) type Output = ();

pub(crate) fn run() -> Result {
  let reference = Reference::new();

  println!("{}", serde_json::to_string_pretty(&reference).unwrap());

  Ok(())
}
