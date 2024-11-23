use std::{
    fs::{copy, create_dir_all},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use imessage_database::tables::attachment::MediaType;

use crate::app::converters::models::{
    AudioConverter, AudioType, ImageConverter, ImageType, VideoConverter, VideoType,
};

/// Copy a file without altering it
pub(super) fn copy_raw(from: &Path, to: &Path) {
    // Ensure the directory tree exists
    if let Some(folder) = to.parent() {
        if !folder.exists() {
            if let Err(why) = create_dir_all(folder) {
                eprintln!("Unable to create {folder:?}: {why}");
            }
        }
    }
    if let Err(why) = copy(from, to) {
        eprintln!("Unable to copy {from:?} to {to:?}: {why}");
    };
}

/// Copy an image file, converting if possible
///
/// - Attachment `HEIC` files convert to `JPEG`
/// - Fallback to the original format
pub(super) fn image_copy_convert(
    from: &Path,
    to: &mut PathBuf,
    converter: &ImageConverter,
    mime_type: MediaType,
) {
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
        }
    } else {
        copy_raw(from, to);
    }
}

/// Copy a sticker, converting if possible
///
/// - Sticker `HEIC` files convert to `PNG`
/// - Sticker `HEICS` files convert to `GIF`
/// - Fallback to the original format
pub(super) fn sticker_copy_convert(
    from: &Path,
    to: &mut PathBuf,
    converter: &ImageConverter,
    mime_type: MediaType,
) {
    // Determine the output type of the sticker
    let output_type: Option<ImageType> = match mime_type {
        // Normal stickers get converted to png
        MediaType::Image("heic") | MediaType::Image("HEIC") => Some(ImageType::Png),
        MediaType::Image("heics")
        | MediaType::Image("HEICS")
        // TODO: For heics, use ffmpeg gif converter process
        | MediaType::Image("heic-sequence") => Some(ImageType::Gif),
        _ => None,
    };

    match output_type {
        Some(output_type) => {
            to.set_extension(output_type.to_str());
            if convert_heic(from, to, converter, &output_type).is_none() {
                eprintln!("Unable to convert {from:?}");
            }
        }
        None => copy_raw(from, to),
    }
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

fn run_command(command: &str, args: Vec<&str>) -> Option<()> {
    match Command::new(command)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .spawn()
    {
        Ok(mut convert) => match convert.wait() {
            Ok(_) => Some(()),
            Err(why) => {
                eprintln!("Conversion failed: {why}");
                None
            }
        },
        Err(why) => {
            eprintln!("Conversion failed: {why}");
            None
        }
    }
}

fn convert_heics(
    from: &Path,
    to: &Path,
    converter: &ImageConverter,
    output_image_type: &ImageType,
) -> Option<()> {
    todo!()
}

pub(super) fn audio_copy_convert(
    from: &Path,
    to: &mut PathBuf,
    converter: &AudioConverter,
    mime_type: MediaType,
) {
    // Normal attachments always get converted to jpeg
    if matches!(mime_type, MediaType::Audio("caf") | MediaType::Audio("CAF")) {
        let output_type = AudioType::Mp4;
        // Update extension for conversion
        to.set_extension(output_type.to_str());
        if convert_caf(from, to, converter).is_none() {
            eprintln!("Unable to convert {from:?}");
        }
    } else {
        copy_raw(from, to);
    }
}

fn convert_caf(from: &Path, to: &Path, converter: &AudioConverter) -> Option<()> {
    // Get the path we want to copy from
    let from_path = from.to_str()?;

    // Get the path we want to write to
    let to_path = to.to_str()?;

    match converter {
        AudioConverter::AfConvert => run_command(
            "afconvert",
            vec!["-f", "mp4f", "-d", "aac", "-v", from_path, to_path],
        ),
        AudioConverter::Ffmpeg => run_command("ffmpeg", vec!["-i", from_path, to_path]),
    }
}

pub(super) fn video_copy_convert(
    from: &Path,
    to: &mut PathBuf,
    converter: &VideoConverter,
    mime_type: MediaType,
) {
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
        }
    } else {
        copy_raw(from, to);
    }
}

fn convert_mov(from: &Path, to: &Path, converter: &VideoConverter) -> Option<()> {
    // Get the path we want to copy from
    let from_path = from.to_str()?;

    // Get the path we want to write to
    let to_path = to.to_str()?;

    match converter {
        VideoConverter::Ffmpeg => run_command("ffmpeg", vec!["-i", from_path, to_path]),
    }
}
