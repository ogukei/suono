
use std::io;
use std::result;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum ErrorCode {
    Io(io::Error),
    WrongMagic,
    InvalidMetadataType,

    FrameOutOfSync,
    FrameHeaderCrcMismatch,
    FrameSampleSizeUnknown,
    FrameBlockSizeUnknown,
    FrameChannelAssignmentUnknown,
    FrameCrcMismatch,

    SubframeReservedType,
    SubframeOutOfSync,

    ResidualCodingMethodUnknown,

    FixedLPCCoefficientUnknown,
    QLPPrecisionInvalid,
    LPCSignalRestoreFailure,

    FrameBufferUnallocated
}

#[derive(Debug)]
pub struct Error {
    u: Box<ErrorCode>
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error {
            u: Box::new(ErrorCode::Io(err))
        }
    }
}

impl Error {
    pub fn from_code(code: ErrorCode) -> Self {
        Error {
            u: Box::new(code)
        }
    }
}