use super::*;

#[derive(Debug, PartialEq)]
pub(crate) enum Message {
  Ok,
  Download(Download),
  Upload(Upload),
  File(File),
}

impl Message {
  pub(crate) fn write(&self, stream: &mut TcpStream) {
    let message = self.encode_to_vec();
    let len = u32::try_from(message.len()).unwrap();
    let len = len.to_be_bytes();
    stream.write_all(&len).unwrap();
    stream.write_all(&message).unwrap();
  }

  pub(crate) fn read(stream: &mut TcpStream) -> Self {
    let mut len = [0; 4];
    stream.read_exact(&mut len).unwrap();
    let len = u32::from_be_bytes(len);
    let len = len.into_usize();
    let mut payload = vec![0; len];
    stream.read_exact(&mut payload).unwrap();
    Self::decode_from_slice(&payload).unwrap()
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
      _ => todo!(),
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
pub(crate) struct Upload {
  #[n(0)]
  pub(crate) hash: Hash,
  #[n(1)]
  pub(crate) file: Vec<u8>,
}
