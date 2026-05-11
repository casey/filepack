use super::*;

#[derive(Parser)]
pub(crate) struct Upload {
  address: String,
  file: Utf8PathBuf,
}

impl Upload {
  pub(crate) fn run(self) -> Result {
    let mut stream = TcpStream::connect(self.address).unwrap();

    let file = filesystem::read(&self.file)?;

    let hash = Hash::bytes(&file);

    let message = Message::Upload(message::Upload { hash, file });

    message.write(&mut stream);

    let message = Message::read(&mut stream);

    assert_eq!(message, Message::Ok);

    Ok(())
  }
}
