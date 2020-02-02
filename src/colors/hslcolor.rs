//! This file implements what I refer to as HSL but which would precisely be called sHSL: a simple
//! transformation of sRGB that creates a cylindrical space. HSL has the same problems with
//! perceptual uniformity and general unsuitability for exact psychophysically-accurate
//! representation as color as sRGB does, but it does have the advantage of being easy to display on
//! a monitor and having some conception of common color attributes. HSL and HSV are very similar
//! but have an important difference: *value* in HSV runs from black to fully saturated colors,
//! whereas *lightness* or *luminosity* in HSL runs from black to fully saturated in the middle to
//! white at the end. This makes the saturation component of HSL extremely inaccurate, because light
//! colors can have a very high saturation even if they are extremely close to white. This space is
//! mathematically cylindrical, but when you account for the actual differentiation of colors
//! (saturation's actual importance varies with lightness) it forms a "bi-hexcone" model, where the
//! hue component is actually a hexagon but simply stretched into a circle, and the area of a
//! horizontal cross-section varies with lightness.  A special note: some implementations of HSV and
//! HSL are circular in nature, using polar coordinates explicitly. This implementation is instead
//! hexagonal: first values are put on a hexagon, and then that hexagon is "squeezed" into a
//! circle. This can cause small variations between Scarlet and other applications.
//! Another small implementation note is that converting gray into HSL or HSV will give a hue of 0
//! degrees, although any hue could be used in its place.

use std::f64;
use std::f64::EPSILON;
use std::str::FromStr;

use bound::Bound;
use color::{Color, RGBColor, XYZColor};
use coord::Coord;
use csscolor::{parse_hsl_hsv_tuple, CSSParseError};
use illuminants::Illuminant;

/// A color in the HSL color space, a direct transformation of the sRGB space. sHSL is used to
/// distinguish this space from a similar transformation of a different RGB space, which can cause
/// some confusion as other implementations of HSL (such as on the web) omit this distinction.
/// # Example
/// Shifting from red to yellow creates two colors of clearly different brightnesses. This is because
/// HSL doesn't account for the perceptual difference in brightness of light and dark colors.
///
/// ```
/// # use scarlet::prelude::*;
/// # use scarlet::colors::HSLColor;
/// let red = HSLColor{h: 20., s: 0.5, l: 0.5};
/// let yellow = HSLColor{h: 60., s: 0.5, l: 0.5};
/// println!("{} {}", red.convert::<RGBColor>().to_string(), yellow.convert::<RGBColor>().to_string());
/// // prints #BF6A40 #BFBF40
/// // note how the second one is strictly more light
/// ```
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct HSLColor {
    /// The hue component. Ranges from 0 to 360, as the angle in a cylindrical space. Exactly the same
    /// as the hue component of HSV.
    pub h: f64,
    /// The saturation component. Ranges between 0 and 1. Note that this is much less accurate to
    /// human perception than the chroma or saturation found in other, higher-fidelity color spaces.
    pub s: f64,
    /// The lightness component. Ranges from 0 to 1. Defined in HSL as the average of the largest and
    /// smallest color components in RGB, which sacrifices accuracy for convenience.
    pub l: f64,
}

