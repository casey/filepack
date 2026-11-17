use super::*;

#[test]
fn revision() {
  Test::new()
    .arg("create")
    .success()
    .arg("revision")
    .stdout("revision12c7922354e5c2496ffc5e87ee0b29cc8767dc35a7c0e31e9d239014df5b44583\n")
    .success();
}
