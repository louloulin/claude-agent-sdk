//! Example: Comprehensive Multimodal Input Handling
//!
//! This example demonstrates comprehensive multimodal input handling
//! including multiple images, mixed content types, and preprocessing.
//!
//! What it demonstrates:
//! 1. Processing multiple images in a single query
//! 2. Mixed content types (text + images + URLs)
//! 3. Image preprocessing and optimization
//! 4. URL vs base64 encoding comparison
//! 5. Handling large images and batch processing

use anyhow::Result;
use std::time::Instant;

/// Simulated image data structure
#[derive(Debug, Clone)]
struct ImageData {
    name: String,
    format: ImageFormat,
    size_bytes: usize,
    dimensions: (u32, u32),
    data: Vec<u8>,
}

/// Supported image formats
#[derive(Debug, Clone, Copy)]
enum ImageFormat {
    Jpeg,
    Png,
    Gif,
    WebP,
}

impl ImageFormat {
    fn mime_type(&self) -> &'static str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::Gif => "image/gif",
            Self::WebP => "image/webp",
        }
    }

    fn extension(&self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::Gif => "gif",
            Self::WebP => "webp",
        }
    }
}

/// Content block types for multimodal queries
#[derive(Debug, Clone)]
enum ContentBlock {
    Text { content: String },
    ImageBase64 { media_type: String, data: String },
    ImageUrl { url: String },
}

/// Image preprocessing options
#[derive(Debug, Clone)]
struct PreprocessOptions {
    max_width: u32,
    max_height: u32,
    quality: u8, // 1-100 for JPEG
    convert_to: Option<ImageFormat>,
}

impl Default for PreprocessOptions {
    fn default() -> Self {
        Self {
            max_width: 2048,
            max_height: 2048,
            quality: 85,
            convert_to: None,
        }
    }
}

fn main() -> Result<()> {
    println!("=== Comprehensive Multimodal Input Examples ===\n");

    multiple_images_example()?;
    mixed_content_example()?;
    image_preprocessing_example()?;
    url_vs_base64_example()?;
    large_image_handling_example()?;

    Ok(())
}

/// Demonstrates processing multiple images in a single query
fn multiple_images_example() -> Result<()> {
    println!("=== Multiple Images in Single Query ===\n");

    // Simulate multiple images
    let images = vec![
        ImageData {
            name: "screenshot1.png".to_string(),
            format: ImageFormat::Png,
            size_bytes: 150_000,
            dimensions: (1920, 1080),
            data: vec![0; 150_000],
        },
        ImageData {
            name: "diagram.jpg".to_string(),
            format: ImageFormat::Jpeg,
            size_bytes: 85_000,
            dimensions: (1200, 800),
            data: vec![0; 85_000],
        },
        ImageData {
            name: "code_snippet.png".to_string(),
            format: ImageFormat::Png,
            size_bytes: 45_000,
            dimensions: (800, 600),
            data: vec![0; 45_000],
        },
    ];

    println!("Processing {} images:", images.len());

    let mut content_blocks = Vec::new();

    // Add text context first
    content_blocks.push(ContentBlock::Text {
        content: "Please analyze these screenshots and diagrams:".to_string(),
    });

    // Add each image
    for (i, image) in images.iter().enumerate() {
        println!("\n  Image {}: {}", i + 1, image.name);
        println!("    Format: {}", image.format.mime_type());
        println!("    Size: {} bytes ({:.1} KB)", image.size_bytes, image.size_bytes as f64 / 1024.0);
        println!("    Dimensions: {}x{}", image.dimensions.0, image.dimensions.1);

        // In real code, you would base64 encode the image data
        let base64_data = simulate_base64_encode(&image.data);
        content_blocks.push(ContentBlock::ImageBase64 {
            media_type: image.format.mime_type().to_string(),
            data: base64_data,
        });

        // Add label for each image
        content_blocks.push(ContentBlock::Text {
            content: format!("[Image {}: {}]", i + 1, image.name),
        });
    }

    println!("\nTotal content blocks: {}", content_blocks.len());
    println!("  Text blocks: {}", content_blocks.iter().filter(|b| matches!(b, ContentBlock::Text { .. })).count());
    println!("  Image blocks: {}", content_blocks.iter().filter(|b| matches!(b, ContentBlock::ImageBase64 { .. })).count());

    println!("\nBest practices for multiple images:");
    println!("  - Add text labels between images for context");
    println!("  - Keep total image count reasonable (≤5 recommended)");
    println!("  - Optimize image sizes before sending");
    println!("  - Use consistent formats when possible");
    println!();

    Ok(())
}

