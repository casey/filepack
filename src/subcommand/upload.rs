use super::*;

#[derive(Parser)]
pub(crate) struct Upload {
  address: String,
  file: Utf8PathBuf,
}

#[derive(Debug, PartialEq)]
enum Message {
  Ok,
  Upload(UploadMessage),
}

impl Message {
  fn write_frame(&self, stream: &mut TcpStream) {
    let message = self.encode_to_vec();
    let len = u32::try_from(message.len()).unwrap();
    let len = len.to_be_bytes();
    stream.write_all(&len).unwrap();
    stream.write_all(&message).unwrap();
  }

  fn read_frame(stream: &mut TcpStream) -> Self {
    let mut len = [0; 4];
    stream.read_exact(&mut len).unwrap();
    let len = u32::from_be_bytes(len);
    let len = len.into_usize();
    let mut payload = vec![0; len];
    stream.read_exact(&mut payload).unwrap();
    Self::decode_from_slice(&payload).unwrap()
  }
}

#[derive(Debug, Decode, Encode, PartialEq)]
struct UploadMessage {
  #[n(0)]
  hash: Hash,
  #[n(1)]
  file: Vec<u8>,
}

impl Decode for Message {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    let mut decoder = decoder.array()?;
    let discriminant = decoder.element::<u8>()?;
    match discriminant {
      0 => Ok(Self::Ok),
      1 => {
        let upload = decoder.element::<UploadMessage>()?;
        Ok(Self::Upload(upload))
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
      Self::Upload(upload) => {
        let mut encoder = encoder.array(2);
        encoder.element(1);
        encoder.element(upload);
      }
    }
  }
}

impl Upload {
  pub(crate) fn run(self) -> Result {
    let mut stream = TcpStream::connect(self.address).unwrap();

    let file = filesystem::read(&self.file)?;

    let hash = Hash::bytes(&file);

    let message = Message::Upload(UploadMessage { hash, file });

    message.write_frame(&mut stream);

    let message = Message::read_frame(&mut stream);

    assert_eq!(message, Message::Ok);

    Ok(())
  }
}
