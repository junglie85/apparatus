use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Copy, PartialEq)]
pub struct Color([u8; 4]); // [a, r, g, b]

impl Color {
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self([a, r, g, b])
    }

    pub const fn r(&self) -> u8 {
        self.0[1]
    }

    pub const fn g(&self) -> u8 {
        self.0[2]
    }

    pub const fn b(&self) -> u8 {
        self.0[3]
    }

    pub const fn a(&self) -> u8 {
        self.0[0]
    }

    pub fn linear_blend(src: Self, dst: Self) -> Self {
        let t = src.a() as f32 / 255.0;
        let r = (Color::interpolate_scalar(src.r() as f32 / 255.0, dst.r() as f32 / 255.0, t)
            * 255.0) as u8;
        let g = (Color::interpolate_scalar(src.g() as f32 / 255.0, dst.g() as f32 / 255.0, t)
            * 255.0) as u8;
        let b = (Color::interpolate_scalar(src.b() as f32 / 255.0, dst.b() as f32 / 255.0, t)
            * 255.0) as u8;

        Self::rgba(r, g, b, 255)
    }

    pub fn interpolate_scalar(src: f32, dst: f32, t: f32) -> f32 {
        dst * (1.0 - t) + src * t
        // Or: `dst + (src - dst) * t`.
    }
}

impl From<Color> for u32 {
    fn from(color: Color) -> Self {
        u32::from_be_bytes(color.0)
    }
}

impl Debug for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Color{{r={:<3} g={:<3} b={:<3} a={:<3}}}",
            self.r(),
            self.g(),
            self.b(),
            self.a()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_has_rgba_components() {
        let color = Color::rgba(50, 100, 150, 200);

        assert_eq!(50, color.r());
        assert_eq!(100, color.g());
        assert_eq!(150, color.b());
        assert_eq!(200, color.a());
    }

    #[test]
    fn color_can_be_represented_in_argb_by_u32() {
        let color = Color::rgba(64, 128, 192, 255);
        let expected = (255 << 24) | (64 << 16) | (128 << 8) | 192;

        assert_eq!(expected, Into::<u32>::into(color));
    }

    #[test]
    fn interpolate_between_two_values() {
        let a = 10.0;
        let b = 50.0;
        let t = 0.75;

        assert_eq!(Color::interpolate_scalar(a, b, t), 20.0);
    }

    #[test]
    fn linear_blend_red_color_full_opacity_onto_blue_color_is_red() {
        let red = css::RED;
        let blue = css::BLUE;

        assert_eq!(Color::linear_blend(red, blue), red);
    }

    #[test]
    fn linear_blend_red_color_full_transparency_onto_blue_color_is_blue() {
        let red = Color::rgba(255, 0, 0, 0);
        let blue = css::BLUE;

        assert_eq!(Color::linear_blend(red, blue), blue);
    }
}

pub mod css {
    use crate::color::Color;

