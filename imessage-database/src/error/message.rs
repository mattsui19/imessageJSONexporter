/*!
 Errors that can happen when parsing message data.
*/

use std::fmt::{Display, Formatter, Result};

use crate::error::{plist::PlistParseError, streamtyped::StreamTypedError};

use super::typedstream::TypedStreamError;

/// Errors that can happen when working with message table data
#[derive(Debug)]
pub enum MessageError {
    /// Required data is missing from the message
    MissingData,
    /// Message has no text content
    NoText,
    /// Error occurred when parsing with the StreamTyped parser
    StreamTypedParseError(StreamTypedError),
    /// Error occurred when parsing with the TypedStream parser
    TypedStreamParseError(TypedStreamError),
    /// Error occurred when parsing Plist data
    PlistParseError(PlistParseError),
    /// Timestamp value is invalid or out of range
    InvalidTimestamp(i64),
}

impl Display for MessageError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        match self {
            MessageError::MissingData => write!(fmt, "No attributedBody found!"),
            MessageError::NoText => write!(fmt, "Message has no text!"),
            MessageError::StreamTypedParseError(why) => {
                write!(
                    fmt,
                    "Failed to parse attributedBody with legacy parser: {why}"
                )
            }
            MessageError::TypedStreamParseError(why) => {
                write!(fmt, "Failed to parse attributedBody: {why}")
            }
            MessageError::PlistParseError(why) => {
                write!(fmt, "Failed to parse plist data: {why}")
            }
            MessageError::InvalidTimestamp(when) => {
                write!(fmt, "Timestamp is invalid: {when}")
            }
        }
    }
}
