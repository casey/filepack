use super::*;

#[derive(Debug, PartialEq, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Message {
  Download(Download),
  File(File),
  Ok,
  Upload(Upload),
}

impl Message {
  pub(crate) fn read(connection: &mut dyn Connection) -> NodeResult<Self> {
    let mut len = [0; 4];
    connection
      .read_exact(&mut len)
      .context(node_error::ConnectionIo)?;
    let len = u32::from_be_bytes(len);
    let len = len.into_usize();
    let mut payload = vec![0; len];
    connection
      .read_exact(&mut payload)
      .context(node_error::ConnectionIo)?;
    Self::decode_from_slice(&payload).context(node_error::DecodeMessage)
  }

  pub(crate) fn write(&self, connection: &mut dyn Connection) -> NodeResult {
    let message = self.encode_to_vec();
    let size = message.len();
    let size = u32::try_from(size).context(node_error::MessageSize { size })?;
    connection
      .write_all(&size.to_be_bytes())
      .context(node_error::ConnectionIo)?;
    connection
      .write_all(&message)
      .context(node_error::ConnectionIo)?;
    Ok(())
  }
}

impl Decode for Message {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    let mut decoder = decoder.array()?;
    let discriminant = decoder.element::<u8>()?;
    match discriminant {
      0 => Ok(Self::Ok),
      1 => {
        let download = decoder.element::<Download>()?;
        Ok(Self::Download(download))
      }
      2 => {
        let upload = decoder.element::<Upload>()?;
        Ok(Self::Upload(upload))
      }
      3 => {
        let file = decoder.element::<File>()?;
        Ok(Self::File(file))
      }
      _ => Err(
        decode_error::InvalidDiscriminant {
          discriminant,
          name: "Message",
        }
        .build(),
      ),
    }
  }
}

impl Encode for Message {
  fn encode(&self, encoder: &mut Encoder) {
    match self {
      Self::Ok => {
        let mut encoder = encoder.array(1);
        encoder.element(0);
      }
      Self::Download(download) => {
        let mut encoder = encoder.array(2);
        encoder.element(1);
        encoder.element(download);
      }
      Self::Upload(upload) => {
        let mut encoder = encoder.array(2);
        encoder.element(2);
        encoder.element(upload);
      }
      Self::File(file) => {
        let mut encoder = encoder.array(2);
        encoder.element(3);
        encoder.element(file);
      }
    }
  }
}

#[derive(Debug, Decode, Encode, PartialEq)]
pub(crate) struct Download {
  #[n(0)]
  pub(crate) hash: Hash,
}

#[derive(Debug, Decode, Encode, PartialEq)]
pub(crate) struct File {
  #[n(0)]
  pub(crate) file: Vec<u8>,
}

#[derive(Debug, Decode, Encode, PartialEq)]
#[allow(clippy::arbitrary_source_item_ordering)]
pub(crate) struct Upload {
  #[n(0)]
  pub(crate) hash: Hash,
  #[n(1)]
  pub(crate) file: Vec<u8>,
}
