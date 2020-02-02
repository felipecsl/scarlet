//! This file provides constants that are used for matrix multiplication and color space conversion,
//! along with a function for computing inverses. The reason for this method of doing things instead
//! of simple multiplications and additions is because the inverses of these transformations become
//! slightly off, allowing for errors to slowly creep in even when doing things that should not
//! change the result at all, e.g., converting to an illuminant and back again. Thus, this method
//! allows for saner checking of constant values and guaranteed precision in inversion.

// This is the minimum acceptable difference in CIEDE2000 distance between two colors to consider
// them equal for the purposes of Scarlet's tests. It doesn't belong outside a tests module, but I'm
// putting it here just so that any test across Scarlet can use it to ensure uniform precision.
#[allow(dead_code)] // this is required because it isn't used outside tests: that's OK though
pub(crate) const TEST_PRECISION: f64 = 1e-12;

use rulinalg::matrix::decomposition::PartialPivLu;
use rulinalg::matrix::Matrix;

lazy_static! {
    pub(crate) static ref ADOBE_RGB_TRANSFORM: Matrix<f64> = {
        matrix![02.04159, -0.56501, -0.34473;
                -0.96924, 01.87957, 00.04156;
                00.01344, -0.11836, 01.01517]
    };
    pub(crate) static ref ADOBE_RGB_TRANSFORM_LU: PartialPivLu<f64> =
        { PartialPivLu::decompose(ADOBE_RGB_TRANSFORM.clone()).expect("Matrix is invertible.") };
    pub(crate) static ref BRADFORD_TRANSFORM: Matrix<f64> = {
        matrix![00.8951, 00.2664, -0.1614;
                -0.7502, 01.7135, 00.0367;
                00.0389, -0.0685, 01.0296]
    };
    pub(crate) static ref BRADFORD_TRANSFORM_LU: PartialPivLu<f64> =
        { PartialPivLu::decompose(BRADFORD_TRANSFORM.clone()).expect("Matrix is invertible.") };
    pub(crate) static ref ROMM_RGB_TRANSFORM: Matrix<f64> = {
        matrix![0.7976749, 0.1351917, 0.0313534;
                0.2880402, 0.7118741, 0.0000857;
                0.0000000, 0.0000000, 0.8252100]
    };
    pub(crate) static ref ROMM_RGB_TRANSFORM_LU: PartialPivLu<f64> =
        { PartialPivLu::decompose(ROMM_RGB_TRANSFORM.clone()).expect("Matrix is invertible.") };
    pub(crate) static ref STANDARD_RGB_TRANSFORM: Matrix<f64> = {
        matrix![03.2406, -1.5372, -0.4986;
                -0.9689, 01.8758, 00.0415;
                00.0557, -0.2040, 01.0570]
    };
    pub(crate) static ref STANDARD_RGB_TRANSFORM_LU: PartialPivLu<f64> =
        { PartialPivLu::decompose(STANDARD_RGB_TRANSFORM.clone()).expect("Matrix is invertible.") };
}

// These next two constants define the X11 color names and hex codes.

