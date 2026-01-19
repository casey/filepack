use super::*;

#[derive(Parser)]
#[group(required = true)]
pub(crate) struct Bech32m {
  #[arg(
    group = "source",
    help = "Convert <BECH32M> to hexadecimal",
    long,
    value_name = "BECH32M"
  )]
  decode: Option<String>,
  #[arg(
    group = "source",
    help = "Encode <HEXADECIMAL> to bech32m",
    long,
    requires = "hrp",
    value_name = "HEXADECIMAL"
  )]
  encode: Option<String>,
  #[arg(help = "Prefix bech32m with human-readable part <HRP>", long)]
  hrp: Option<String>,
  #[arg(help = "Strip or add <VERSION> character", long)]
  version: Option<char>,
}

impl Bech32m {
  pub(crate) fn run(self) -> Result {
    let version = self
      .version
      .map(|version| Fe32::from_char(version).context(error::Bech32mVersion { version }))
      .transpose()?;

    if let Some(bech32m) = self.decode {
      let hrp_string =
        CheckedHrpstring::new::<bech32::Bech32m>(&bech32m).context(error::Bech32mDecode {
          bech32m: bech32m.clone(),
        })?;

      let mut fe32s = hrp_string.fe32_iter::<std::vec::IntoIter<u8>>();

      if let Some(expected) = version {
        let actual = fe32s.next().context(error::Bech32mVersionMissing)?;

        ensure!(
          actual == expected,
          error::Bech32mVersionMismatch { actual, expected },
        );
      }

      let bytes = fe32s.fes_to_bytes().collect::<Vec<u8>>();
      let hex = hex::encode(bytes);
      println!("{hex}");
    } else {
      let hrp = Hrp::parse(&self.hrp.unwrap()).context(error::Bech32mHrp)?;
      let hex = self.encode.unwrap();
      let hex = hex::decode(&hex).context(error::Hex { hex })?;

      let iter = hex
        .iter()
        .copied()
        .bytes_to_fes()
        .with_checksum::<bech32::Bech32m>(&hrp);

      let iter = if let Some(v) = version {
        iter.with_witness_version(v)
      } else {
        iter
      };

      for c in iter.chars() {
        print!("{c}");
      }

      println!();
    }

    Ok(())
  }
}
