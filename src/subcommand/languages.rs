use super::*;

pub(crate) type Input = ();

pub(crate) type Output = BTreeMap<Language, String>;

pub(crate) fn run() -> Result {
  let languages = language::ALL
    .iter()
    .map(|language| (*language, language.name().into()))
    .collect::<Output>();

  DisplayJson::<Output>::print(&languages)?;

  Ok(())
}
