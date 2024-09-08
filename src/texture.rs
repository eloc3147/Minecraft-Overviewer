use image::imageops::FilterType;
use image::{imageops, GenericImageView, ImageBuffer, RgbaImage};
use pyo3::types::PyBytes;
use pyo3::{pyfunction, Bound, PyResult, Python};

const fn check_fixed(matrix: &[f64; 6], x: u32, y: u32) -> bool {
    let xf = x as f64;
    let yf = y as f64;
    let mut c1 = xf * matrix[0] + yf * matrix[1] + matrix[2];
    let mut c2 = xf * matrix[3] + yf * matrix[4] + matrix[5];
    if c1 < 0.0 {
        c1 *= -1.0;
    }
    if c2 < 0.0 {
        c2 *= -1.0;
    }

    c1 < 32768.0 && c2 < 32768.0
}

struct AffineTransformConfig {
    pub matrix: [f64; 6],
    pub fixed_matrix: [i32; 6],
    pub width: u32,
    pub height: u32,
    pub fixed: bool,
    pub scale: bool,
}

impl AffineTransformConfig {
    const fn new(matrix: [f64; 6], width: u32, height: u32) -> Self {
        let fixed = check_fixed(&matrix, 0, 0)
            && check_fixed(&matrix, width, height)
            && check_fixed(&matrix, 0, height)
            && check_fixed(&matrix, width, 0);

        let scale = matrix[1] == 0.0 && matrix[3] == 0.0;

        let fixed_matrix = if fixed {
            [
                (matrix[0] * 65536.0 + 0.5) as i32,
                (matrix[1] * 65536.0 + 0.5) as i32,
                ((matrix[2] + matrix[0] * 0.5 + matrix[1] * 0.5) * 65536.0 + 0.5) as i32,
                (matrix[3] * 65536.0 + 0.5) as i32,
                (matrix[4] * 65536.0 + 0.5) as i32,
                ((matrix[5] + matrix[3] * 0.5 + matrix[4] * 0.5) * 65536.0 + 0.5) as i32,
            ]
        } else {
            [0; 6]
        };

        Self {
            matrix,
            fixed_matrix,
            width,
            height,
            fixed,
            scale,
        }
    }
}

/// Shear transform
/// `numpy.array(numpy.matrix(numpy.identity(3)) * numpy.matrix("[1,0,0;-0.5,1,0;0,0,1]"))[:2,:].ravel().tolist()`
const TRANSFORM_SIDE: AffineTransformConfig =
    AffineTransformConfig::new([1.0, 0.0, 0.0, -0.5, 1.0, 0.0], 12, 18);

#[pyfunction]
pub fn transform_image_side<'py>(
    width: u32,
    height: u32,
    data: Vec<u8>,
    py: Python<'py>,
) -> PyResult<Bound<'py, PyBytes>> {
    let image: RgbaImage =
        ImageBuffer::from_raw(width, height, data).expect("Can't load image data");

    let resized = imageops::resize(&image, 12, 12, FilterType::Lanczos3);
    let sheared = affine_transform(&resized, &TRANSFORM_SIDE);

    Ok(PyBytes::new_bound(py, &sheared.as_raw()))
}

fn affine_transform(src: &RgbaImage, config: &AffineTransformConfig) -> RgbaImage {
    if config.scale {
        /* Scaling */
        unimplemented!()
        //return ImagingScaleAffine(imOut, imIn, x0, y0, x1, y1, a, fill);
    }

    if config.fixed {
        return affine_fixed(src, config);
    }

    affine_float(src, config)
}

fn affine_fixed(src: &RgbaImage, config: &AffineTransformConfig) -> RgbaImage {
    let mut dest = RgbaImage::new(config.width, config.height);
    let [m0, m1, mut m2, m3, m4, mut m5] = config.fixed_matrix;

    for row in dest.rows_mut() {
        let mut x = m2;
        let mut y = m5;

        for out in row {
            let yin = y >> 16;
            let srcy = yin as u32;

            if yin >= 0 && srcy < src.height() {
                let xin = x >> 16;
                let srcx = xin as u32;

                if xin >= 0 && srcx < src.width() {
                    *out = *src.get_pixel(srcx, srcy);
                }
            }

            x += m0;
            y += m3;
        }
        m2 += m1;
        m5 += m4;
    }

    dest
}

fn affine_float(src: &RgbaImage, config: &AffineTransformConfig) -> RgbaImage {
    let mut dest = RgbaImage::new(config.width, config.height);
    let [m0, m1, m2, m3, m4, m5] = config.matrix;

    let mut xo = m2 + m1 * 0.5 + m0 * 0.5;
    let mut yo = m5 + m4 * 0.5 + m3 * 0.5;

    for row in dest.rows_mut() {
        let mut x = xo;
        let mut y = yo;
        for out in row {
            let xin = x as u32;
            let yin = y as u32;

            if x >= 0.0 && xin < src.width() && y >= 0.0 && yin < src.height() {
                *out = *src.get_pixel(xin, yin);
            }
            x += m0;
            y += m3;
        }
        xo += m1;
        yo += m4;
    }

    dest
}
