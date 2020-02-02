//! This file uses the CSS numeric parsing in `cssnumeric.rs` to parse CSS functional color notation
//! according to the W3 specification. The only difference is that arithmetic is not supported to
//! specify colors. Its end goal is the implementation of FromStr for RGB, HSL, and HSV colors,
//! although the specific `impl` blocks are in their respective source files. You can see the full
//! spec here: [https://www.w3.org/TR/css-color-3/](https://www.w3.org/TR/css-color-3/). One quick caveat:
//! as is relatively standard, percents are only integral: "45.5%" will be treated as invalid.

pub(crate) use cssnumeric::CSSParseError;
use cssnumeric::{parse_css_number, CSSNumeric};

/// Given a string, attempts to parse as a CSS numeric. If successful, interprets the number given as
/// a component of an RGB color, clamping accordingly. Returns the appropriate `u8`: e.g., "102%" maps
/// to 255, and "34.5" maps to 35. Gives an error on invalid input.
fn parse_rgb_num(num: &str) -> Result<u8, CSSParseError> {
    let parsed_num = parse_css_number(num)?;
    match parsed_num {
        // integer: clamp to 0-255 and use directly
        CSSNumeric::Integer(val) => {
            if val >= 255 {
                Ok(255u8)
            } else if val <= 0 {
                Ok(0u8)
            } else {
                Ok(val as u8)
            }
        }
        CSSNumeric::Float(val) => {
            // interpret between 0 and 1, clamping
            let clamped = if val <= 0. {
                0.
            } else if val >= 1. {
                1.
            } else {
                val
            };
            // return that value as u8
            // the minus bit is to adjust rounding so that, e.g., 50% maps to 127 not 128
            Ok((clamped * 255. - 0.000001).round() as u8)
        }
        CSSNumeric::Percentage(val) => {
            // clamp between 0 and 100
            let clamped = if val <= 0 {
                0
            } else if val >= 100 {
                100
            } else {
                val
            };
            // divide by 100 and then multiply by 255, or equivalently multiply by 2.55
            Ok((clamped as f64 * 2.55).round() as u8)
        }
    }
}

/// Parses a string of the form "rgb(r, g, b)", where r, g, and b are numbers, returning a tuple of
/// u8s for the three components. Gives a CSSParseError on invalid input.
pub(crate) fn parse_rgb_str(num: &str) -> Result<(u8, u8, u8), CSSParseError> {
    // must have at least 10 characters
    // has to start with "rgb(" or not a valid color
    if !num.starts_with("rgb(") || num.len() < 10 {
        return Err(CSSParseError::InvalidColorSyntax);
    }
    // remove first four chars, put in Vec
    let mut chars: Vec<char> = num.chars().skip(4).collect();
    // check for and remove parenthesis
    if chars.iter().last().unwrap() != &')' {
        return Err(CSSParseError::InvalidColorSyntax);
    }
    chars.pop();

    // test for disallowed characters
    if chars.iter().any(|&c| !"0123456789+-,. %".contains(c)) {
        println!("hi");
        return Err(CSSParseError::InvalidColorSyntax);
    }
    // this now requires a very specific format: three commas, a parenthesis at the end, and spaces
    // in between
    // check for commas (the right number of them) and split into numbers, remove whitespace,
    // parse, and recombine
    let split_iter = (&chars).split(|c| c == &',');
    // now remove surrounding whitespace and pass to number parsing, propagating errors
    let mut nums: Vec<u8> = vec![];
    for split in split_iter {
        nums.push(parse_rgb_num(&(split.iter().collect::<String>().trim()))?);
    }
    if nums.len() != 3 {
        return Err(CSSParseError::InvalidColorSyntax);
    }
    Ok((nums[0], nums[1], nums[2]))
}

