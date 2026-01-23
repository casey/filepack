use super::*;

#[test]
fn decode() {
  Test::new()
    .args(["bech32m", "--decode", "foo1vehk7e3xjxp"])
    .stdout("666f6f\n")
    .success();
}

#[test]
fn decode_invalid_bech32m() {
  Test::new()
    .args(["bech32m", "--decode", "invalid"])
    .stderr_regex("error: failed to decode bech32m `invalid`\n.*")
    .failure();
}

#[test]
fn decode_prefix_missing() {
  Test::new()
    .args(["bech32m", "--decode", "foo1vehk7e3xjxp", "--prefix", "q"])
    .stderr("error: bech32m prefix missing\n")
    .failure();
}

#[test]
fn decode_with_prefix() {
  Test::new()
    .args(["bech32m", "--decode", "foo1qvehk7ml2uzp", "--prefix", "q"])
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

#[test]
fn encode_invalid_prefix() {
  Test::new()
    .args([
      "bech32m", "--encode", "666f6f", "--hrp", "foo", "--prefix", "!",
    ])
    .stderr_regex("error: invalid bech32m prefix character `!`\n.*")
    .failure();
}

#[test]
fn encode_requires_hrp() {
  Test::new()
    .args(["bech32m", "--encode", "666f6f"])
    .stderr_regex("error: the following required arguments were not provided:.*--hrp.*")
    .status(2);
}

#[test]
fn encode_with_prefix() {
  Test::new()
    .args([
      "bech32m", "--encode", "666f6f", "--hrp", "foo", "--prefix", "q",
    ])
    .stdout("foo1qvehk7ml2uzp\n")
    .success();
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
