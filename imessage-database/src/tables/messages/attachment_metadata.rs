/*!
 This module represents attachment metadata stored in [`typedstream`](crate::util::typedstream) `attributedBody` data.
*/

use crate::util::typedstream::models::Archivable;

/// Representation of attachment metadata used for rendering message body in a conversation feed.
#[derive(Debug, PartialEq, Default)]
pub struct AttachmentMeta<'a> {
    /// GUID of the attachment in the table
    pub guid: Option<&'a str>,
    /// The transcription, if the attachment was an [audio message](https://support.apple.com/guide/iphone/send-and-receive-audio-messages-iph2e42d3117/ios) sent from or received on a [supported platform](https://www.apple.com/ios/feature-availability/#messages-audio-message-transcription).
    pub transcription: Option<&'a str>,
    /// The height of the attachment in points
    pub height: Option<&'a f64>,
    /// The width of the attachment in points
    pub width: Option<&'a f64>,
    /// The attachment's original filename
    pub name: Option<&'a str>,
}

impl<'a> AttachmentMeta<'a> {
    /// Given a slice of parsed [`typedstream`](crate::util::typedstream) data, populate the attachment's metadata fields.
    ///
    /// # Example
    /// ```
    /// use imessage_database::util::typedstream::models::{Archivable, Class, OutputData};
    /// use imessage_database::tables::messages::attachment_metadata::AttachmentMeta;
    ///
    /// // Sample components
    /// let components = vec![
    ///    Archivable::Object(
    ///        Class {
    ///            name: "NSString".to_string(),
    ///            version: 1,
    ///        },
    ///        vec![OutputData::String(
    ///            "__kIMFileTransferGUIDAttributeName".to_string(),
    ///        )],
    ///    ),
    ///    Archivable::Object(
    ///        Class {
    ///            name: "NSString".to_string(),
    ///            version: 1,
    ///        },
    ///        vec![OutputData::String(
    ///            "4C339597-EBBB-4978-9B87-521C0471A848".to_string(),
    ///        )],
    ///    ),
    /// ];
    /// let meta = AttachmentMeta::from_typedstream(&components);
    /// ```
    pub fn from_components(components: &'a [Archivable]) -> Option<Self> {
        let mut guid = None;
        let mut transcription = None;
        let mut height = None;
        let mut width = None;
        let mut name = None;

        for (idx, key) in components.iter().enumerate() {
            if let Some(key_name) = key.as_nsstring() {
                match key_name {
                    "__kIMFileTransferGUIDAttributeName" => {
                        guid = components.get(idx + 1)?.as_nsstring()
                    }
                    "IMAudioTranscription" => {
                        transcription = components.get(idx + 1)?.as_nsstring()
                    }
                    "__kIMInlineMediaHeightAttributeName" => {
                        height = components.get(idx + 1)?.as_nsnumber_float()
                    }
                    "__kIMInlineMediaWidthAttributeName" => {
                        width = components.get(idx + 1)?.as_nsnumber_float()
                    }
                    "__kIMFilenameAttributeName" => name = components.get(idx + 1)?.as_nsstring(),
                    _ => {}
                }
            }
        }

        Some(Self {
            guid,
            transcription,
            height,
            width,
            name,
        })
    }
}
