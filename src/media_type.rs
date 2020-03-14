use crate::EPUBError;
use std::str::FromStr;
use std::convert::TryFrom;
use std::path::PathBuf;

/// https://imagedrive.github.io/spec/epub30-publications.xhtml#tbl-core-media-types
#[derive(Clone, PartialEq, Debug)]
pub enum MediaType {
    Image(ImageType),
    Application(ApplicationType),
    Audio(AudioType),
    Text(TextType),
}

impl FromStr for MediaType {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Option::None
            .or(
                ImageType::from_str(s).ok().map(|t| MediaType::Image(t))
            )
            .or(
                ApplicationType::from_str(s).ok().map(|t| MediaType::Application(t))
            )
            .or(
                AudioType::from_str(s).ok().map(|t| MediaType::Audio(t))
            )
            .or(
                TextType::from_str(s).ok().map(|t| MediaType::Text(t))
            )
            .ok_or(
                EPUBError::MediaTypeError {
                    err_msg: format!("Invalid extension: {}", s)
                }.into()
            )
    }
}

impl Default for MediaType {
    fn default() -> Self {
        MediaType::Application(ApplicationType::XHTML)
    }
}

impl ToString for MediaType {
    fn to_string(&self) -> String {
        match self {
            MediaType::Image(t) => format!("image/{}", &t.to_string()),
            MediaType::Application(t) => format!("application/{}", &t.to_string()),
            MediaType::Audio(t) => format!("audio/{}", &t.to_string()),
            MediaType::Text(t) => format!("text/{}", t.to_string()),
        }
    }
}

impl TryFrom<&PathBuf> for MediaType {
    type Error = failure::Error;

    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        let ext = value.extension()
            .map(|s| s.to_str().unwrap_or(""))
            .unwrap_or("");
        Ok(MediaType::from_str(&ext)?)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ImageType {
    GIF,
    JPEG,
    PNG,
    SVG,
}

impl FromStr for ImageType {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "gif" => Ok(ImageType::GIF),
            "jpeg" | "jpg" | "jpe" => Ok(ImageType::JPEG),
            "png" => Ok(ImageType::PNG),
            "svg" | "svgz" => Ok(ImageType::SVG),
            _ =>
                Err(EPUBError::MediaTypeError {
                    err_msg: format!("Invalid extension: {}", s)
                }.into())
        }
    }
}

impl ToString for ImageType {
    fn to_string(&self) -> String {
        match self {
            ImageType::GIF => "gif",
            ImageType::JPEG => "jpeg",
            ImageType::PNG => "png",
            ImageType::SVG => "svg+xml",
        }.to_string()
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ApplicationType {
    /// XHTML Content Document と EPUB Navigation Document
    XHTML,
    /// OpenType Font
    OpenType,
    /// WOFF Font
    WOFF,
    /// EPUB Media Overlay Document
    MediaOverlays,
    /// Text-to-Speech (TTS) 発音語彙
    PLS,
}

impl FromStr for ApplicationType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "xhtml" | "xht" => Ok(ApplicationType::XHTML),
            "otf" | "otc" | "ttf" | "ttc" => Ok(ApplicationType::OpenType),
            "woff" | "woff2" => Ok(ApplicationType::WOFF),
            "smil" => Ok(ApplicationType::MediaOverlays),
            "pls" => Ok(ApplicationType::PLS),
            _ => Err(())
        }
    }
}

impl ToString for ApplicationType {
    fn to_string(&self) -> String {
        match self {
            ApplicationType::XHTML => "xhtml+xml",
            ApplicationType::OpenType => "vnd.ms-opentype",
            ApplicationType::WOFF => "font-woff",
            ApplicationType::MediaOverlays => "smil+xml",
            ApplicationType::PLS => "pls+xml",
        }.to_string()
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum AudioType {
    /// MP3 オーディオ
    MPEG,
    /// MP4 コンテナを使用している AAC LC オーディオ
    MP4,
}

impl FromStr for AudioType {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mp3" => Ok(AudioType::MPEG),
            "aac" | "mp4" => Ok(AudioType::MP4),
            _ =>
                Err(EPUBError::MediaTypeError {
                    err_msg: format!("Invalid extension: {}", s)
                }.into())
        }
    }
}

impl ToString for AudioType {
    fn to_string(&self) -> String {
        match self {
            AudioType::MPEG => "mpeg",
            AudioType::MP4 => "mp4",
        }.to_string()
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum TextType {
    CSS,
    JS,
}

impl FromStr for TextType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "css" => Ok(TextType::CSS),
            "js" => Ok(TextType::JS),
            _ => Err(())
        }
    }
}

impl ToString for TextType {
    fn to_string(&self) -> String {
        match self {
            TextType::CSS => "css",
            TextType::JS => "javascript",
        }.to_string()
    }
}