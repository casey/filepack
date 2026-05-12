use super::*;

pub(crate) trait Connection: Read + Write {}

impl Connection for TcpStream {}
