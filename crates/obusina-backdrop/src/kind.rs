//! The kinds of picture the store takes, told by their first bytes.

/// A kind of picture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// PNG.
    Png,
    /// JPEG.
    Jpeg,
    /// WebP.
    Webp,
}

impl Kind {
    /// The kind of the picture in `bytes`, by its signature; `None` for
    /// another kind. The name of the file is not asked: a `.jpg` that holds
    /// a PNG is a PNG.
    pub fn of(bytes: &[u8]) -> Option<Kind> {
        if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            Some(Kind::Png)
        } else if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
            Some(Kind::Jpeg)
        } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
            Some(Kind::Webp)
        } else {
            None
        }
    }

    /// The extension of a file of this kind.
    pub fn extension(self) -> &'static str {
        match self {
            Kind::Png => "png",
            Kind::Jpeg => "jpg",
            Kind::Webp => "webp",
        }
    }

    /// The extensions a dialog of files offers: every kind, and the other
    /// spelling of JPEG.
    pub const EXTENSIONS: [&'static str; 4] = ["png", "jpg", "jpeg", "webp"];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_kind_is_told_by_its_signature() {
        assert_eq!(Kind::of(b"\x89PNG\r\n\x1a\n...."), Some(Kind::Png));
        assert_eq!(Kind::of(&[0xff, 0xd8, 0xff, 0xe0]), Some(Kind::Jpeg));
        assert_eq!(Kind::of(b"RIFF\0\0\0\0WEBPVP8 "), Some(Kind::Webp));
        assert_eq!(Kind::of(b"GIF89a"), None);
        assert_eq!(Kind::of(b""), None);
    }
}
