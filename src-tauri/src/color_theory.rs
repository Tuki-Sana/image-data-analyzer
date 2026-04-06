//! 色空間・色差・WCAG コントラスト

const XN: f64 = 95.047;
const YN: f64 = 100.0;
const ZN: f64 = 108.883;

const LAB_EPS: f64 = 216.0 / 24389.0;
const LAB_KAPPA: f64 = 24389.0 / 27.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lab {
    pub l: f64,
    pub a: f64,
    pub b: f64,
}

#[inline]
pub fn srgb_u8_channel_to_linear(c: u8) -> f64 {
    let c = c as f64 / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

pub fn srgb_to_xyz(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let r = srgb_u8_channel_to_linear(r);
    let g = srgb_u8_channel_to_linear(g);
    let b = srgb_u8_channel_to_linear(b);
    let x = r * 0.4124564 + g * 0.3575761 + b * 0.1804375;
    let y = r * 0.2126729 + g * 0.7151522 + b * 0.0721750;
    let z = r * 0.0193339 + g * 0.1191920 + b * 0.9503041;
    (x * 100.0, y * 100.0, z * 100.0)
}

#[inline]
fn lab_f(t: f64) -> f64 {
    if t > LAB_EPS {
        t.cbrt()
    } else {
        (LAB_KAPPA * t + 16.0) / 116.0
    }
}

pub fn lab_from_srgb(r: u8, g: u8, b: u8) -> Lab {
    let (x, y, z) = srgb_to_xyz(r, g, b);
    let fx = lab_f(x / XN);
    let fy = lab_f(y / YN);
    let fz = lab_f(z / ZN);
    Lab {
        l: 116.0 * fy - 16.0,
        a: 500.0 * (fx - fy),
        b: 200.0 * (fy - fz),
    }
}

/// Lab → sRGB の逆変換。クレート内では未使用だが API として公開し、ユニットテストで検証する。
#[allow(dead_code)]
mod lab_inverse {
    use super::{Lab, LAB_EPS, LAB_KAPPA, XN, YN, ZN};

    #[inline]
    fn lab_f_inv(t: f64) -> f64 {
        let t3 = t * t * t;
        if t3 > LAB_EPS {
            t3
        } else {
            (116.0 * t - 16.0) / LAB_KAPPA
        }
    }

    fn xyz_from_lab(lab: &Lab) -> (f64, f64, f64) {
        let fy = (lab.l + 16.0) / 116.0;
        let fx = lab.a / 500.0 + fy;
        let fz = fy - lab.b / 200.0;
        (
            lab_f_inv(fx) * XN,
            lab_f_inv(fy) * YN,
            lab_f_inv(fz) * ZN,
        )
    }

    fn linear_srgb_from_xyz100(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
        let xn = x / 100.0;
        let yn = y / 100.0;
        let zn = z / 100.0;
        let rl = 3.2404542 * xn - 1.5371385 * yn - 0.4985314 * zn;
        let gl = -0.9692660 * xn + 1.8760108 * yn + 0.0415560 * zn;
        let bl = 0.0556434 * xn - 0.2040259 * yn + 1.0572252 * zn;
        (rl, gl, bl)
    }

    #[inline]
    fn linear_channel_to_srgb_u8(c: f64) -> u8 {
        let c = c.clamp(0.0, 1.0);
        let companded = if c <= 0.0031308 {
            12.92 * c
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        };
        (companded * 255.0).round().clamp(0.0, 255.0) as u8
    }

    pub fn srgb_u8_from_lab(lab: &Lab) -> (u8, u8, u8) {
        let (x, y, z) = xyz_from_lab(lab);
        let (rl, gl, bl) = linear_srgb_from_xyz100(x, y, z);
        (
            linear_channel_to_srgb_u8(rl),
            linear_channel_to_srgb_u8(gl),
            linear_channel_to_srgb_u8(bl),
        )
    }
}

#[allow(unused_imports)] // クレート外向け API。ユニットテストは `super::srgb_u8_from_lab` 経由。
pub use lab_inverse::srgb_u8_from_lab;

pub fn delta_e_76(a: Lab, b: Lab) -> f64 {
    let dl = a.l - b.l;
    let da = a.a - b.a;
    let db = a.b - b.b;
    (dl * dl + da * da + db * db).sqrt()
}

pub fn relative_luminance(r: u8, g: u8, b: u8) -> f64 {
    let r = srgb_u8_channel_to_linear(r);
    let g = srgb_u8_channel_to_linear(g);
    let b = srgb_u8_channel_to_linear(b);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

pub fn contrast_ratio(lum1: f64, lum2: f64) -> f64 {
    let l = lum1.max(lum2);
    let s = lum1.min(lum2);
    (l + 0.05) / (s + 0.05)
}

pub fn wcag_contrast_rgb(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8) -> f64 {
    contrast_ratio(
        relative_luminance(r1, g1, b1),
        relative_luminance(r2, g2, b2),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wcag_black_white() {
        let r = wcag_contrast_rgb(0, 0, 0, 255, 255, 255);
        assert!((r - 21.0).abs() < 0.05);
    }

    #[test]
    fn lab_srgb_roundtrip_gray() {
        let (r, g, b) = (128u8, 128u8, 128u8);
        let lab = lab_from_srgb(r, g, b);
        let (r2, g2, b2) = srgb_u8_from_lab(&lab);
        assert!((r2 as i16 - r as i16).abs() <= 2);
        assert!((g2 as i16 - g as i16).abs() <= 2);
        assert!((b2 as i16 - b as i16).abs() <= 2);
    }
}
