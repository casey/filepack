use super::*;

#[test]
fn decode() {
  Test::new()
    .args(["bech32", "--decode", "foo1vehk7e3xjxp"])
    .stdout("666f6f\n")
    .success();
}

#[test]
fn decode_invalid_bech32m() {
  Test::new()
    .args(["bech32", "--decode", "invalid"])
    .stderr_regex("error: failed to decode bech32 `invalid`\n.*")
    .failure();
}

#[test]
fn decode_prefix_missing() {
  Test::new()
    .args(["bech32", "--decode", "foo1vehk7e3xjxp", "--prefix", "q"])
    .stderr("error: bech32 prefix missing\n")
    .failure();
}

#[test]
fn decode_with_prefix() {
  Test::new()
    .args(["bech32", "--decode", "foo1qvehk7ml2uzp", "--prefix", "q"])
    .stdout("666f6f\n")
    .success();
}

#[test]
fn encode() {
  Test::new()
    .args(["bech32", "--encode", "666f6f", "--hrp", "foo"])
    .stdout("foo1vehk7e3xjxp\n")
    .success();
}

#[test]
fn encode_invalid_hex() {
  Test::new()
    .args(["bech32", "--encode", "gg", "--hrp", "foo"])
    .stderr_regex("error: failed to parse hexadecimal `gg`\n.*")
    .failure();
}

#[test]
fn encode_invalid_hrp() {
  Test::new()
    .args(["bech32", "--encode", "666f6f", "--hrp", " "])
    .stderr_regex("error: failed to parse bech32 human-readable part\n.*")
    .failure();
}

#[test]
fn encode_invalid_prefix() {
  Test::new()
    .args([
      "bech32", "--encode", "666f6f", "--hrp", "foo", "--prefix", "!",
    ])
    .stderr_regex("error: invalid bech32 prefix character `!`\n.*")
    .failure();
}

#[test]
fn encode_requires_hrp() {
  Test::new()
    .args(["bech32", "--encode", "666f6f"])
    .stderr_regex("error: the following required arguments were not provided:.*--hrp.*")
    .status(2);
}

#[test]
fn encode_with_prefix() {
  Test::new()
    .args([
      "bech32", "--encode", "666f6f", "--hrp", "foo", "--prefix", "q",
    ])
    .stdout("foo1qvehk7ml2uzp\n")
    .success();
}

#[test]
fn source_required() {
  Test::new()
    .arg("bech32")
    .stderr_regex(
      "error: the following required arguments were not provided:.*--decode.*--encode.*",
    )
    .status(2);
}