/// Parses an HSL or HSV tuple, given after "hsl" or "hsv" in normal CSS, such as "(250, 50%, 50%)"
/// into a tuple (f64, f64, f64) such that the first float lies within the range 0-360 and the other
/// two lie within the range 0-1. Gives a CSSParseError if invalid.
pub(crate) fn parse_hsl_hsv_tuple(tup: &str) -> Result<(f64, f64, f64), CSSParseError> {
    // must have '(' at start and ')' at end: remove them, and store in chars vec
    if !tup.starts_with('(') || !tup.ends_with(')') {
        return Err(CSSParseError::InvalidColorSyntax);
    }
    let mut chars: Vec<char> = tup.chars().skip(1).collect();
    chars.pop();

    // split with commas: must be 3 distinct things
    let split_iter = (&chars).split(|c| c == &',');
    let mut numerics: Vec<CSSNumeric> = vec![];
    for split in split_iter {
        numerics.push(parse_css_number(
            &(split.iter().collect::<String>().trim()),
        )?);
    }
    if numerics.len() != 3 {
        return Err(CSSParseError::InvalidColorSyntax);
    }
    // hue is special: require float or integer, normalize to 0-360
    let hue: f64 = match numerics[0] {
        CSSNumeric::Integer(val) => {
            let mut clamped = val;
            while clamped < 0 {
                clamped += 360;
            }
            while clamped >= 360 {
                clamped -= 360;
            }
            clamped as f64
        }
        CSSNumeric::Float(val) => {
            let mut clamped = val;
            while clamped < 0. {
                clamped += 360.;
            }
            while clamped >= 360. {
                clamped -= 360.;
            }
            clamped
        }
        _ => return Err(CSSParseError::InvalidColorSyntax),
    };
    // saturation and lightness/value all work the same way: clamp between 0 and 1 and expect a
    // percentage
    let sat: f64 = match numerics[1] {
        CSSNumeric::Percentage(val) => {
            if val < 0 {
                0.
            } else if val > 100 {
                1.
            } else {
                (val as f64) / 100.
            }
        }
        _ => return Err(CSSParseError::InvalidColorSyntax),
    };
    let l_or_v: f64 = match numerics[2] {
        CSSNumeric::Percentage(val) => {
            if val < 0 {
                0.
            } else if val > 100 {
                1.
            } else {
                (val as f64) / 100.
            }
        }
        _ => return Err(CSSParseError::InvalidColorSyntax),
    };
    // now return
    Ok((hue, sat, l_or_v))
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_rgb_num_parsing() {
        // test integers
        assert_eq!(104u8, parse_rgb_num("104").unwrap());
        assert_eq!(255u8, parse_rgb_num("234923").unwrap());
        // test floats
        assert_eq!(123u8, parse_rgb_num(".48235").unwrap());
        assert_eq!(255u8, parse_rgb_num("1.04").unwrap());
        // test percents
        assert_eq!(122u8, parse_rgb_num("48%").unwrap());
        assert_eq!(255u8, parse_rgb_num("115%").unwrap());
        // test errors
        assert_eq!(
            Err(CSSParseError::InvalidNumericCharacters),
            parse_rgb_num("abc")
        );
        assert_eq!(
            Err(CSSParseError::InvalidNumericSyntax),
            parse_rgb_num("123%%")
        );
    }

    #[test]
    fn test_rgb_str_parsing() {
        // test integers and percents all at once
        let rgb = parse_rgb_str("rgb(125, 20%, 0.5)").unwrap();
        assert_eq!(rgb, (125, 51, 127));
        // test clamping in every direction
        let rgb = parse_rgb_str("rgb(-125, -20%, 10.5)").unwrap();
        assert_eq!(rgb, (0, 0, 255));
        // test error on bad syntax
        assert_eq!(
            Err(CSSParseError::InvalidColorSyntax),
            parse_rgb_str("rgB(123, 33, 2)")
        );
        assert_eq!(
            Err(CSSParseError::InvalidColorSyntax),
            parse_rgb_str("rgb(123, 123, 41, 22)")
        );
        assert_eq!(
            Err(CSSParseError::InvalidColorSyntax),
            parse_rgb_str("rgB(())")
        );
    }

    #[test]
    fn test_hslv_str_parsing() {
        // test normal
        let hsl = parse_hsl_hsv_tuple("(123, 40%, 40%)").unwrap();
        assert_eq!(hsl.0.round() as u8, 123u8);
        assert_eq!((hsl.1 * 100.).round() as u8, 40u8);
        assert_eq!((hsl.2 * 100.).round() as u8, 40u8);
        // test hue angle stuff
        let hsl = parse_hsl_hsv_tuple("(-597, 40%, 40%)").unwrap();
        assert_eq!(hsl.0.round() as u8, 123u8);
        assert_eq!((hsl.1 * 100.).round() as u8, 40u8);
        assert_eq!((hsl.2 * 100.).round() as u8, 40u8);
        let hsl = parse_hsl_hsv_tuple("(1203, 40%, 40%)").unwrap();
        assert_eq!(hsl.0.round() as u8, 123u8);
        assert_eq!((hsl.1 * 100.).round() as u8, 40u8);
        assert_eq!((hsl.2 * 100.).round() as u8, 40u8);
        // test percentage clamping
        let hsl = parse_hsl_hsv_tuple("(123, 140%, -40%)").unwrap();
        assert_eq!(hsl.0.round() as u8, 123u8);
        assert_eq!((hsl.1 * 100.).round() as u8, 100u8);
        assert_eq!((hsl.2 * 100.).round() as u8, 0u8);
        // test error
        assert_eq!(
            parse_hsl_hsv_tuple("(14%, 140%, 12%)"),
            Err(CSSParseError::InvalidColorSyntax)
        );
    }
}