/// Demonstrates mixed content types (text + images + URLs)
fn mixed_content_example() -> Result<()> {
    println!("=== Mixed Content Types ===\n");

    let content_blocks = vec![
        // Opening text
        ContentBlock::Text {
            content: "I need help understanding this architecture:".to_string(),
        },
        // Local image (base64)
        ContentBlock::ImageBase64 {
            media_type: "image/png".to_string(),
            data: "base64_encoded_diagram_data...".to_string(),
        },
        // More context
        ContentBlock::Text {
            content: "The diagram above shows the current system. Compare with this reference:".to_string(),
        },
        // External image (URL)
        ContentBlock::ImageUrl {
            url: "https://example.com/reference-architecture.png".to_string(),
        },
        // Follow-up question
        ContentBlock::Text {
            content: "What improvements would you suggest?".to_string(),
        },
    ];

    println!("Mixed content query structure:");
    for (i, block) in content_blocks.iter().enumerate() {
        match block {
            ContentBlock::Text { content } => {
                let preview = if content.len() > 50 {
                    &content[..50]
                } else {
                    content
                };
                println!("  {}. [TEXT] \"{}...\"", i + 1, preview);
            }
            ContentBlock::ImageBase64 { media_type, .. } => {
                println!("  {}. [IMAGE_BASE64] {}", i + 1, media_type);
            }
            ContentBlock::ImageUrl { url } => {
                println!("  {}. [IMAGE_URL] {}", i + 1, url);
            }
        }
    }

    println!("\nUse cases for mixed content:");
    println!("  - Comparing local and reference images");
    println!("  - Providing context with visual examples");
    println!("  - Multi-step visual analysis tasks");
    println!("  - Documentation with embedded images");
    println!();

    Ok(())
}

/// Demonstrates image preprocessing and optimization
fn image_preprocessing_example() -> Result<()> {
    println!("=== Image Preprocessing ===\n");

    let original_image = ImageData {
        name: "large_photo.png".to_string(),
        format: ImageFormat::Png,
        size_bytes: 5_200_000, // 5.2 MB
        dimensions: (4000, 3000),
        data: vec![0; 5_200_000],
    };

    println!("Original image:");
    println!("  Name: {}", original_image.name);
    println!("  Format: {}", original_image.format.mime_type());
    println!("  Size: {:.2} MB", original_image.size_bytes as f64 / 1_000_000.0);
    println!("  Dimensions: {}x{}", original_image.dimensions.0, original_image.dimensions.1);
    println!();

    // Apply preprocessing options
    let options = PreprocessOptions {
        max_width: 2048,
        max_height: 2048,
        quality: 85,
        convert_to: Some(ImageFormat::Jpeg),
    };

    println!("Preprocessing options:");
    println!("  Max dimensions: {}x{}", options.max_width, options.max_height);
    println!("  JPEG quality: {}%", options.quality);
    println!("  Convert to: {:?}", options.convert_to.map(|f| f.mime_type()));
    println!();

    // Simulate preprocessing
    let processed = preprocess_image(&original_image, &options)?;

    println!("Processed image:");
    println!("  Format: {}", processed.format.mime_type());
    println!("  Size: {:.2} MB ({:.0}% reduction)",
        processed.size_bytes as f64 / 1_000_000.0,
        (1.0 - processed.size_bytes as f64 / original_image.size_bytes as f64) * 100.0
    );
    println!("  Dimensions: {}x{}", processed.dimensions.0, processed.dimensions.1);
    println!();

    println!("Preprocessing recommendations:");
    println!("  - Resize large images to max 2048x2048");
    println!("  - Convert PNG to JPEG for photos (smaller size)");
    println!("  - Use WebP for best compression (if supported)");
    println!("  - Strip EXIF data for privacy");
    println!("  - Crop to relevant regions when possible");
    println!();

    Ok(())
}

/// Demonstrates URL vs base64 encoding comparison
fn url_vs_base64_example() -> Result<()> {
    println!("=== URL vs Base64 Comparison ===\n");

    let test_image_size = 100_000; // 100KB image

    println!("Comparison for a {}KB image:", test_image_size / 1000);
    println!();

    // Base64 approach
    println!("Base64 Encoding:");
    let base64_overhead = test_image_size * 4 / 3; // ~33% overhead
    println!("  Original size: {} KB", test_image_size / 1000);
    println!("  Base64 size: {} KB", base64_overhead / 1000);
    println!("  Overhead: +{:.0}%", (base64_overhead - test_image_size) as f64 / test_image_size as f64 * 100.0);
    println!("  Transfer: Full payload in request");
    println!("  Latency: Single request");
    println!("  Use case: Small images, privacy-sensitive data");
    println!();

    // URL approach
    println!("URL Reference:");
    println!("  Reference size: ~100 bytes (URL string)");
    println!("  Image size: {} KB (fetched separately)", test_image_size / 1000);
    println!("  Overhead: Minimal in request");
    println!("  Transfer: Two requests (query + image fetch)");
    println!("  Latency: Depends on image server");
    println!("  Use case: Public images, already hosted content");
    println!();

    // Decision matrix
    println!("Decision Matrix:");
    println!("  ┌─────────────────┬─────────┬─────────┐");
    println!("  │ Criteria        │ Base64  │ URL     │");
    println!("  ├─────────────────┼─────────┼─────────┤");
    println!("  │ Small images    │ ✓       │ -       │");
    println!("  │ Large images    │ -       │ ✓       │");
    println!("  │ Privacy         │ ✓       │ -       │");
    println!("  │ Already hosted  │ -       │ ✓       │");
    println!("  │ Offline support │ ✓       │ -       │");
    println!("  │ Speed (local)   │ ✓       │ -       │");
    println!("  └─────────────────┴─────────┴─────────┘");
    println!();

    Ok(())
}

