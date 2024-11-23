use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};

use imessage_database::tables::attachment::MediaType;

use crate::app::compatibility::{
    converters::common::{copy_raw, run_command},
    models::{AudioConverter, AudioType},
};

pub(crate) fn audio_copy_convert(
    from: &Path,
    to: &mut PathBuf,
    converter: &AudioConverter,
    mime_type: MediaType,
) -> Option<MediaType<'static>> {
    // Normal attachments always get converted to jpeg
    if matches!(
        mime_type,
        MediaType::Audio("caf") | MediaType::Audio("CAF") | MediaType::Audio("x-caf; codecs=opus")
    ) {
        let output_type = AudioType::Mp4;
        // Update extension for conversion
        to.set_extension(output_type.to_str());
        if convert_caf(from, to, converter).is_none() {
            eprintln!("Unable to convert {from:?}");
        } else {
            return Some(MediaType::Audio(output_type.to_str()));
        }
    }
    copy_raw(from, to);
    None
}

fn convert_caf(from: &Path, to: &Path, converter: &AudioConverter) -> Option<()> {
    // Get the path we want to copy from
    let from_path = from.to_str()?;

    // Get the path we want to write to
    let to_path = to.to_str()?;

    // Ensure the directory tree exists
    if let Some(folder) = to.parent() {
        if !folder.exists() {
            if let Err(why) = create_dir_all(folder) {
                eprintln!("Unable to create {folder:?}: {why}");
                return None;
            }
        }
    }

    match converter {
        AudioConverter::AfConvert => run_command(
            "afconvert",
            vec!["-f", "mp4f", "-d", "aac", "-v", from_path, to_path],
        ),
        AudioConverter::Ffmpeg => run_command("ffmpeg", vec!["-i", from_path, to_path]),
    }
}
