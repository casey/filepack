use super::*;

pub(crate) type Input = ();

pub(crate) type Output = BTreeMap<LintGroup, BTreeSet<Lint>>;

pub(crate) fn run() -> Result {
  let groups = LintGroup::iter()
    .map(|group| (group, group.lints()))
    .collect::<Output>();

  println!("{}", DisplayJson::<Output>(&groups));

  Ok(())
}
