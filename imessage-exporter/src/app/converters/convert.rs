use core::num;
use std::{
    fs::{copy, create_dir_all, read_dir},
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
    } else {
        copy_raw(from, to);
    }
    None
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
) -> Option<MediaType<'static>> {
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
            } else {
                return Some(MediaType::Image(output_type.to_str()));
            }
        }
        None => copy_raw(from, to),
    }
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
    converter: &VideoConverter,
    output_image_type: &ImageType,
) -> Option<()> {
    // Get the path we want to copy from
    let from_path = from.to_str()?;

    // Get the path we want to write to
    let to_path = to.to_str()?;

    // Frames per second in the original sticker, generated by Apple
    let fps = 10;

    // Directory to store intermediate renders
    let tmp_path = PathBuf::from("/tmp/imessage");
    // Ensure the temp directory tree exists
    if !tmp_path.exists() {
        if let Err(why) = create_dir_all(&tmp_path) {
            eprintln!("Unable to create {tmp_path:?}: {why}");
            return None;
        }
    }
    let tmp = tmp_path.to_str()?;
    println!("{tmp}");

    // HEICS format contains 4 video streams
    // The first one is the first still
    // Stream #0:0[0x1]: Video: hevc (Main) (hvc1 / 0x31637668), yuv420p(tv, smpte170m/unknown/unknown), 524x600, 1 fps, 1 tbr, 1 tbn (default)
    // The second one is the alpha mask for the first still
    // Stream #0:1[0x2]: Video: hevc (Rext) (hvc1 / 0x31637668), gray(pc), 524x600, 1 fps, 1 tbr, 1 tbn

    // The third stream is the video data
    // Stream #0:2[0x1](und): Video: hevc (Main) (hvc1 / 0x31637668), yuv420p(tv, smpte170m/unknown/unknown), 524x600, 1370 kb/s, 22.98 fps, 30 tbr, 600 tbn (default)
    run_command(
        "ffmpeg",
        vec![
            "-i",
            from_path,
            "-map",
            "0:2",
            "-y",
            &format!("{tmp}/frame_%04d.png"),
        ],
    );

    // The fourth stream is the alpha mask
    // Stream #0:3[0x2](und): Video: hevc (Rext) (hvc1 / 0x31637668), gray(pc), 524x600, 426 kb/s, 22.98 fps, 30 tbr, 600 tbn (default)
    run_command(
        "ffmpeg",
        vec![
            "-i",
            from_path,
            "-map",
            "0:3",
            "-y",
            &format!("{tmp}/alpha_%04d.png"),
        ],
    );

    // This step applies the transparency mask to the images
    let files = read_dir(tmp).ok()?;
    // let (frames, alphas): (Vec<_>, Vec<_>) = files
    //     .filter_map(Result::ok) // Handle potential errors in reading directory entries
    //     .partition(|entry| {
    //         entry
    //             .file_name()
    //             .to_str()
    //             .map(|name| name.starts_with("frame"))
    //             .unwrap_or(false)
    //     });
    let num_frames = &files.into_iter().count() / 2;
    (0..num_frames).for_each(|item| {
        println!(
            "{:?}",
            vec![
                "-i",
                &format!("{tmp}/frame_{:04}", item),
                "-i",
                &format!("{tmp}/alpha_{:04}", item),
                "-filter_complex",
                "[1:v]format=gray,geq=lum='p(X,Y)':a='p(X,Y)'[mask];[0:v][mask]alphamerge",
                &format!("{tmp}/merged_{:04}.png", item),
            ]
        );
        run_command(
            "ffmpeg",
            vec![
                "-i",
                &format!("{tmp}/frame_{:04}.png", item),
                "-i",
                &format!("{tmp}/alpha_{:04}.png", item),
                "-filter_complex",
                "[1:v]format=gray,geq=lum='p(X,Y)':a='p(X,Y)'[mask];[0:v][mask]alphamerge",
                &format!("{tmp}/merged_{:04}.png", item),
            ],
        );
    });

    // Once we have the transparent frames,
    // we use the first frame to generate a transparency palette
    // the palette_command block
    run_command(
        "ffmpeg",
        vec![
            "-i",
            &format!("{tmp}/merged_0001.png"),
            "-vf",
            "palettegen=reserve_transparent=1",
            &format!("{tmp}/palette.png"),
        ],
    );

    // Create the gif from the parts we parsed above
    run_command(
        "ffmpeg",
        vec![
            "-i",
            &format!("{tmp}/merged_%04d.png"),
            "-i",
            &format!("{tmp}/palette.png"),
            "-lavfi",
            &format!("fps={fps},paletteuse=alpha_threshold=128"),
            "-gifflags",
            "-offsetting",
            to_path,
        ],
    );

    None
}

pub(super) fn audio_copy_convert(
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
    } else {
        copy_raw(from, to);
    }
    None
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
    } else {
        copy_raw(from, to);
    }
    None
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
