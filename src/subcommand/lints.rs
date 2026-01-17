use super::*;

#[expect(clippy::unnecessary_wraps)]
pub(crate) fn run() -> Result {
  let groups = LintGroup::iter()
    .map(|group| (group, group.lints()))
    .collect::<BTreeMap<LintGroup, BTreeSet<Lint>>>();

  println!("{}", serde_json::to_string_pretty(&groups).unwrap());

  Ok(())
}
