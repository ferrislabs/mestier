/// A stored secret: the nonce is not sensitive and travels with the
/// ciphertext, but it must never repeat under one key — reusing a nonce with
/// GCM discloses the keystream.
///
/// Lives in the domain because ports exchange it. The cipher that produces it
/// stays in infrastructure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealedSecret {
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
}
