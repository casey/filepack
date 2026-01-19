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
}

impl Bech32m {
  pub(crate) fn run(self) -> Result {
    if let Some(bech32m) = self.decode {
      let checked =
        CheckedHrpstring::new::<bech32::Bech32m>(&bech32m).context(error::Bech32mDecode)?;
      let bytes = checked.byte_iter().collect::<Vec<u8>>();
      let hex = hex::encode(bytes);
      println!("{hex}");
    } else {
      let hrp = Hrp::parse(&self.hrp.unwrap()).context(error::Bech32mHrp)?;
      let hex = hex::decode(self.encode.unwrap()).context(error::Hex)?;
      let bech32m = bech32::encode::<bech32::Bech32m>(hrp, &hex).context(error::Bech32mEncode)?;
      println!("{bech32m}");
    }

    Ok(())
  }
}
