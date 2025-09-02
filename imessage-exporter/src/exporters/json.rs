use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
};

use crate::{
    app::{
        error::RuntimeError,
        progress::ExportProgress, runtime::Config,
    },
    exporters::exporter::{ATTACHMENT_NO_FILENAME, BalloonFormatter, Exporter, MessageFormatter},
};

use imessage_database::{
    error::plist::PlistParseError,
    message_types::edited::EditedMessage,
    tables::{
        attachment::Attachment,
        messages::{Message, models::AttachmentMeta, models::TextAttributes},
        table::{ORPHANED, Table},
    },
    util::{
        dates::{format, get_local_time},
    },
};

use serde_json::json;
use rusqlite;

pub struct JSON<'a> {
    /// Data that is setup from the application's runtime
    pub config: &'a Config,
    /// Handles to files we want to write messages to
    /// Map of resolved chatroom file location to a buffered writer
    pub files: HashMap<String, BufWriter<File>>,
    /// Writer instance for orphaned messages
    pub orphaned: BufWriter<File>,
    /// Progress Bar model for alerting the user about current export state
    pb: ExportProgress,
}

// MARK: Exporter
impl<'a> Exporter<'a> for JSON<'a> {
    fn new(config: &'a Config) -> Result<Self, RuntimeError> {
        let mut orphaned = config.options.export_path.clone();
        orphaned.push(ORPHANED);
        orphaned.set_extension("json");

        let file = File::options().append(true).create(true).open(&orphaned)?;

        Ok(JSON {
            config,
            files: HashMap::new(),
            orphaned: BufWriter::new(file),
            pb: ExportProgress::new(),
        })
    }

    fn iter_messages(&mut self) -> Result<(), RuntimeError> {
        // Tell the user what we are doing
        eprintln!(
            "Exporting to {} as json...",
            self.config.options.export_path.display()
        );

        // Keep track of current message ROWID
        let mut current_message_row = -1;

        // Set up progress bar
        let mut current_message = 0;
        let total_messages =
            Message::get_count(self.config.db(), &self.config.options.query_context)?;
        self.pb.start(total_messages);

        let mut statement =
            Message::stream_rows(self.config.db(), &self.config.options.query_context)?;

        let messages = statement
            .query_map([], |row| Ok(Message::from_row(row)))
            .map_err(|err| RuntimeError::DatabaseError(imessage_database::error::table::TableError::QueryError(err)))?;

        for message in messages {
            let mut msg = Message::extract(message)?;

            // Early escape if we try and render the same message GUID twice
            if msg.rowid == current_message_row {
                current_message += 1;
                continue;
            }
            current_message_row = msg.rowid;

            // Generate the text of the message
            let _ = msg.generate_text(self.config.db());

            // Skip tapbacks as they're handled in context
            if !msg.is_tapback() {
                let message_json = self.format_message(&msg, 0)?;
                JSON::write_to_file(self.get_or_create_file(&msg)?, &message_json)?;
            }
            
            current_message += 1;
            if current_message % 99 == 0 {
                self.pb.set_position(current_message);
            }
        }
        self.pb.finish();
        Ok(())
    }

    /// Create a file for the given chat, caching it so we don't need to build it later
    fn get_or_create_file(
        &mut self,
        message: &Message,
    ) -> Result<&mut BufWriter<File>, RuntimeError> {
        match self.config.conversation(message) {
            Some((chatroom, _)) => {
                let filename = self.config.filename(chatroom);
                match self.files.entry(filename) {
                    std::collections::hash_map::Entry::Occupied(entry) => Ok(entry.into_mut()),
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        let mut path = self.config.options.export_path.clone();
                        path.push(self.config.filename(chatroom));
                        path.set_extension("json");

                        let file = File::options().append(true).create(true).open(&path)?;

                        Ok(entry.insert(BufWriter::new(file)))
                    }
                }
            }
            None => Ok(&mut self.orphaned),
        }
    }

    fn write_to_file(file: &mut BufWriter<File>, text: &str) -> Result<(), RuntimeError> {
        file.write_all(text.as_bytes())
            .map_err(RuntimeError::DiskError)
    }
}

