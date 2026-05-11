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

      let message = Message::read_frame(&mut stream);

      match message {
        Message::Upload(upload) => {
          let actual = Hash::bytes(&upload.file);
          assert_eq!(actual, upload.hash);
          filesystem::write(&files, upload.file)?;
          Message::Ok.write_frame(&mut stream);
          stream.shutdown(net::Shutdown::Write).unwrap();
        }
        Message::Ok => todo!(),
      }
    }

    Ok(())
  }
}
