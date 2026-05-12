use super::*;

// later:
// - reading and writing files should be incremental
// - don't allow large messages

pub(crate) struct Node {
  files: Utf8PathBuf,
}

impl Node {
  pub(crate) fn new(files: Utf8PathBuf) -> Self {
    Self { files }
  }

  pub(crate) fn serve(&self, mut connection: &mut dyn Connection) -> Result {
    let message = Message::read(connection);

    match message {
      Message::Download(download) => {
        let path = self.files.join(download.hash.to_string());
        let file = filesystem::read(&path)?;
        Message::File(message::File { file }).write(connection);
      }
      Message::Upload(upload) => {
        let actual = Hash::bytes(&upload.file);
        assert_eq!(actual, upload.hash);
        let path = self.files.join(actual.to_string());
        // todo: don't write if it already exists (use create options)
        filesystem::write(&path, upload.file)?;
        Message::Ok.write(connection);
      }
      Message::File(_) | Message::Ok => todo!(),
    }

    Ok(())
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
  }
}
