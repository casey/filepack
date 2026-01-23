use super::*;

#[derive(Parser)]
#[group(required = true)]
pub(crate) struct Bech32 {
  #[arg(
    group = "source",
    help = "Convert <BECH32> to hexadecimal",
    long,
    value_name = "BECH32"
  )]
  decode: Option<String>,
  #[arg(
    group = "source",
    help = "Encode <HEXADECIMAL> to bech32",
    long,
    requires = "hrp",
    value_name = "HEXADECIMAL"
  )]
  encode: Option<String>,
  #[arg(help = "Prefix bech32 with human-readable part <HRP>", long)]
  hrp: Option<String>,
  #[arg(help = "Strip or add <PREFIX> characters", long)]
  prefix: Option<String>,
}

impl Bech32 {
  pub(crate) fn run(self) -> Result {
    let prefix = self
      .prefix
      .map(|prefix| {
        prefix
          .chars()
          .map(|character| Fe32::from_char(character).context(error::Bech32Prefix { character }))
          .collect::<Result<Vec<Fe32>>>()
      })
      .transpose()?;

    if let Some(bech32) = self.decode {
      let hrp_string =
        CheckedHrpstring::new::<::bech32::Bech32m>(&bech32).context(error::Bech32Decode {
          bech32: bech32.clone(),
        })?;

      let mut fes = hrp_string
        .fe32_iter::<std::vec::IntoIter<u8>>()
        .collect::<Vec<Fe32>>();

      if let Some(prefix) = prefix {
        ensure! {
          fes.starts_with(&prefix),
          error::Bech32PrefixMissing,
        }

        fes.drain(..prefix.len());
      }

      let bytes = fes.into_iter().fes_to_bytes().collect::<Vec<u8>>();
      let hex = hex::encode(bytes);
      println!("{hex}");
    } else {
      let hrp = Hrp::parse(&self.hrp.unwrap()).context(error::Bech32Hrp)?;
      let hex = self.encode.unwrap();
      let hex = hex::decode(&hex).context(error::Hex { hex })?;

      let fes = if let Some(prefix) = prefix {
        prefix
          .into_iter()
          .chain(hex.iter().copied().bytes_to_fes())
          .collect::<Vec<Fe32>>()
      } else {
        hex.iter().copied().bytes_to_fes().collect::<Vec<Fe32>>()
      };

      for c in fes
        .into_iter()
        .with_checksum::<::bech32::Bech32m>(&hrp)
        .chars()
      {
        print!("{c}");
      }

      println!();
    }

    Ok(())
  }
}
