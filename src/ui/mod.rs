use bevy::render::color::Color;

pub mod button;

pub struct ColorScheme;

impl ColorScheme {
    pub const TEXT: Color = Color::rgb_linear(0.85, 1.0, 0.85);
    pub const TEXT_DARK: Color = Color::rgb_linear(53. / 255., 186. / 255., 243. / 255.);
    pub const TEXT_DIM: Color = Color::rgb_linear(0.6, 0.6, 0.6);
    pub const TEXT_HIGHLIGHT: Color = Color::rgb_linear(0.94, 0.84, 0.);
}
