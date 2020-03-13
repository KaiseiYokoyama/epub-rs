use std::str::FromStr;
use std::convert::TryFrom;
use std::path::PathBuf;

/// https://imagedrive.github.io/spec/epub30-publications.xhtml#tbl-core-media-types
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum MediaType {
    Image(ImageType),
    Application(ApplicationType),
    Audio(AudioType),
    Text(TextType),
    NonCoreMediaType(String),
}

impl FromStr for MediaType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vec:Vec<&str> = s.split('/').collect();
        if let (Some(prefix), Some(detail)) = (vec.get(0), vec.get(1)) {
            match prefix {
                &"image" => ImageType::from_str(detail).map(|i| MediaType::Image(i)),
                &"application" => ApplicationType::from_str(detail).map(|i| MediaType::Application(i)),
                &"audio" => AudioType::from_str(detail).map(|i| MediaType::Audio(i)),
                &"text" => TextType::from_str(detail).map(|i| MediaType::Text(i)),
                _ => Ok(MediaType::NonCoreMediaType(s.to_string()))
            }
        } else {Err(())}
    }
}

impl TryFrom<&PathBuf> for MediaType {
    type Error = ();

    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        Err(())
            .or(
                ImageType::try_from(value).map(|i| MediaType::Image(i))
            )
            .or(
                ApplicationType::try_from(value).map(|i| MediaType::Application(i))
            )
            .or(
                AudioType::try_from(value).map(|i| MediaType::Audio(i))
            )
            .or(
                TextType::try_from(value).map(|i| MediaType::Text(i))
            )
    }
}

impl Default for MediaType {
    fn default() -> Self {
        MediaType::Application(ApplicationType::XhtmlXml)
    }
}

impl ToString for MediaType {
    fn to_string(&self) -> String {
        match self {
            MediaType::Image(t) => format!("image/{}", &t.to_string()),
            MediaType::Application(t) => format!("application/{}", &t.to_string()),
            MediaType::Audio(t) => format!("audio/{}", &t.to_string()),
            MediaType::Text(t) => format!("text/{}", t.to_string()),
            MediaType::NonCoreMediaType(s) => s.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ImageType {
    GIF,
    JPEG,
    PNG,
    SVG,
}

impl FromStr for ImageType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "gif" => Ok(ImageType::GIF),
            "jpeg" => Ok(ImageType::JPEG),
            "png" => Ok(ImageType::PNG),
            "svg+xml" => Ok(ImageType::SVG),
            _ => Err(())
        }
    }
}

impl TryFrom<&PathBuf> for ImageType {
    type Error = ();

    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        let ext = value.extension()
            .map(|os| os.to_str())
            .flatten()
            .ok_or(())?;

        match ext {
            "gif" => Ok(ImageType::GIF),
            "jpeg" | "jpg" | "jpe" => Ok(ImageType::JPEG),
            "png" => Ok(ImageType::PNG),
            "svg" | "svgz" => Ok(ImageType::SVG),
            _ => Err(())
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

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ApplicationType {
    /// XHTML Content Document と EPUB Navigation Document
    XhtmlXml,
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
            "xhtml+xml" => Ok(ApplicationType::XhtmlXml),
            "vnd.ms-opentype" => Ok(ApplicationType::OpenType),
            "font-woff" => Ok(ApplicationType::WOFF),
            "smil+xml" => Ok(ApplicationType::MediaOverlays),
            "pls+xml" => Ok(ApplicationType::PLS),
            _ => Err(())
        }
    }
}

impl TryFrom<&PathBuf> for ApplicationType {
    type Error = ();

    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        let ext = value.extension()
            .map(|os| os.to_str())
            .flatten()
            .ok_or(())?;

        match ext {
            "xhtml" | "xht" => Ok(ApplicationType::XhtmlXml),
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
            ApplicationType::XhtmlXml => "xhtml+xml",
            ApplicationType::OpenType => "vnd.ms-opentype",
            ApplicationType::WOFF => "font-woff",
            ApplicationType::MediaOverlays => "smil+xml",
            ApplicationType::PLS => "pls+xml",
        }.to_string()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum AudioType {
    /// MP3 オーディオ
    MPEG,
    /// MP4 コンテナを使用している AAC LC オーディオ
    MP4,
}

impl FromStr for AudioType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mpeg" => Ok(AudioType::MPEG),
            "mp4" => Ok(AudioType::MP4),
            _ => Err(())
        }
    }
}

impl TryFrom<&PathBuf> for AudioType {
    type Error = ();

    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        let ext = value.extension()
            .map(|os| os.to_str())
            .flatten()
            .ok_or(())?;

        match ext {
            "mp3" => Ok(AudioType::MPEG),
            "aac" | "mp4" => Ok(AudioType::MP4),
            _ => Err(())
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

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum TextType {
    CSS,
    JS,
}

impl FromStr for TextType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "css" => Ok(TextType::CSS),
            "javascript" => Ok(TextType::JS),
            _ => Err(())
        }
    }
}

impl TryFrom<&PathBuf> for TextType {
    type Error = ();

    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        let ext = value.extension()
            .map(|os| os.to_str())
            .flatten()
            .ok_or(())?;

        match ext {
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