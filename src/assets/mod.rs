mod lazy_theme_set;

use anyhow::Error;
use anyhow::Result;

pub(crate) use lazy_theme_set::LazyThemeSet;

pub(crate) fn load_themes() -> LazyThemeSet {
    get_integrated_themeset()
}

pub(crate) fn load_syntaxes() -> syntect::parsing::SyntaxSet {
    from_binary(get_serialized_integrated_syntaxset(), COMPRESS_SYNTAXES)
}

pub(crate) fn to_anstyle_color(color: syntect::highlighting::Color) -> Option<anstyle::Color> {
    if color.a == 0 {
        // Themes can specify one of the user-configurable terminal colors by
        // encoding them as #RRGGBBAA with AA set to 00 (transparent) and RR set
        // to the 8-bit color palette number. The built-in themes ansi, base16,
        // and base16-256 use this.
        Some(match color.r {
            // For the first 8 colors, use the Color enum to produce ANSI escape
            // sequences using codes 30-37 (foreground) and 40-47 (background).
            // For example, red foreground is \x1b[31m. This works on terminals
            // without 256-color support.
            0x00 => anstyle::AnsiColor::Black.into(),
            0x01 => anstyle::AnsiColor::Red.into(),
            0x02 => anstyle::AnsiColor::Green.into(),
            0x03 => anstyle::AnsiColor::Yellow.into(),
            0x04 => anstyle::AnsiColor::Blue.into(),
            0x05 => anstyle::AnsiColor::Magenta.into(),
            0x06 => anstyle::AnsiColor::Cyan.into(),
            0x07 => anstyle::AnsiColor::White.into(),
            // For all other colors, use Fixed to produce escape sequences using
            // codes 38;5 (foreground) and 48;5 (background). For example,
            // bright red foreground is \x1b[38;5;9m. This only works on
            // terminals with 256-color support.
            //
            // TODO: When ansi_term adds support for bright variants using codes
            // 90-97 (foreground) and 100-107 (background), we should use those
            // for values 0x08 to 0x0f and only use Fixed for 0x10 to 0xff.
            n => anstyle::Ansi256Color(n).into(),
        })
    } else if color.a == 1 {
        // Themes can specify the terminal's default foreground/background color
        // (i.e. no escape sequence) using the encoding #RRGGBBAA with AA set to
        // 01. The built-in theme ansi uses this.
        None
    } else {
        Some(anstyle::RgbColor(color.r, color.g, color.b).into())
    }
}

/// Lazy-loaded syntaxes are already compressed, and we don't want to compress
/// already compressed data.
const COMPRESS_SYNTAXES: bool = false;

/// We don't want to compress our [`LazyThemeSet`] since the lazy-loaded themes
/// within it are already compressed, and compressing another time just makes
/// performance suffer
const COMPRESS_THEMES: bool = false;

/// Compress for size of ~40 kB instead of ~200 kB without much difference in
/// performance due to lazy-loading
const COMPRESS_LAZY_THEMES: bool = true;

/// Compress for size of ~10 kB instead of ~120 kB
const COMPRESS_ACKNOWLEDGEMENTS: bool = true;

fn get_serialized_integrated_syntaxset() -> &'static [u8] {
    include_bytes!("../../assets/syntaxes.bin")
}

fn get_integrated_themeset() -> LazyThemeSet {
    from_binary(include_bytes!("../../assets/themes.bin"), COMPRESS_THEMES)
}

pub(crate) fn get_acknowledgements() -> String {
    from_binary(
        include_bytes!("../../assets/acknowledgements.bin"),
        COMPRESS_ACKNOWLEDGEMENTS,
    )
}

fn from_binary<T: serde::de::DeserializeOwned>(v: &[u8], compressed: bool) -> T {
    asset_from_contents(v, "n/a", compressed)
        .expect("data integrated in binary is never faulty, but make sure `compressed` is in sync!")
}

fn asset_from_contents<T: serde::de::DeserializeOwned>(
    contents: &[u8],
    description: &str,
    compressed: bool,
) -> Result<T, Error> {
    let config = bincode::config::legacy();
    if compressed {
        let mut reader = flate2::read::ZlibDecoder::new(contents);
        bincode::serde::decode_from_std_read(&mut reader, config)
    } else {
        bincode::serde::decode_from_slice(contents, config).map(|(a, _)| a)
    }
    .map_err(|_| anyhow::format_err!("Could not parse {}", description))
}
