use super::*;

#[test]
fn schema() {
  Test::new()
    .arg("schema")
    .stdout_check(|stdout| {
      let schema = serde_json::from_str::<Schema>(stdout).unwrap();
      assert_eq!(schema.commands.name, "filepack");
      assert!(schema.commands.subcommands.contains_key("schema"));
    })
    .success();
}