    pub const ALICEBLUE: Color = Color::rgba(240, 248, 255, 255);
    pub const ANTIQUEWHITE: Color = Color::rgba(250, 235, 215, 255);
    pub const AQUA: Color = Color::rgba(0, 255, 255, 255);
    pub const AQUAMARINE: Color = Color::rgba(127, 255, 212, 255);
    pub const AZURE: Color = Color::rgba(240, 255, 255, 255);
    pub const BEIGE: Color = Color::rgba(245, 245, 220, 255);
    pub const BISQUE: Color = Color::rgba(255, 228, 196, 255);
    pub const BLACK: Color = Color::rgba(0, 0, 0, 255);
    pub const BLANCHEDALMOND: Color = Color::rgba(255, 235, 205, 255);
    pub const BLUE: Color = Color::rgba(0, 0, 255, 255);
    pub const BLUEVIOLET: Color = Color::rgba(138, 43, 226, 255);
    pub const BROWN: Color = Color::rgba(165, 42, 42, 255);
    pub const BURLYWOOD: Color = Color::rgba(222, 184, 135, 255);
    pub const CADETBLUE: Color = Color::rgba(95, 158, 160, 255);
    pub const CHARTREUSE: Color = Color::rgba(127, 255, 0, 255);
    pub const CHOCOLATE: Color = Color::rgba(210, 105, 30, 255);
    pub const CORAL: Color = Color::rgba(255, 127, 80, 255);
    pub const CORNFLOWERBLUE: Color = Color::rgba(100, 149, 237, 255);
    pub const CORNSILK: Color = Color::rgba(255, 248, 220, 255);
    pub const CRIMSON: Color = Color::rgba(220, 20, 60, 255);
    pub const CYAN: Color = Color::rgba(0, 255, 255, 255);
    pub const DARKBLUE: Color = Color::rgba(0, 0, 139, 255);
    pub const DARKCYAN: Color = Color::rgba(0, 139, 139, 255);
    pub const DARKGOLDENROD: Color = Color::rgba(184, 134, 11, 255);
    pub const DARKGRAY: Color = Color::rgba(169, 169, 169, 255);
    pub const DARKGREEN: Color = Color::rgba(0, 100, 0, 255);
    pub const DARKGREY: Color = Color::rgba(169, 169, 169, 255);
    pub const DARKKHAKI: Color = Color::rgba(189, 183, 107, 255);
    pub const DARKMAGENTA: Color = Color::rgba(139, 0, 139, 255);
    pub const DARKOLIVEGREEN: Color = Color::rgba(85, 107, 47, 255);
    pub const DARKORANGE: Color = Color::rgba(255, 140, 0, 255);
    pub const DARKORCHID: Color = Color::rgba(153, 50, 204, 255);
    pub const DARKRED: Color = Color::rgba(139, 0, 0, 255);
    pub const DARKSALMON: Color = Color::rgba(233, 150, 122, 255);
    pub const DARKSEAGREEN: Color = Color::rgba(143, 188, 143, 255);
    pub const DARKSLATEBLUE: Color = Color::rgba(72, 61, 139, 255);
    pub const DARKSLATEGRAY: Color = Color::rgba(47, 79, 79, 255);
    pub const DARKTURQUOISE: Color = Color::rgba(0, 206, 209, 255);
    pub const DARKVIOLET: Color = Color::rgba(148, 0, 211, 255);
    pub const DEEPPINK: Color = Color::rgba(255, 20, 147, 255);
    pub const DEEPSKYBLUE: Color = Color::rgba(0, 191, 255, 255);
    pub const DIMGRAY: Color = Color::rgba(105, 105, 105, 255);
    pub const DODGERBLUE: Color = Color::rgba(30, 144, 255, 255);
    pub const FIREBRICK: Color = Color::rgba(178, 34, 34, 255);
    pub const FLORALWHITE: Color = Color::rgba(255, 250, 240, 255);
    pub const FORESTGREEN: Color = Color::rgba(34, 139, 34, 255);
    pub const FUCHSIA: Color = Color::rgba(255, 0, 255, 255);
    pub const GAINSBORO: Color = Color::rgba(220, 220, 220, 255);
    pub const GHOSTWHITE: Color = Color::rgba(248, 248, 255, 255);
    pub const GOLD: Color = Color::rgba(255, 215, 0, 255);
    pub const GOLDENROD: Color = Color::rgba(218, 165, 32, 255);
    pub const GRAY: Color = Color::rgba(128, 128, 128, 255);
    pub const GREEN: Color = Color::rgba(0, 128, 0, 255);
    pub const GREENYELLOW: Color = Color::rgba(173, 255, 47, 255);
    pub const GREY: Color = Color::rgba(128, 128, 128, 255);
    pub const HONEYDEW: Color = Color::rgba(240, 255, 240, 255);
    pub const HOTPINK: Color = Color::rgba(255, 105, 180, 255);
    pub const INDIANRED: Color = Color::rgba(205, 92, 92, 255);
    pub const INDIGO: Color = Color::rgba(75, 0, 130, 255);
    pub const IVORY: Color = Color::rgba(255, 255, 240, 255);
    pub const KHAKI: Color = Color::rgba(240, 230, 140, 255);
    pub const LAVENDER: Color = Color::rgba(230, 230, 250, 255);
    pub const LAVENDERBLUSH: Color = Color::rgba(255, 240, 245, 255);
    pub const LAWNGREEN: Color = Color::rgba(124, 252, 0, 255);
    pub const LEMONCHIFFON: Color = Color::rgba(255, 250, 205, 255);
    pub const LIGHTBLUE: Color = Color::rgba(173, 216, 230, 255);
    pub const LIGHTCORAL: Color = Color::rgba(240, 128, 128, 255);
    pub const LIGHTCYAN: Color = Color::rgba(224, 255, 255, 255);
    pub const LIGHTGOLDENRODYELLOW: Color = Color::rgba(250, 250, 210, 255);
    pub const LIGHTGRAY: Color = Color::rgba(211, 211, 211, 255);
    pub const LIGHTGREEN: Color = Color::rgba(144, 238, 144, 255);
    pub const LIGHTGREY: Color = Color::rgba(211, 211, 211, 255);
    pub const LIGHTPINK: Color = Color::rgba(255, 182, 193, 255);
    pub const LIGHTSALMON: Color = Color::rgba(255, 160, 122, 255);
    pub const LIGHTSEAGREEN: Color = Color::rgba(32, 178, 170, 255);
    pub const LIGHTSKYBLUE: Color = Color::rgba(135, 206, 250, 255);
    pub const LIGHTSLATEGRAY: Color = Color::rgba(119, 136, 153, 255);
    pub const LIGHTSTEELBLUE: Color = Color::rgba(176, 196, 222, 255);
    pub const LIGHTYELLOW: Color = Color::rgba(255, 255, 224, 255);
    pub const LIME: Color = Color::rgba(0, 255, 0, 255);
    pub const LIMEGREEN: Color = Color::rgba(50, 205, 50, 255);
    pub const LINEN: Color = Color::rgba(250, 240, 230, 255);
    pub const MAGENTA: Color = Color::rgba(255, 0, 255, 255);
    pub const MAROON: Color = Color::rgba(128, 0, 0, 255);
    pub const MEDIUMAQUAMARINE: Color = Color::rgba(102, 205, 170, 255);
    pub const MEDIUMBLUE: Color = Color::rgba(0, 0, 205, 255);
    pub const MEDIUMORCHID: Color = Color::rgba(186, 85, 211, 255);
    pub const MEDIUMPURPLE: Color = Color::rgba(147, 112, 219, 255);
    pub const MEDIUMSEAGREEN: Color = Color::rgba(60, 179, 113, 255);
    pub const MEDIUMSLATEBLUE: Color = Color::rgba(123, 104, 238, 255);
    pub const MEDIUMSPRINGGREEN: Color = Color::rgba(0, 250, 154, 255);
    pub const MEDIUMTURQUOISE: Color = Color::rgba(72, 209, 204, 255);
    pub const MEDIUMVIOLETRED: Color = Color::rgba(199, 21, 133, 255);
    pub const MIDNIGHTBLUE: Color = Color::rgba(25, 25, 112, 255);
    pub const MINTCREAM: Color = Color::rgba(245, 255, 250, 255);
    pub const MISTYROSE: Color = Color::rgba(255, 228, 225, 255);
    pub const MOCCASIN: Color = Color::rgba(255, 228, 181, 255);
    pub const NAVAJOWHITE: Color = Color::rgba(255, 222, 173, 255);
    pub const NAVY: Color = Color::rgba(0, 0, 128, 255);
    pub const OLDLACE: Color = Color::rgba(253, 245, 230, 255);
    pub const OLIVE: Color = Color::rgba(128, 128, 0, 255);
    pub const OLIVEDRAB: Color = Color::rgba(107, 142, 35, 255);
    pub const ORANGE: Color = Color::rgba(255, 165, 0, 255);
    pub const ORANGERED: Color = Color::rgba(255, 69, 0, 255);
    pub const ORCHID: Color = Color::rgba(218, 112, 214, 255);
    pub const PALEGOLDENROD: Color = Color::rgba(238, 232, 170, 255);
    pub const PALEGREEN: Color = Color::rgba(152, 251, 152, 255);
    pub const PALETURQUOISE: Color = Color::rgba(175, 238, 238, 255);
    pub const PALEVIOLETRED: Color = Color::rgba(219, 112, 147, 255);
    pub const PAPAYAWHIP: Color = Color::rgba(255, 239, 213, 255);
    pub const PEACHPUFF: Color = Color::rgba(255, 218, 185, 255);
    pub const PERU: Color = Color::rgba(205, 133, 63, 255);
    pub const PINK: Color = Color::rgba(255, 192, 203, 255);
    pub const PLUM: Color = Color::rgba(221, 160, 221, 255);
    pub const POWDERBLUE: Color = Color::rgba(176, 224, 230, 255);
    pub const PURPLE: Color = Color::rgba(128, 0, 128, 255);
    pub const RED: Color = Color::rgba(255, 0, 0, 255);
    pub const ROSYBROWN: Color = Color::rgba(188, 143, 143, 255);
    pub const ROYALBLUE: Color = Color::rgba(65, 105, 225, 255);
    pub const SADDLEBROWN: Color = Color::rgba(139, 69, 19, 255);
    pub const SALMON: Color = Color::rgba(250, 128, 114, 255);
    pub const SANDYBROWN: Color = Color::rgba(244, 164, 96, 255);
    pub const SEAGREEN: Color = Color::rgba(46, 139, 87, 255);
    pub const SEASHELL: Color = Color::rgba(255, 245, 238, 255);
    pub const SIENNA: Color = Color::rgba(160, 82, 45, 255);
    pub const SILVER: Color = Color::rgba(192, 192, 192, 255);
    pub const SKYBLUE: Color = Color::rgba(135, 206, 235, 255);
    pub const SLATEBLUE: Color = Color::rgba(106, 90, 205, 255);
    pub const SLATEGRAY: Color = Color::rgba(112, 128, 144, 255);
    pub const SNOW: Color = Color::rgba(255, 250, 250, 255);
    pub const SPRINGGREEN: Color = Color::rgba(0, 255, 127, 255);
    pub const STEELBLUE: Color = Color::rgba(70, 130, 180, 255);
    pub const TAN: Color = Color::rgba(210, 180, 140, 255);
    pub const TEAL: Color = Color::rgba(0, 128, 128, 255);
    pub const THISTLE: Color = Color::rgba(216, 191, 216, 255);
    pub const TOMATO: Color = Color::rgba(255, 99, 71, 255);
    pub const TURQUOISE: Color = Color::rgba(64, 224, 208, 255);
    pub const VIOLET: Color = Color::rgba(238, 130, 238, 255);
    pub const WHEAT: Color = Color::rgba(245, 222, 179, 255);
    pub const WHITE: Color = Color::rgba(255, 255, 255, 255);
    pub const WHITESMOKE: Color = Color::rgba(245, 245, 245, 255);
    pub const YELLOW: Color = Color::rgba(255, 255, 0, 255);
    pub const YELLOWGREEN: Color = Color::rgba(154, 205, 50, 255);
}
