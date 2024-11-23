/*!
 Contains data structures used to describe file converters and associated types.
*/

use std::{
    fmt::{Display, Formatter, Result},
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
}

impl VideoType {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum AudioType {
    Mp4,
}

impl AudioType {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
        }
    }
}

pub trait Converter {
    fn determine() -> Option<Self>
    where
        Self: Sized;
}

#[derive(Debug, PartialEq, Eq)]
/// Program used to convert/encode images
pub enum ImageConverter {
    /// macOS Builtin
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
/// Program used to convert/encode audio
pub enum AudioConverter {
    /// macOS Builtin
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
/// Program used to convert/encode videos
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
