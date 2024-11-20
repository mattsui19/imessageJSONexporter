/*!
 Data structures and models used to parse and represent message data.
*/

pub use message::Message;

pub mod attachment_metadata;
pub(crate) mod body;
pub mod message;
pub mod models;
mod tests;
