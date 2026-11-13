use super::*;

pub(crate) type Input = ();

pub(crate) type Output = BTreeMap<LintGroup, BTreeSet<Lint>>;

#[expect(clippy::unnecessary_wraps)]
pub(crate) fn run() -> Result {
  let groups = LintGroup::iter()
    .map(|group| (group, group.lints()))
    .collect::<Output>();

  println!("{}", serde_json::to_string_pretty(&groups).unwrap());

  Ok(())
}
