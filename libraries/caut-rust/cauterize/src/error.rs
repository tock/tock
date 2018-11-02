#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Encode,
    Decode,
    InvalidTag,
    InvalidValue,
    ElementCount,
    OutOfRange,
}
