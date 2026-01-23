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
  #[arg(help = "Strip or add <PREFIX> characters", long)]
  prefix: Option<String>,
}

impl Bech32m {
  pub(crate) fn run(self) -> Result {
    let prefix = self
      .prefix
      .map(|prefix| {
        prefix
          .chars()
          .map(|character| {
            Fe32::from_char(character).context(error::Bech32mVersion { version: character })
          })
          .collect::<Result<Vec<Fe32>>>()
      })
      .transpose()?;

    if let Some(bech32m) = self.decode {
      let hrp_string =
        CheckedHrpstring::new::<bech32::Bech32m>(&bech32m).context(error::Bech32mDecode {
          bech32m: bech32m.clone(),
        })?;

      let mut fe32s = hrp_string
        .fe32_iter::<std::vec::IntoIter<u8>>()
        .collect::<Vec<Fe32>>();

      if let Some(prefix) = prefix {
        ensure! {
          fe32s.starts_with(&prefix),
          error::Bech32mPrefixMissing,
        }

        fe32s.drain(..prefix.len());
      }

      let bytes = fe32s.into_iter().fes_to_bytes().collect::<Vec<u8>>();
      let hex = hex::encode(bytes);
      println!("{hex}");
    } else {
      let hrp = Hrp::parse(&self.hrp.unwrap()).context(error::Bech32mHrp)?;
      let hex = self.encode.unwrap();
      let hex = hex::decode(&hex).context(error::Hex { hex })?;

      let fe32s = if let Some(prefix) = prefix {
        prefix
          .into_iter()
          .chain(hex.iter().copied().bytes_to_fes())
          .collect::<Vec<Fe32>>()
      } else {
        hex.iter().copied().bytes_to_fes().collect::<Vec<Fe32>>()
      };

      for c in fe32s
        .into_iter()
        .with_checksum::<bech32::Bech32m>(&hrp)
        .chars()
      {
        print!("{c}");
      }

      println!();
    }

    Ok(())
  }
}
