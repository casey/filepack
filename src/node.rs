use super::*;

pub(crate) struct Node {
  files: Utf8PathBuf,
}

impl Node {
  pub(crate) fn new(files: Utf8PathBuf) -> Self {
    Self { files }
  }

  pub(crate) fn serve(&self, mut stream: TcpStream) -> Result {
    let message = Message::read(&mut stream);

    match message {
      Message::Download(download) => {
        let path = self.files.join(download.hash.to_string());
        let file = filesystem::read(&path)?;
        Message::File(message::File { file }).write(&mut stream);
        stream.shutdown(net::Shutdown::Both).unwrap();
      }
      Message::Upload(upload) => {
        let actual = Hash::bytes(&upload.file);
        assert_eq!(actual, upload.hash);
        let path = self.files.join(actual.to_string());
        // todo: don't write if it already exists (use create options)
        filesystem::write(&path, upload.file)?;
        Message::Ok.write(&mut stream);
        stream.shutdown(net::Shutdown::Both).unwrap();
      }
      Message::File(_) | Message::Ok => todo!(),
    }

    Ok(())
  }
}
