//! Module for loading and handling images

use std::sync::atomic::{AtomicUsize, Ordering};
use webrender::api::{
    ImageFormat as RawImageFormat,
    ImageData, ImageDescriptor, ImageKey
};
#[cfg(feature = "image_loading")]
use image::{
    self, ImageResult, ImageFormat,
    ImageError, DynamicImage, GenericImageView,
};

pub type CssImageId = String;

static IMAGE_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
static RAW_IMAGE_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImageId { id: usize }

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawImageId { id: usize }

impl ImageId {
    pub fn new() -> Self {
        ImageId { id: IMAGE_ID_COUNTER.fetch_add(1, Ordering::SeqCst) }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawImageId { id: usize }

impl RawImageId {
    pub fn new() -> Self {
        ImageId { id: RAW_IMAGE_ID_COUNTER.fetch_add(1, Ordering::SeqCst) }
    }
}

pub struct RawImage {
    pub dimensions: (u32, u32),
    pub pixels: Vec<u8>,
    pub data_format: RawImageFormat,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ImageInfo {
    pub(crate) key: ImageKey,
    pub(crate) descriptor: ImageDescriptor,
}

#[derive(Debug, Clone)]
pub(crate) enum ImageState {
    // resource is available for the renderer
    Uploaded(ImageInfo),
    // image is loaded & decoded, but not yet available
    ReadyForUpload((ImageData, ImageDescriptor)),
    // Image is about to get deleted in the next frame
    AboutToBeDeleted((Option<ImageKey>, ImageDescriptor)),
}

impl ImageState {
    /// Returns the original dimensions of the image
    pub fn get_dimensions(&self) -> (f32, f32) {
        use self::ImageState::*;
        match self {
            Uploaded(ImageInfo { descriptor, .. }) |
            ReadyForUpload((_, descriptor)) |
            AboutToBeDeleted((_, descriptor)) => (descriptor.size.width as f32, descriptor.size.height as f32)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExternalImageSource {
    /// The image is embedded inside the binary file
    Embedded(&'static [u8]),
    File(PathBuf),
}

impl From<ExternalImageSource> for ImageSource {
    fn from(source: ExternalImageSource) -> ImageSource {
        use ExternalImageSource::*;
        match source {
            Embedded(e) => ImageSource::Embedded(e),
            File(path) => ImageSource::File(path),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImageSource {
    /// The image is embedded inside the binary file
    Embedded(&'static [u8]),
    File(PathBuf),
    /// Image has no source, but was added via the raw bytes
    RawBytes(RawImageId),
}

#[derive(Debug)]
pub enum ImageReloadError {
    Io(IoError, PathBuf),
    InvalidRawImageId(RawImageId),
}

impl Clone for ImageReloadError {
    fn clone(&self) -> Self {
        use self::ImageReloadError::*;
        match self {
            Io(err, path) => Io(IoError::new(err.kind(), "Io Error"), path.clone()),
            InvalidRawImageId(id) => InvalidRawImageId(*id),
        }
    }
}

impl fmt::Display for ImageReloadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ImageReloadError::*;
        match self {
            Io(err, path_buf) => write!(f, "Could not load \"{}\" - IO error: {}", path_buf.as_path().to_string_lossy(), err),
            InvalidRawImageId(id) => write!(f, "Invalid raw image ID: {:?}", id),
        }
    }
}

impl ImageSource {

    /// Creates an image source using a unique image key
    pub fn new_raw() -> Self {
        ImageSource::Raw(RawImageId::new())
    }

    /// Creates an image source from a `&static [u8]`.
    pub fn new_from_static(bytes: &'static [u8]) -> Self {
        ImageSource::Embedded(bytes)
    }

    /// Creates an image source from a file
    pub fn new_from_file<I: Into<PathBuf>>(file_path: I) -> Self {
        ImageSource::File(file_path.into())
    }

    /// Returns the bytes of the font
    pub(crate) fn get_bytes(&self, raw_images: &FastHashMap<RawImageId, RawImage>) -> Result<Vec<u8>, ImageReloadError> {
        use std::fs;
        use self::ImageSource::*;
        match self {
            Embedded(bytes) => Ok(bytes.to_vec()),
            File(file_path) => fs::read(file_path).map_err(|e| ImageReloadError::Io(e, file_path.clone())),
            Raw(id) => raw_images.get(id).ok_or(ImageReloadError::InvalidRawImageId(*id)),
        }
    }
}


// The next three functions are taken from:
// https://github.com/christolliday/limn/blob/master/core/src/resources/image.rs

/// Reshuffles the data in a `DynamicImage` into RGBA8 / A8 form.
#[cfg(feature = "image_loading")]
pub fn prepare_image(image_decoded: DynamicImage)
    -> Result<(ImageData, ImageDescriptor), ImageError>
{
    let image_dims = image_decoded.dimensions();

    // see: https://github.com/servo/webrender/blob/80c614ab660bf6cca52594d0e33a0be262a7ac12/wrench/src/yaml_frame_reader.rs#L401-L427
    let (format, bytes) = match image_decoded {
        image::ImageLuma8(bytes) => {
            let pixels = bytes.into_raw();
            (RawImageFormat::R8, pixels)
        },
        image::ImageLumaA8(bytes) => {
            let mut pixels = Vec::with_capacity(image_dims.0 as usize * image_dims.1 as usize * 4);
            for greyscale_alpha in bytes.chunks(2) {
                let grey = greyscale_alpha[0];
                let alpha = greyscale_alpha[1];
                pixels.extend_from_slice(&[
                    grey,
                    grey,
                    grey,
                    alpha,
                ]);
            }
            // TODO: necessary for greyscale?
            premultiply(pixels.as_mut_slice());
            (RawImageFormat::BGRA8, pixels)
        },
        image::ImageRgba8(mut bytes) => {
            let mut pixels = bytes.into_raw();
            // no extra allocation necessary, but swizzling
            for rgba in pixels.chunks_mut(4) {
                let r = rgba[0];
                let g = rgba[1];
                let b = rgba[2];
                let a = rgba[3];
                rgba[0] = b;
                rgba[1] = r;
                rgba[2] = g;
                rgba[3] = a;
            }
            premultiply(pixels.as_mut_slice());
            (RawImageFormat::BGRA8, pixels)
        },
        image::ImageRgb8(bytes) => {
            let mut pixels = Vec::with_capacity(image_dims.0 as usize * image_dims.1 as usize * 4);
            for rgb in bytes.chunks(3) {
                pixels.extend_from_slice(&[
                    rgb[2], // b
                    rgb[1], // g
                    rgb[0], // r
                    0xff    // a
                ]);
            }
            (RawImageFormat::BGRA8, pixels)
        },
        image::ImageBgr8(bytes) => {
            let mut pixels = Vec::with_capacity(image_dims.0 as usize * image_dims.1 as usize * 4);
            for bgr in bytes.chunks(3) {
                pixels.extend_from_slice(&[
                    bgr[0], // b
                    bgr[1], // g
                    bgr[2], // r
                    0xff    // a
                ]);
            }
            (RawImageFormat::BGRA8, pixels)
        },
        image::ImageBgra8(bytes) => {
            // Already in the correct format
            let mut pixels = bytes.into_raw();
            premultiply(pixels.as_mut_slice());
            (RawImageFormat::BGRA8, pixels)
        },
    };

    let opaque = is_image_opaque(format, &bytes[..]);
    let allow_mipmaps = true;
    let descriptor = ImageDescriptor::new(image_dims.0 as i32, image_dims.1 as i32, format, opaque, allow_mipmaps);
    let data = ImageData::new(bytes);
    Ok((data, descriptor))
}

pub(crate) fn is_image_opaque(format: RawImageFormat, bytes: &[u8]) -> bool {
    match format {
        RawImageFormat::BGRA8 => {
            let mut is_opaque = true;
            for i in 0..(bytes.len() / 4) {
                if bytes[i * 4 + 3] != 255 {
                    is_opaque = false;
                    break;
                }
            }
            is_opaque
        }
        RawImageFormat::R8 => true,
        _ => unreachable!(),
    }
}

// From webrender/wrench
// These are slow. Gecko's gfx/2d/Swizzle.cpp has better versions
pub(crate) fn premultiply(data: &mut [u8]) {
    for pixel in data.chunks_mut(4) {
        let a = u32::from(pixel[3]);
        pixel[0] = (((pixel[0] as u32 * a) + 128) / 255) as u8;
        pixel[1] = (((pixel[1] as u32 * a) + 128) / 255) as u8;
        pixel[2] = (((pixel[2] as u32 * a) + 128) / 255) as u8;
    }
}

#[test]
fn test_premultiply() {
    let mut color = [255, 0, 0, 127];
    premultiply(&mut color);
    assert_eq!(color, [127, 0, 0, 127]);
}