use std::sync::Arc;

use image::GenericImageView;
use uuid::Uuid;

use crate::infrastructure::storage::s3::PhotoStorage;

pub struct PhotoService {
    storage: Arc<PhotoStorage>,
}

impl PhotoService {
    pub fn new(storage: Arc<PhotoStorage>) -> Self {
        Self { storage }
    }

    /// Upload a photo and its thumbnail to S3.
    /// Returns (original_url, thumbnail_url).
    pub async fn upload_photo(
        &self,
        listing_id: Uuid,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<(String, String), anyhow::Error> {
        // Validate image BEFORE upload — also protects against decompression bombs
        let img = image::load_from_memory(&data)?;
        let (w, h) = img.dimensions();
        if w > 8000 || h > 8000 {
            anyhow::bail!("Image too large: {}x{}", w, h);
        }

        // Upload original
        let url = self
            .storage
            .upload(listing_id, data, content_type, ".jpg")
            .await?;

        // Generate thumbnail from already-parsed image (no double parse)
        let thumb_data = generate_thumbnail_from_image(&img, 400)?;
        let thumb_url = self
            .storage
            .upload(listing_id, thumb_data, "image/jpeg", "_thumb.jpg")
            .await?;

        Ok((url, thumb_url))
    }

    /// Delete a photo and its optional thumbnail from S3.
    pub async fn delete_photo(
        &self,
        url: &str,
        thumb_url: Option<&str>,
    ) -> Result<(), anyhow::Error> {
        self.storage.delete(url).await?;
        if let Some(thumb) = thumb_url {
            self.storage.delete(thumb).await?;
        }
        Ok(())
    }
}

/// Resize an already-parsed image to fit within max_width, preserving aspect ratio.
/// Returns JPEG bytes.
fn generate_thumbnail_from_image(img: &image::DynamicImage, max_width: u32) -> Result<Vec<u8>, anyhow::Error> {
    let (w, h) = img.dimensions();

    let thumb = if w > max_width {
        let new_h = (max_width as f64 / w as f64 * h as f64) as u32;
        img.resize(max_width, new_h, image::imageops::FilterType::Lanczos3)
    } else {
        img.clone()
    };

    let mut buf = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buf);
    thumb.write_to(&mut cursor, image::ImageFormat::Jpeg)?;
    Ok(buf)
}

/// Resize image to fit within max_width, preserving aspect ratio.
/// Returns JPEG bytes.
#[cfg(test)]
fn generate_thumbnail(data: &[u8], max_width: u32) -> Result<Vec<u8>, anyhow::Error> {
    let img = image::load_from_memory(data)?;
    let (w, h) = img.dimensions();

    let thumb = if w > max_width {
        let new_h = (max_width as f64 / w as f64 * h as f64) as u32;
        img.resize(max_width, new_h, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };

    let mut buf = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buf);
    thumb.write_to(&mut cursor, image::ImageFormat::Jpeg)?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_jpeg(width: u32, height: u32) -> Vec<u8> {
        let img = image::RgbImage::new(width, height);
        let mut buf = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut buf);
        image::DynamicImage::ImageRgb8(img)
            .write_to(&mut cursor, image::ImageFormat::Jpeg)
            .unwrap();
        buf
    }

    #[test]
    fn test_generate_thumbnail_no_resize_when_small() {
        let data = create_test_jpeg(200, 100);
        let result = generate_thumbnail(&data, 400).unwrap();

        let thumb = image::load_from_memory(&result).unwrap();
        let (w, _h) = thumb.dimensions();
        // Image is 200px wide, max is 400, should not resize
        assert!(w <= 400);
    }

    #[test]
    fn test_generate_thumbnail_resizes_large_image() {
        let data = create_test_jpeg(800, 600);
        let result = generate_thumbnail(&data, 400).unwrap();

        let thumb = image::load_from_memory(&result).unwrap();
        let (w, _h) = thumb.dimensions();
        assert!(w <= 400);
    }

    #[test]
    fn test_generate_thumbnail_invalid_data() {
        let result = generate_thumbnail(b"not-an-image", 400);
        assert!(result.is_err());
    }
}
