/*!
 Errors that can happen when extracting data from a `SQLite` table.
*/

use std::fmt::{Display, Formatter, Result};

/// Errors that can happen when extracting data from a `SQLite` table
#[derive(Debug)]
pub enum TableError {
    /// Error when parsing attachment data
    Attachment(rusqlite::Error),
    /// Error when parsing chat to handle relationship data
    ChatToHandle(rusqlite::Error),
    /// Error when parsing chat data
    Chat(rusqlite::Error),
    /// Error when parsing handle data
    Handle(rusqlite::Error),
    /// Error when parsing messages data
    Messages(rusqlite::Error),
    /// Error when connecting to the database
    CannotConnect(String),
    /// Error when reading from the database file
    CannotRead(std::io::Error),
}

impl Display for TableError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        match self {
            TableError::Attachment(why) => write!(fmt, "Failed to parse attachment row: {why}"),
            TableError::ChatToHandle(why) => write!(fmt, "Failed to parse chat handle row: {why}"),
            TableError::Chat(why) => write!(fmt, "Failed to parse chat row: {why}"),
            TableError::Handle(why) => write!(fmt, "Failed to parse handle row: {why}"),
            TableError::Messages(why) => write!(fmt, "Failed to parse messages row: {why}"),
            TableError::CannotConnect(why) => write!(fmt, "{why}"),
            TableError::CannotRead(why) => write!(fmt, "{why}"),
        }
    }
}
