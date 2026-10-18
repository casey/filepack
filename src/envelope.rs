use super::*;

/// Domain-separating message wrapper that commits to the application and
/// context, so signatures of one kind over a serialized envelope cannot be
/// confused for another.
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
