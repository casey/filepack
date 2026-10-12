use super::*;

#[test]
fn reference() {
  Test::new()
    .arg("reference")
    .stdout_check(|stdout| {
      let reference = serde_json::from_str::<Reference>(stdout).unwrap();
      assert_eq!(reference.commands.name, "filepack");
      assert!(reference.commands.subcommands.contains_key("reference"));
    })
    .success();
}
