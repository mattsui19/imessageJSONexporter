use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};

use imessage_database::tables::attachment::MediaType;

use crate::app::compatibility::{
    converters::common::{copy_raw, run_command},
    models::{ImageConverter, ImageType},
};

/// Copy an image file, converting if possible
///
/// - Attachment `HEIC` files convert to `JPEG`
/// - Fallback to the original format
pub(crate) fn image_copy_convert(
    from: &Path,
    to: &mut PathBuf,
    converter: &ImageConverter,
    mime_type: MediaType,
) -> Option<MediaType<'static>> {
    // Normal attachments always get converted to jpeg
    if matches!(
        mime_type,
        MediaType::Image("heic") | MediaType::Image("HEIC")
    ) {
        let output_type = ImageType::Jpeg;
        // Update extension for conversion
        to.set_extension(output_type.to_str());
        if convert_heic(from, to, converter, &output_type).is_none() {
            eprintln!("Unable to convert {from:?}");
        } else {
            return Some(MediaType::Image(output_type.to_str()));
        }
    }

    // Fallback
    copy_raw(from, to);
    None
}

/// Convert a HEIC image file to the provided format
///
/// This uses the macOS builtin `sips` program
/// Docs: <https://www.unix.com/man-page/osx/1/sips/> (or `man sips`)
///
/// If `to` contains a directory that does not exist, i.e. `/fake/out.jpg`, instead
/// of failing, `sips` will create a file called `fake` in `/`. Subsequent writes
/// by `sips` to the same location will not fail, but since it is a file instead
/// of a directory, this will fail for non-`sips` copies.
pub(super) fn convert_heic(
    from: &Path,
    to: &Path,
    converter: &ImageConverter,
    output_image_type: &ImageType,
) -> Option<()> {
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
        ImageConverter::Sips => {
            // Build the command
            run_command(
                "sips",
                vec![
                    "-s",
                    "format",
                    output_image_type.to_str(),
                    from_path,
                    "-o",
                    to_path,
                ],
            )
        }
        ImageConverter::Imagemagick =>
        // Build the command
        {
            run_command("magick", vec![from_path, to_path])
        }
    };

    Some(())
}
