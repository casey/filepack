use super::*;

pub(crate) type Input = ();

pub(crate) type Output = Reference;

pub(crate) fn run() -> Result {
  let reference = Reference::new();

  println!("{}", DisplayJson::<Output>(&reference));

  Ok(())
}
