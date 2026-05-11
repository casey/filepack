use super::*;

#[derive(Parser)]
pub(crate) struct Node {
  address: String,
}

impl Node {
  pub(crate) fn run(self, options: Options) -> Result {
    let listener = TcpListener::bind(self.address).unwrap();

    let files = options.data_dir()?.join("files");

    loop {
      let (mut stream, addr) = listener.accept().unwrap();

      let message = Message::read(&mut stream);

      match message {
        Message::Download(download) => {
          let path = files.join(download.hash.to_string());
          let file = filesystem::read(&path)?;
          Message::File(message::File { file }).write(&mut stream);
          stream.shutdown(net::Shutdown::Both).unwrap();
        }
        Message::Upload(upload) => {
          let actual = Hash::bytes(&upload.file);
          assert_eq!(actual, upload.hash);
          let path = files.join(actual.to_string());
          // don't write if it already exists (use create options)
          filesystem::write(&path, upload.file)?;
          Message::Ok.write(&mut stream);
          stream.shutdown(net::Shutdown::Both).unwrap();
        }
        Message::File(_) | Message::Ok => todo!(),
      }
    }

    Ok(())
  }
}
