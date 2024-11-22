use std::{
    fmt::{Display, Formatter, Result},
    fs::create_dir_all,
    path::Path,
    process::{Command, Stdio},
};

#[derive(Debug, PartialEq, Eq)]
pub enum ImageType {
    Jpeg,
    Gif,
    Png,
}

impl ImageType {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Jpeg => "jpeg",
            Self::Gif => "gif",
            Self::Png => "png",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum VideoType {
    Mp4,
    Mov,
}

impl VideoType {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::Mov => "mov",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum AudioType {
    Caf,
    Mp3,
}

impl AudioType {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Caf => "caf",
            Self::Mp3 => "mp3",
        }
    }
}

pub trait Converter {
    fn determine() -> Option<Self>
    where
        Self: Sized;
}

#[derive(Debug, PartialEq, Eq)]
pub enum ImageConverter {
    Sips,
    Imagemagick,
}

impl Converter for ImageConverter {
    /// Determine the converter type for the current shell environment
    fn determine() -> Option<ImageConverter> {
        if exists("sips") {
            return Some(ImageConverter::Sips);
        }
        if exists("magick") {
            return Some(ImageConverter::Imagemagick);
        }
        eprintln!("No HEIC converter found, image attachments will not be converted!");
        None
    }
}

impl Display for ImageConverter {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ImageConverter::Sips => write!(f, "sips"),
            ImageConverter::Imagemagick => write!(f, "imagemagick"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum AudioConverter {
    AfConvert,
    Ffmpeg,
}

impl Converter for AudioConverter {
    fn determine() -> Option<AudioConverter> {
        if exists("afconvert") {
            return Some(AudioConverter::AfConvert);
        }
        if exists("ffmpeg") {
            return Some(AudioConverter::Ffmpeg);
        }
        eprintln!("No CAF converter found, audio attachments will not be converted!");
        None
    }
}

impl Display for AudioConverter {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            AudioConverter::AfConvert => write!(f, "afconvert"),
            AudioConverter::Ffmpeg => write!(f, "ffmpeg"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum VideoConverter {
    Ffmpeg,
}

impl Converter for VideoConverter {
    fn determine() -> Option<VideoConverter> {
        if exists("ffmpeg") {
            return Some(VideoConverter::Ffmpeg);
        }
        eprintln!("No MOV converter found, video attachments will not be converted!");
        None
    }
}

impl Display for VideoConverter {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            VideoConverter::Ffmpeg => write!(f, "ffmpeg"),
        }
    }
}

/// Determine if a shell program exists on the system
#[cfg(not(target_family = "windows"))]
fn exists(name: &str) -> bool {
    if let Ok(process) = Command::new("type")
        .args(vec![name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .spawn()
    {
        if let Ok(output) = process.wait_with_output() {
            return output.status.success();
        }
    };
    false
}

/// Determine if a shell program exists on the system
#[cfg(target_family = "windows")]
fn exists(name: &str) -> bool {
    Command::new("where")
        .arg(name)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
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
pub fn convert_heic(
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
            match Command::new("sips")
                .args(vec![
                    "-s",
                    "format",
                    output_image_type.to_str(),
                    from_path,
                    "-o",
                    to_path,
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .stdin(Stdio::null())
                .spawn()
            {
                Ok(mut sips) => match sips.wait() {
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
        ImageConverter::Imagemagick =>
        // Build the command
        {
            match Command::new("magick")
                .args(vec![from_path, to_path])
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
    };

    Some(())
}

#[cfg(test)]
mod test {
    use super::exists;

    #[test]
    fn can_find_program() {
        assert!(exists("ls"));
    }

    #[test]
    fn can_miss_program() {
        assert!(!exists("fake_name"));
    }
}
