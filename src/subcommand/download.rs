use super::*;

#[derive(Parser)]
pub(crate) struct Download {
  address: String,
  hash: Hash,
  output: Utf8PathBuf,
}

impl Download {
  pub(crate) fn run(self) -> Result {
    let mut stream = TcpStream::connect(self.address).unwrap();

    let message = Message::Download(message::Download { hash: self.hash });

    message.write(&mut stream)?;

    let message = Message::read(&mut stream)?;

    let Message::File(message::File { file }) = message else {
      todo!();
    };

    let hash = Hash::bytes(&file);

    assert_eq!(hash, self.hash);

    filesystem::write(&self.output, file).unwrap();

    Ok(())
  }
}
