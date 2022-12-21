use anstyle::{Color, RgbColor, Style, AnsiColor, XTermColor};

#[test]
fn test_rgb_color_output() {
    let style = Style::new()
        .fg_color(Some(Color::Rgb(RgbColor(255, 0, 170))))
        .bg_color(Some(Color::Rgb(RgbColor(0, 255, 0))));

    let mut roff_style = anstyle_roff::to_roff(style);
    roff_style.text("test".to_owned());
    snapbox::assert_eq_path("tests/roffs/rgb_color.roff", roff_style.render());
}

#[test]
fn test_ansi_color_output() {
    let style = Style::new()
        .fg_color(Some(Color::Ansi(AnsiColor::Red)))
        .bg_color(Some(Color::Ansi(AnsiColor::Blue)));

    let mut roff_style = anstyle_roff::to_roff(style);
    roff_style.text("test".to_owned());
    snapbox::assert_eq_path("tests/roffs/ansi_color.roff", roff_style.render());
}


#[test]
fn test_xterm_color_output() {
    let style = Style::new()
        .fg_color(Some(Color::XTerm(XTermColor(1))))
        .bg_color(Some(Color::XTerm(XTermColor(4))));

    let mut roff_style = anstyle_roff::to_roff(style);
    roff_style.text("test".to_owned());
    snapbox::assert_eq_path("tests/roffs/ansi_color.roff", roff_style.render());
}


#[test]
fn test_bold_output() {
    let style = Style::new().bold();
    let mut roff_style = anstyle_roff::to_roff(style);

    roff_style.text("test".to_owned());
    snapbox::assert_eq_path("tests/roffs/bold.roff", roff_style.render());
}

#[test]
fn test_italic_output() {
    let style = Style::new().italic();

    let mut roff_style = anstyle_roff::to_roff(style);
    roff_style.text("test".to_owned());
    snapbox::assert_eq_path("tests/roffs/italic.roff", roff_style.render());
}
