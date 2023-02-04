//! Convert from ansi stylings to ROFF Control Lines
//! Currently uses [roff](https://docs.rs/roff/0.2.1/roff/) as the engine for generating
//! roff output.

mod style_stream;
use anstyle::{Color, RgbColor};
use roff::{bold, italic, Roff};
use style_stream::StyledStr;

/// Generate A RoffStyle from Style
///
/// ```rust
/// use anstyle::{Color, RgbColor};
///
/// let text = "\u{1b}[44;31mtest\u{1b}[0m";
///
/// let roff_doc = anstyle_roff::to_roff(text);
/// let expected = r#".gcolor red
/// .fcolor blue
/// test
/// "#;
///
/// assert_eq!(roff_doc.to_roff(), expected);
/// ```
pub fn to_roff(styled_text: &str) -> Roff {
    let mut roff_docs = vec![];
    for styled_str in style_stream::styled_stream(styled_text) {
        roff_docs.push(as_roff(&styled_str))
    }

    let mut doc = Roff::new();
    doc.extend(roff_docs);
    doc
}

fn as_roff(styled: &StyledStr) -> Roff {
    let style = styled.style;
    let mut doc = Roff::new();
    doc.extend([
        set_color((&style.get_fg_color(), &style.get_bg_color())),
        set_effects(styled),
    ]);
    doc
}

fn set_effects(styled: &StyledStr) -> Roff {
    // Roff (the crate) only supports these inline commands
    //  - Bold
    //  - Italic
    //  - Roman (plain text)
    // If we want more support, or even support combined formats, we will need
    // to push improvements to roff upstream or implement a more thorough roff crate
    // perhaps by spinning off some of this code
    let effects = styled.style.get_effects();
    let mut doc = Roff::new();

    if effects.contains(anstyle::Effects::BOLD) {
        doc.text(vec![bold(styled.text)]);
        return doc;
    }

    if effects.contains(anstyle::Effects::ITALIC) {
        doc.text(vec![italic(styled.text)]);
        return doc;
    }

    if effects.is_plain() {
        doc.text(vec![roff::roman(styled.text)]);
        return doc;
    }

    doc
}

/// Set the foreground, background color
fn set_color(colors: (&Option<Color>, &Option<Color>)) -> Roff {
    let mut doc = Roff::new();
    // Set foreground
    add_color_to_roff(&mut doc, control_requests::FOREGROUND, colors.0);
    // Set background
    add_color_to_roff(&mut doc, control_requests::BACKGROUND, colors.1);
    doc
}

fn add_color_to_roff(doc: &mut Roff, control_request: &str, color: &Option<Color>) {
    match color {
        Some(Color::Rgb(c)) => {
            let name = rgb_name(c);
            doc.control(
                control_requests::CREATE_COLOR,
                vec![name.as_str(), "rgb", as_hex(c).as_str()],
            )
            .control(control_request, vec![name.as_str()]);
        }

        Some(Color::Ansi(c)) => {
            doc.control(control_request, vec![ansi_color_to_roff(c)]);
        }
        _ => {
            // TODO: get rid of "default" hardcoded str?
            doc.control(control_request, vec!["default"]);
        }
    }
}

fn rgb_name(c: &RgbColor) -> String {
    format!("hex_{}", as_hex(c).as_str())
}

fn as_hex(rgb: &RgbColor) -> String {
    let val: usize = ((rgb.0 as usize) << 16) + ((rgb.1 as usize) << 8) + (rgb.2 as usize);
    format!("#{:06x}", val)
}

fn ansi_color_to_roff(color: &anstyle::AnsiColor) -> &'static str {
    match color {
        anstyle::AnsiColor::Black => "black",
        anstyle::AnsiColor::Red => "red",
        anstyle::AnsiColor::Green => "green",
        anstyle::AnsiColor::Yellow => "yellow",
        anstyle::AnsiColor::Blue => "blue",
        anstyle::AnsiColor::Magenta => "magenta",
        anstyle::AnsiColor::Cyan => "cyan",
        anstyle::AnsiColor::White => "white",
        _ => "default",
    }
}

/// Static Strings defining ROFF Control Requests
mod control_requests {
    /// Control to Create a Color definition
    pub const CREATE_COLOR: &'static str = "defcolor";
    /// Roff control request to set background color (fill color)
    pub const BACKGROUND: &'static str = "fcolor";
    /// Roff control request to set foreground color (glyph color)
    pub const FOREGROUND: &'static str = "gcolor";
}

/// Default AsciiColors supported by roff
#[cfg(test)]
mod tests {
    use super::*;
    use anstyle::RgbColor;

    #[test]
    fn to_hex() {
        assert_eq!(as_hex(&RgbColor(0, 0, 0)).as_str(), "#000000");
        assert_eq!(as_hex(&RgbColor(255, 0, 0)).as_str(), "#ff0000");
        assert_eq!(as_hex(&RgbColor(0, 255, 0)).as_str(), "#00ff00");
        assert_eq!(as_hex(&RgbColor(0, 0, 255)).as_str(), "#0000ff");
    }
}
