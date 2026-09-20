use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum AuthorizationError {
  #[snafu(display("token has incorrect audience `{audience}`"))]
  Audience { audience: String },
  #[snafu(display(
    "token is expired: token with timestamp {timestamp} expires at {expiry} which is before now {now}"
  ))]
  Expired {
    expiry: u64,
    now: u64,
    timestamp: u64,
  },
  #[snafu(display(
    "token is not yet valid: token with timestamp {timestamp} becomes valid at {start} which is after now {now}"
  ))]
  Pending {
    now: u64,
    start: u64,
    timestamp: u64,
  },
  #[snafu(context(false), display("token has invalid signature"))]
  Signature { source: SignatureError },
  #[snafu(display("token signed with incorrect public key `{signer}`"))]
  Signer { signer: PublicKey },
  #[snafu(display("failed to decode token"))]
  Token { source: HexError },
}