// This is the color names
// I used a Python script to process it from this site:
// https://github.com/bahamas10/css-color-names/blob/master/css-color-names.json let
pub(crate) const X11_NAMES: [&str; 148] = [
    "aliceblue",
    "antiquewhite",
    "aqua",
    "aquamarine",
    "azure",
    "beige",
    "bisque",
    "black",
    "blanchedalmond",
    "blue",
    "blueviolet",
    "brown",
    "burlywood",
    "cadetblue",
    "chartreuse",
    "chocolate",
    "coral",
    "cornflowerblue",
    "cornsilk",
    "crimson",
    "cyan",
    "darkblue",
    "darkcyan",
    "darkgoldenrod",
    "darkgray",
    "darkgreen",
    "darkgrey",
    "darkkhaki",
    "darkmagenta",
    "darkolivegreen",
    "darkorange",
    "darkorchid",
    "darkred",
    "darksalmon",
    "darkseagreen",
    "darkslateblue",
    "darkslategray",
    "darkslategrey",
    "darkturquoise",
    "darkviolet",
    "deeppink",
    "deepskyblue",
    "dimgray",
    "dimgrey",
    "dodgerblue",
    "firebrick",
    "floralwhite",
    "forestgreen",
    "fuchsia",
    "gainsboro",
    "ghostwhite",
    "gold",
    "goldenrod",
    "gray",
    "green",
    "greenyellow",
    "grey",
    "honeydew",
    "hotpink",
    "indianred",
    "indigo",
    "ivory",
    "khaki",
    "lavender",
    "lavenderblush",
    "lawngreen",
    "lemonchiffon",
    "lightblue",
    "lightcoral",
    "lightcyan",
    "lightgoldenrodyellow",
    "lightgray",
    "lightgreen",
    "lightgrey",
    "lightpink",
    "lightsalmon",
    "lightseagreen",
    "lightskyblue",
    "lightslategray",
    "lightslategrey",
    "lightsteelblue",
    "lightyellow",
    "lime",
    "limegreen",
    "linen",
    "magenta",
    "maroon",
    "mediumaquamarine",
    "mediumblue",
    "mediumorchid",
    "mediumpurple",
    "mediumseagreen",
    "mediumslateblue",
    "mediumspringgreen",
    "mediumturquoise",
    "mediumvioletred",
    "midnightblue",
    "mintcream",
    "mistyrose",
    "moccasin",
    "navajowhite",
    "navy",
    "oldlace",
    "olive",
    "olivedrab",
    "orange",
    "orangered",
    "orchid",
    "palegoldenrod",
    "palegreen",
    "paleturquoise",
    "palevioletred",
    "papayawhip",
    "peachpuff",
    "peru",
    "pink",
    "plum",
    "powderblue",
    "purple",
    "rebeccapurple",
    "red",
    "rosybrown",
    "royalblue",
    "saddlebrown",
    "salmon",
    "sandybrown",
    "seagreen",
    "seashell",
    "sienna",
    "silver",
    "skyblue",
    "slateblue",
    "slategray",
    "slategrey",
    "snow",
    "springgreen",
    "steelblue",
    "tan",
    "teal",
    "thistle",
    "tomato",
    "turquoise",
    "violet",
    "wheat",
    "white",
    "whitesmoke",
    "yellow",
    "yellowgreen",
];

pub(crate) const X11_COLOR_CODES: [&str; 148] = [
    "#f0f8ff", "#faebd7", "#00ffff", "#7fffd4", "#f0ffff", "#f5f5dc", "#ffe4c4", "#000000",
    "#ffebcd", "#0000ff", "#8a2be2", "#a52a2a", "#deb887", "#5f9ea0", "#7fff00", "#d2691e",
    "#ff7f50", "#6495ed", "#fff8dc", "#dc143c", "#00ffff", "#00008b", "#008b8b", "#b8860b",
    "#a9a9a9", "#006400", "#a9a9a9", "#bdb76b", "#8b008b", "#556b2f", "#ff8c00", "#9932cc",
    "#8b0000", "#e9967a", "#8fbc8f", "#483d8b", "#2f4f4f", "#2f4f4f", "#00ced1", "#9400d3",
    "#ff1493", "#00bfff", "#696969", "#696969", "#1e90ff", "#b22222", "#fffaf0", "#228b22",
    "#ff00ff", "#dcdcdc", "#f8f8ff", "#ffd700", "#daa520", "#808080", "#008000", "#adff2f",
    "#808080", "#f0fff0", "#ff69b4", "#cd5c5c", "#4b0082", "#fffff0", "#f0e68c", "#e6e6fa",
    "#fff0f5", "#7cfc00", "#fffacd", "#add8e6", "#f08080", "#e0ffff", "#fafad2", "#d3d3d3",
    "#90ee90", "#d3d3d3", "#ffb6c1", "#ffa07a", "#20b2aa", "#87cefa", "#778899", "#778899",
    "#b0c4de", "#ffffe0", "#00ff00", "#32cd32", "#faf0e6", "#ff00ff", "#800000", "#66cdaa",
    "#0000cd", "#ba55d3", "#9370db", "#3cb371", "#7b68ee", "#00fa9a", "#48d1cc", "#c71585",
    "#191970", "#f5fffa", "#ffe4e1", "#ffe4b5", "#ffdead", "#000080", "#fdf5e6", "#808000",
    "#6b8e23", "#ffa500", "#ff4500", "#da70d6", "#eee8aa", "#98fb98", "#afeeee", "#db7093",
    "#ffefd5", "#ffdab9", "#cd853f", "#ffc0cb", "#dda0dd", "#b0e0e6", "#800080", "#663399",
    "#ff0000", "#bc8f8f", "#4169e1", "#8b4513", "#fa8072", "#f4a460", "#2e8b57", "#fff5ee",
    "#a0522d", "#c0c0c0", "#87ceeb", "#6a5acd", "#708090", "#708090", "#fffafa", "#00ff7f",
    "#4682b4", "#d2b48c", "#008080", "#d8bfd8", "#ff6347", "#40e0d0", "#ee82ee", "#f5deb3",
    "#ffffff", "#f5f5f5", "#ffff00", "#9acd32",
];