/// Demonstrates handling large images and batch processing
fn large_image_handling_example() -> Result<()> {
    println!("=== Large Image Handling ===\n");

    // Size limits
    let limits = ImageLimits {
        max_base64_size: 15_000_000, // 15MB limit
        max_decoded_size: 20_000_000, // ~20MB decoded
        recommended_max_dimension: 2048,
    };

    println!("Image Size Limits:");
    println!("  Max base64 size: {} MB", limits.max_base64_size / 1_000_000);
    println!("  Max decoded size: {} MB", limits.max_decoded_size / 1_000_000);
    println!("  Recommended max dimension: {}px", limits.recommended_max_dimension);
    println!();

    // Test cases
    let test_images = vec![
        ("Small image", 50_000, (640, 480)),
        ("Medium image", 500_000, (1920, 1080)),
        ("Large image", 5_000_000, (4000, 3000)),
        ("Very large", 20_000_000, (8000, 6000)),
        ("Too large", 25_000_000, (10000, 8000)),
    ];

    println!("Image validation:");
    for (name, size, dims) in &test_images {
        let result = validate_image(size, dims, &limits);
        let status = match result {
            Ok(()) => "✓ OK",
            Err(e) => e,
        };
        println!("  {} ({}x{}, {:.1}MB): {}",
            name, dims.0, dims.1, *size as f64 / 1_000_000.0, status);
    }
    println!();

    // Batch processing strategy
    println!("Batch Processing Strategy for Many Images:");
    println!("  1. Validate all images first");
    println!("  2. Group images by size category");
    println!("  3. Process small/medium images together");
    println!("  4. Handle large images individually");
    println!("  5. For very large images, consider:");
    println!("     - Tiling/splitting into regions");
    println!("     - Progressive loading");
    println!("     - Pre-analysis summary generation");
    println!();

    // Simulate batch processing
    println!("Example batch processing:");
    let batch_images: Vec<ImageData> = (1..=5).map(|i| ImageData {
        name: format!("image_{}.png", i),
        format: ImageFormat::Png,
        size_bytes: 100_000 * i,
        dimensions: ((800 * i) as u32, (600 * i) as u32),
        data: vec![0; 100_000 * i],
    }).collect();

    let start = Instant::now();

    println!("  Processing {} images...", batch_images.len());
    for (i, img) in batch_images.iter().enumerate() {
        println!("    [{}/{}] {} ({:.1}KB)",
            i + 1, batch_images.len(), img.name, img.size_bytes as f64 / 1024.0);
        // Simulate processing
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    println!("  Batch completed in {:?}", start.elapsed());
    println!();

    Ok(())
}

// ============== Helper Functions ==============

/// Simulates base64 encoding
fn simulate_base64_encode(data: &[u8]) -> String {
    // In real code: use base64 crate
    format!("base64({}_bytes)", data.len())
}

/// Simulates image preprocessing
fn preprocess_image(image: &ImageData, options: &PreprocessOptions) -> Result<ImageData> {
    // Calculate new dimensions maintaining aspect ratio
    let (orig_w, orig_h) = image.dimensions;
    let (max_w, max_h) = (options.max_width, options.max_height);

    let ratio = (orig_w as f64 / max_w as f64).max(orig_h as f64 / max_h as f64);
    let (new_w, new_h) = if ratio > 1.0 {
        ((orig_w as f64 / ratio) as u32, (orig_h as f64 / ratio) as u32)
    } else {
        (orig_w, orig_h)
    };

    // Estimate new size (rough approximation)
    let size_ratio = (new_w * new_h) as f64 / (orig_w * orig_h) as f64;
    let new_size = (image.size_bytes as f64 * size_ratio) as usize;

    // Apply format conversion impact
    let format = options.convert_to.unwrap_or(image.format);
    let format_factor = match format {
        ImageFormat::Jpeg => 0.3, // JPEG is smaller for photos
        ImageFormat::WebP => 0.25,
        ImageFormat::Png => 1.0,
        ImageFormat::Gif => 0.8,
    };

    Ok(ImageData {
        name: image.name.clone(),
        format,
        size_bytes: (new_size as f64 * format_factor) as usize,
        dimensions: (new_w, new_h),
        data: vec![0; (new_size as f64 * format_factor) as usize],
    })
}

/// Image size limits
struct ImageLimits {
    max_base64_size: usize,
    max_decoded_size: usize,
    recommended_max_dimension: u32,
}

/// Validates an image against limits
fn validate_image(size: &usize, dims: &(u32, u32), limits: &ImageLimits) -> Result<(), &'static str> {
    if *size > limits.max_base64_size {
        return Err("✗ Exceeds max size");
    }
    if dims.0 > limits.recommended_max_dimension * 2 || dims.1 > limits.recommended_max_dimension * 2 {
        return Err("⚠ Dimensions too large");
    }
    if dims.0 > limits.recommended_max_dimension || dims.1 > limits.recommended_max_dimension {
        return Err("⚠ Consider resizing");
    }
    Ok(())
}
