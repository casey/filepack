use super::*;

// later:
// - reading and writing files should be incremental
// - don't allow large messages
// - return error messages when things go wrong
// - return an error message when file doesn't exist
// - should I use mut dyn Connection or generic?
// - add logging for node errors
// - figure out if I want to add peer address to all errors
// - derive message Encode and Decode

pub(crate) struct Node {
  files: Utf8PathBuf,
}

impl Node {
  pub(crate) fn new(files: Utf8PathBuf) -> Self {
    Self { files }
  }

  fn read_file(&self, hash: Hash) -> NodeResult<Vec<u8>> {
    let path = self.files.join(hash.to_string());
    fs::read(&path).context(node_error::FilesystemIo { path })
  }

  pub(crate) fn serve(&self, connection: &mut dyn Connection) -> NodeResult {
    let message = Message::read(connection)?;

    match message {
      Message::Download(download) => {
        let file = self.read_file(download.hash)?;
        Message::File(message::File { file }).write(connection)?;
      }
      Message::Upload(upload) => {
        let hash = Hash::bytes(&upload.file);
        assert_eq!(hash, upload.hash);
        self.write_file(hash, &upload.file)?;
        Message::Ok.write(connection)?;
      }
      Message::File(_) | Message::Ok => {
        return Err(node_error::UnexpectedMessage { message }.build());
      }
    }

    Ok(())
  }

  fn write_file(&self, hash: Hash, contents: &[u8]) -> NodeResult {
    let path = self.files.join(hash.to_string());

    let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
      Ok(file) => file,
      Err(err) => {
        if err.kind() == io::ErrorKind::AlreadyExists {
          return Ok(());
        }

        return Err(node_error::FilesystemIo { path }.into_error(err));
      }
    };

    file
      .write_all(contents)
      .context(node_error::FilesystemIo { path })
  }
}

#[cfg(test)]
mod tests {
  use {super::*, std::collections::VecDeque};

  #[derive(Default)]
  struct TestConnection {
    read: VecDeque<u8>,
    write: Vec<u8>,
  }

  impl TestConnection {
    fn new() -> Self {
      Self::default()
    }
  }

  impl Connection for TestConnection {}

  impl Read for TestConnection {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
      self.read.read(buffer)
    }
  }

  impl Write for TestConnection {
    fn flush(&mut self) -> io::Result<()> {
      self.write.flush()
    }

    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
      self.write.write(buffer)
    }
  }

  #[test]
  fn unexpected_message() {
    let (_dir, path) = tempdir();

    let node = Node::new(path);

    let mut connection = TestConnection::new();

    node.serve(&mut connection).unwrap();
  }
}