// MARK: MessageFormatter
impl<'a> MessageFormatter<'a> for JSON<'a> {
    fn format_message(&self, message: &Message, _indent_size: usize) -> Result<String, imessage_database::error::table::TableError> {
        // Get basic message info
        let timestamp = format(&message.date(&self.config.offset));
        let sender = self.config.who(
            message.handle_id,
            message.is_from_me(),
            &message.destination_caller_id,
        );

        // Get read time if available
        let readtime = if message.date_read > 0 {
            let read_time = get_local_time(&message.date_read, &self.config.offset);
            Some(format(&read_time))
        } else {
            None
        };

        // Get message contents
        let contents = message.text.clone().unwrap_or_default();

        // Get attachments
        let mut attachments = Vec::new();
        if let Ok(attachments_list) = Attachment::from_message(self.config.db(), message) {
            for attachment in attachments_list {
                let attachment_info = json!({
                    "filename": attachment.filename().unwrap_or(ATTACHMENT_NO_FILENAME),
                    "mime_type": format!("{:?}", attachment.mime_type()),
                    "file_size": attachment.file_size()
                });
                attachments.push(attachment_info);
            }
        }

        // Create the JSON object
        let message_json = json!({
            "timestamp": timestamp,
            "sender": sender,
            "contents": contents,
            "attachments": attachments,
            "readtime": readtime,
            "is_from_me": message.is_from_me,
            "guid": message.guid
        });

        // Convert to pretty-printed JSON string
        Ok(serde_json::to_string_pretty(&message_json)
            .map_err(|_| imessage_database::error::table::TableError::QueryError(rusqlite::Error::InvalidParameterName("JSON serialization failed".to_string())))?)
    }

    fn format_attachment(
        &self,
        attachment: &'a mut Attachment,
        _message: &Message,
        _metadata: &AttachmentMeta,
    ) -> Result<String, &'a str> {
        Ok(format!("{:?}", attachment.mime_type()))
    }

    fn format_sticker(&self, attachment: &'a mut Attachment, _message: &Message) -> String {
        format!("Sticker: {:?}", attachment.mime_type())
    }

    fn format_app(
        &self,
        _message: &'a Message,
        _attachments: &mut Vec<Attachment>,
        _indent: &str,
    ) -> Result<String, PlistParseError> {
        Ok("App message".to_string())
    }

    fn format_tapback(&self, _msg: &Message) -> Result<String, imessage_database::error::table::TableError> {
        Ok("Tapback".to_string())
    }

    fn format_expressive(&self, _msg: &'a Message) -> &'a str {
        "Expressive message"
    }

    fn format_announcement(&self, _msg: &'a Message) -> String {
        "Announcement".to_string()
    }

    fn format_shareplay(&self) -> &str {
        "SharePlay message"
    }

    fn format_shared_location(&self, _msg: &'a Message) -> &str {
        "Shared location"
    }

    fn format_edited(
        &self,
        _msg: &'a Message,
        _edited_message: &'a EditedMessage,
        _message_part_idx: usize,
        _indent: &str,
    ) -> Option<String> {
        Some("Edited message".to_string())
    }

    fn format_attributes(&'a self, text: &'a str, _attributes: &'a [TextAttributes]) -> String {
        text.to_string()
    }
}

// MARK: BalloonFormatter - Basic implementations for required traits
impl<'a> BalloonFormatter<&'a str> for JSON<'a> {
    fn format_url(&self, _msg: &Message, _balloon: &imessage_database::message_types::url::URLMessage, _indent: &str) -> String {
        "URL message".to_string()
    }

    fn format_music(&self, _balloon: &imessage_database::message_types::music::MusicMessage, _indent: &str) -> String {
        "Music message".to_string()
    }

    fn format_collaboration(&self, _balloon: &imessage_database::message_types::collaboration::CollaborationMessage, _indent: &str) -> String {
        "Collaboration message".to_string()
    }

    fn format_app_store(&self, _balloon: &imessage_database::message_types::app_store::AppStoreMessage, _indent: &str) -> String {
        "App Store message".to_string()
    }

    fn format_placemark(&self, _balloon: &imessage_database::message_types::placemark::PlacemarkMessage, _indent: &str) -> String {
        "Placemark message".to_string()
    }

    fn format_handwriting(&self, _msg: &Message, _balloon: &imessage_database::message_types::handwriting::HandwrittenMessage, _indent: &str) -> String {
        "Handwritten message".to_string()
    }

    fn format_digital_touch(&self, _: &Message, _balloon: &imessage_database::message_types::digital_touch::DigitalTouch, _indent: &str) -> String {
        "Digital Touch message".to_string()
    }

    fn format_apple_pay(&self, _balloon: &imessage_database::message_types::app::AppMessage, _indent: &str) -> String {
        "Apple Pay message".to_string()
    }

    fn format_fitness(&self, _balloon: &imessage_database::message_types::app::AppMessage, _indent: &str) -> String {
        "Fitness message".to_string()
    }

    fn format_slideshow(&self, _balloon: &imessage_database::message_types::app::AppMessage, _indent: &str) -> String {
        "Slideshow message".to_string()
    }

    fn format_check_in(&self, _balloon: &imessage_database::message_types::app::AppMessage, _indent: &str) -> String {
        "Check-in message".to_string()
    }

    fn format_generic_app(&self, _balloon: &imessage_database::message_types::app::AppMessage, _bundle_id: &str, _: &mut Vec<Attachment>, _indent: &str) -> String {
        "Generic app message".to_string()
    }

    fn format_find_my(&self, _balloon: &imessage_database::message_types::app::AppMessage, _indent: &str) -> String {
        "Find My message".to_string()
    }
}

// MARK: Tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Config, Exporter, Options, app::export_type::ExportType};

    #[test]
    fn can_create() {
        let options = Options::fake_options(ExportType::Html); // Use HTML as base since JSON isn't in ExportType yet
        let config = Config::fake_app(options);
        let exporter = JSON::new(&config).unwrap();
        assert_eq!(exporter.files.len(), 0);
    }

    #[test]
    fn can_format_basic_message() {
        let options = Options::fake_options(ExportType::Html);
        let config = Config::fake_app(options);
        let exporter = JSON::new(&config).unwrap();

        let mut message = Config::fake_message();
        message.text = Some("Hello world".to_string());
        message.is_from_me = true;
        message.date = 674526582885055488; // May 17, 2022 8:29:42 PM

        let result = exporter.format_message(&message, 0).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["sender"], "Me");
        assert_eq!(parsed["contents"], "Hello world");
        assert_eq!(parsed["is_from_me"], true);
        assert_eq!(parsed["guid"], message.guid);
    }
}