impl Color for HSLColor {
    /// Converts from XYZ to HSL through RGB: thus, there is a limited precision because RGB colors
    /// are limited to integer values of R, G, and B.
    fn from_xyz(xyz: XYZColor) -> HSLColor {
        // first get RGB color
        let rgb = RGBColor::from_xyz(xyz);

        // this is sorta interesting: a hexagonal projection instead of the circular projection used
        // in CIEHCL. It turns out that, if you tilt the RGB cube and project it into a hexagon, the
        // equivalent of radius is simply the largest component minus the smallest component: adding
        // a constant to every component simply travels up and down vertically and doesn't change the
        // projection.
        // I call this chroma, but it's a very very rough estimate of the actual color attribute.
        // More info: https://en.wikipedia.org/wiki/HSL_and_HSV#Formal_derivation
        let components = [rgb.r, rgb.g, rgb.b];
        let max_c = components.iter().cloned().fold(-1.0, f64::max);
        let min_c = components.iter().cloned().fold(2.0, f64::min);
        let chroma = max_c - min_c;

        // hue is crazy in a hexagon! no more trig functions for us!
        // it's technically the proportion of the length of the hexagon through the point, but it's
        // treated as degrees
        let mut hue = if chroma == 0.0 {
            // could be anything, undefined according to Wikipedia, in Scarlet just 0 for gray
            0.0
        } else if (max_c - rgb.r).abs() < EPSILON {
            // in red sector: find which part by comparing green and blue and scaling
            // adding green moves up on the hexagon, adding blue moves down: hence, linearity
            // the modulo makes sure it's in the range 0-360
            (((rgb.g - rgb.b) / chroma) % 6.0) * 60.0
        } else if (max_c - rgb.g).abs() < EPSILON {
            // similar to above, but you add an offset
            (((rgb.b - rgb.r) / chroma) % 6.0) * 60.0 + 120.0
        } else {
            // same as above, different offset
            (((rgb.r - rgb.g) / chroma) % 6.0) * 60.0 + 240.0
        };
        // if hue still not in 0-360, add until it does: this can sometimes happen
        while hue < 0. {
            hue += 360.;
        }
        while hue >= 360. {
            hue -= 360.;
        }

        // saturation, scientifically speaking, is chroma adjusted for lightness. For HSL, it's
        // defined relative to the maximum chroma, which varies depending on the place on the
        // cone. Thus, I'll compute lightness first.

        // now we choose lightness as the average of the largest and smallest components. This
        // essentially translates to a double hex cone, quite the interesting structure!
        let lightness = (max_c + min_c) / 2.0;
        // now back to saturation
        let saturation = if (lightness - 1.0).abs() < EPSILON || lightness == 0.0 {
            // this would be a divide by 0 otherwise, just set it to 0 because it doesn't matter
            0.0
        } else {
            chroma / (1.0 - (2.0 * lightness - 1.0).abs())
        };

        HSLColor {
            h: hue,
            s: saturation,
            l: lightness,
        }
    }
    // Converts back to XYZ through RGB.
    fn to_xyz(&self, illuminant: Illuminant) -> XYZColor {
        // first get back chroma

        let chroma = (1.0 - (2.0 * self.l - 1.0).abs()) * self.s;
        // find the point with 0 lightness that matches ours in the other two components

        // intermediate value is the second-largest RGB value, where C is the largest because the
        // smallest is 0: call this x
        let x = chroma * (1.0 - ((self.h / 60.0) % 2.0 - 1.0).abs());
        // now split based on which line of the hexagon we're on, i.e., which are the two largest
        // components
        let (r1, g1, b1) = if self.h <= 60.0 {
            (chroma, x, 0.0)
        } else if self.h <= 120.0 {
            (x, chroma, 0.0)
        } else if self.h <= 180.0 {
            (0.0, chroma, x)
        } else if self.h <= 240.0 {
            (0.0, x, chroma)
        } else if self.h <= 300.0 {
            (x, 0.0, chroma)
        } else {
            (chroma, 0.0, x)
        };
        // now we add the right value to each component to get the correct lightness and scale back
        // to 0-255
        let offset = self.l - chroma / 2.0;
        let r = r1 + offset;
        let g = g1 + offset;
        let b = b1 + offset;
        RGBColor { r, g, b }.to_xyz(illuminant)
    }
}

impl From<Coord> for HSLColor {
    fn from(c: Coord) -> HSLColor {
        HSLColor {
            h: c.x,
            s: c.y,
            l: c.z,
        }
    }
}

impl Into<Coord> for HSLColor {
    fn into(self) -> Coord {
        Coord {
            x: self.h,
            y: self.s,
            z: self.l,
        }
    }
}

impl Bound for HSLColor {
    fn bounds() -> [(f64, f64); 3] {
        [(0., 360.), (0., 1.), (0., 1.)]
    }
}

impl FromStr for HSLColor {
    type Err = CSSParseError;

    fn from_str(s: &str) -> Result<HSLColor, CSSParseError> {
        if !s.starts_with("hsl(") {
            return Err(CSSParseError::InvalidColorSyntax);
        }
        let tup: String = s.chars().skip(3).collect::<String>();
        match parse_hsl_hsv_tuple(&tup) {
            Ok(res) => Ok(HSLColor {
                h: res.0,
                s: res.1,
                l: res.2,
            }),
            Err(_e) => Err(_e),
        }
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use consts::TEST_PRECISION;

    #[test]
    fn test_hsl_rgb_conversion() {
        let red_rgb = RGBColor {
            r: 1.,
            g: 0.,
            b: 0.,
        };
        let red_hsl: HSLColor = red_rgb.convert();
        assert!(red_hsl.h.abs() <= 0.0001);
        assert!((red_hsl.s - 1.0) <= 0.0001);
        assert!((red_hsl.l - 0.5) <= 0.0001);
        assert!(red_hsl.distance(&red_rgb) < TEST_PRECISION);
        let lavender_hsl = HSLColor {
            h: 245.0,
            s: 0.5,
            l: 0.6,
        };
        let lavender_rgb: RGBColor = lavender_hsl.convert();
        assert_eq!(lavender_rgb.to_string(), "#6E66CC");
    }

    #[test]
    fn test_hsl_string_parsing() {
        let red_hsl: HSLColor = "hsl(0, 120%, 50%)".parse().unwrap();
        assert!(red_hsl.h.abs() <= 0.0001);
        assert!((red_hsl.s - 1.0) <= 0.0001);
        assert!((red_hsl.l - 0.5) <= 0.0001);
        let lavender_hsl: HSLColor = "hsl(-475, 50%, 60%)".parse().unwrap();
        let lavender_rgb: RGBColor = lavender_hsl.convert();
        assert_eq!(lavender_rgb.to_string(), "#6E66CC");
        // test error
        assert!("hsl(254%, 0, 0)".parse::<HSLColor>().is_err());
    }
}
