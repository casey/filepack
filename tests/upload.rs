use {
  super::*,
  std::{
    io::{self, Read},
    os::{fd::AsRawFd, unix::process::CommandExt},
  },
};

#[test]
fn node_creates_file() {
  let node_dir = tempdir();

  let (mut reader, writer) = std::io::pipe().unwrap();
  let fd = writer.as_raw_fd();

  let mut command = Command::new(env!("CARGO_BIN_EXE_filepack"));

  command
    .args(["node", "--ready-fd", "3", "127.0.0.1:0"])
    .env("FILEPACK_DATA_DIR", node_dir.path())
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());

  unsafe {
    command.pre_exec(move || {
      if fd == 3 {
        let flags = libc::fcntl(fd, libc::F_GETFD);

        if flags == -1 {
          return Err(io::Error::last_os_error());
        }

        if libc::fcntl(fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) == -1 {
          return Err(io::Error::last_os_error());
        }
      } else if libc::dup2(fd, 3) == -1 {
        return Err(io::Error::last_os_error());
      }

      Ok(())
    });
  }

  let mut node = command.spawn().unwrap();

  drop(writer);

  let mut port = String::new();
  reader.read_to_string(&mut port).unwrap();
  let port = port.parse::<u16>().unwrap();

  Test::new()
    .write("foo", "bar")
    .args(["upload", &format!("127.0.0.1:{port}"), "foo"])
    .success();

  let path = node_dir
    .path()
    .join("files")
    .join(Hash::bytes(b"bar").to_string());

  assert_eq!(fs::read(path).unwrap(), b"bar");

  node.kill().unwrap();
  node.wait().unwrap();
}
