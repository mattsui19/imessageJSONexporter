/*!
 Defines routines for converting video files.
*/


use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};

use imessage_database::tables::attachment::MediaType;

use crate::app::compatibility::{
    converters::common::{copy_raw, run_command},
    models::{VideoConverter, VideoType},
};

/// Copy a video file, converting if possible
///
/// - Attachment `MOV` files convert to `MP4`
/// - Fallback to the original format
pub(crate) fn video_copy_convert(
    from: &Path,
    to: &mut PathBuf,
    converter: &VideoConverter,
    mime_type: MediaType,
) -> Option<MediaType<'static>> {
    // Normal attachments always get converted to jpeg
    if matches!(
        mime_type,
        MediaType::Video("mov") | MediaType::Video("MOV") | MediaType::Video("quicktime")
    ) {
        let output_type = VideoType::Mp4;
        // Update extension for conversion
        to.set_extension(output_type.to_str());
        if convert_mov(from, to, converter).is_none() {
            eprintln!("Unable to convert {from:?}");
        } else {
            return Some(MediaType::Video(output_type.to_str()));
        }
    }

    // Fallback
    copy_raw(from, to);
    None
}

fn convert_mov(from: &Path, to: &Path, converter: &VideoConverter) -> Option<()> {
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
        VideoConverter::Ffmpeg => run_command("ffmpeg", vec!["-i", from_path, to_path]),
    }
}
