use super::*;

#[test]
fn decode() {
  Test::new()
    .args(["bech32m", "--decode", "foo1vehk7e3xjxp"])
    .stdout("666f6f\n")
    .success();
}

#[test]
fn encode() {
  Test::new()
    .args(["bech32m", "--encode", "666f6f", "--hrp", "foo"])
    .stdout("foo1vehk7e3xjxp\n")
    .success();
}

#[test]
fn encode_requires_hrp() {
  Test::new()
    .args(["bech32m", "--encode", "666f6f"])
    .stderr_regex("error: the following required arguments were not provided:.*--hrp.*")
    .status(2);
}

#[test]
fn source_required() {
  Test::new()
    .arg("bech32m")
    .stderr_regex(
      "error: the following required arguments were not provided:.*--decode.*--encode.*",
    )
    .status(2);
}

#[test]
fn decode_invalid_bech32m() {
  Test::new()
    .args(["bech32m", "--decode", "invalid"])
    .stderr_regex("error: failed to decode bech32m `invalid`\n.*")
    .failure();
}

#[test]
fn encode_invalid_hex() {
  Test::new()
    .args(["bech32m", "--encode", "gg", "--hrp", "foo"])
    .stderr_regex("error: failed to parse hexadecimal `gg`\n.*")
    .failure();
}

#[test]
fn encode_invalid_hrp() {
  Test::new()
    .args(["bech32m", "--encode", "666f6f", "--hrp", " "])
    .stderr_regex("error: failed to parse bech32m human-readable part\n.*")
    .failure();
}
