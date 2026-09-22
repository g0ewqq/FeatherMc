#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProtoError {
    #[error("unexpected end of data")]
    Truncated,

    #[error("VarInt is too long")]
    VarIntTooLong,

    #[error("VarLong is too long")]
    VarLongTooLong,

    #[error("string is too long: {0} bytes")]
    StringTooLong(i32),

    #[error("string is not valid UTF-8")]
    InvalidUtf8,

    #[error("packet is too long: {0} bytes")]
    PacketTooLong(i32),

    #[error("unknown packet with id {0:#04X}")]
    UnknownPacket(i32),

    #[error("invalid handshake next state: {0}")]
    InvalidNextState(i32),

    #[error("invalid NBT tag id: {0}")]
    InvalidTag(u8),

    #[error("negative NBT length: {0}")]
    NegativeLength(i32),

    #[error("NBT list holds mixed tag types")]
    MixedList,

    #[error("NBT nesting is too deep")]
    DepthExceeded,
}
