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
  #[arg(
    help = "Version character (bech32 field element: q-z, 0-9, excluding 1, b, i, o)",
    long
  )]
  version: Option<char>,
}

impl Bech32m {
  pub(crate) fn run(self) -> Result {
    use bech32::Fe32;

    let version = self
      .version
      .map(|c| Fe32::from_char(c).map_err(|_| error::Bech32mVersion { version: c }.build()))
      .transpose()?;

    if let Some(bech32m) = self.decode {
      let mut checked =
        CheckedHrpstring::new::<bech32::Bech32m>(&bech32m).context(error::Bech32mDecode {
          bech32m: bech32m.clone(),
        })?;

      if version.is_some()
        && let Some(v) = checked.remove_witness_version()
      {
        if let Some(expected) = version {
          ensure!(
            v == expected,
            error::Bech32mVersionMismatch {
              actual: v,
              expected,
            },
          );
        }
        eprintln!("version: {v}");
      }

      let bytes = checked.byte_iter().collect::<Vec<u8>>();
      let hex = hex::encode(bytes);
      println!("{hex}");
    } else {
      use {bech32::ByteIterExt, bech32::Fe32IterExt, std::fmt::Write};

      let hrp = Hrp::parse(&self.hrp.unwrap()).context(error::Bech32mHrp)?;
      let hex = self.encode.unwrap();
      let hex = hex::decode(&hex).context(error::Hex { hex })?;

      let mut result = String::new();
      let iter = hex
        .iter()
        .copied()
        .bytes_to_fes()
        .with_checksum::<bech32::Bech32m>(&hrp);

      let chars = if let Some(v) = version {
        iter.with_witness_version(v).chars().collect::<Vec<_>>()
      } else {
        iter.chars().collect::<Vec<_>>()
      };

      for c in chars {
        result.write_char(c).unwrap();
      }
      println!("{result}");
    }

    Ok(())
  }
}
