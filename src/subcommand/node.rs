use super::*;

#[derive(Parser)]
pub(crate) struct Node {
  address: String,
}

impl Node {
  pub(crate) fn run(self) -> Result {
    let listener = TcpListener::bind(self.address).unwrap();

    loop {
      let (socket, addr) = listener.accept().unwrap();
    }

    Ok(())
  }
}
