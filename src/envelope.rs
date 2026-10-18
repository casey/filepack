use super::*;

#[derive(Encode)]
pub(crate) struct Envelope<T> {
  /// Application the message belongs to.
  #[n(1)]
  pub(crate) application: Application,
  /// Type of the enclosed message.
  #[n(2)]
  pub(crate) context: Context,
  /// Message being signed.
  #[n(3)]
  pub(crate) message: T,
}
