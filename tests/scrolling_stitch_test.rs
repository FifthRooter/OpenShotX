use image::RgbaImage;
use openshotx::scrolling::{stitch_frames, CapturedFrame, ScrollCaptureConfig};

fn make_test_image(width: u32, height: u32, r: u8, g: u8, b: u8) -> RgbaImage {
    RgbaImage::from_fn(width, height, |_, _| image::Rgba([r, g, b, 255]))
}

fn make_gradient_image(width: u32, height: u32) -> RgbaImage {
    RgbaImage::from_fn(width, height, |x, y| image::Rgba([x as u8, y as u8, 128, 255]))
}

fn make_shifted_image(base: &RgbaImage, shift_y: i32) -> RgbaImage {
    let (width, height) = base.dimensions();
    let mut shifted = RgbaImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let src_y = y as i32 - shift_y;
            if src_y >= 0 && src_y < height as i32 {
                let pixel = base.get_pixel(x, src_y as u32);
                shifted.put_pixel(x, y, *pixel);
            }
        }
    }
    shifted
}

#[test]
fn test_calculate_diff_identical() {
    let img = make_test_image(100, 100, 255, 0, 0);
    let frame1 = CapturedFrame::new(img.clone());
    let frame2 = CapturedFrame::new(img);

    let diff = frame1.calculate_diff(&frame2);
    assert_eq!(diff, 0, "Identical images should have zero diff");
}

#[test]
fn test_calculate_diff_different() {
    let red = make_test_image(100, 100, 255, 0, 0);
    let blue = make_test_image(100, 100, 0, 0, 255);

    let frame1 = CapturedFrame::new(red);
    let frame2 = CapturedFrame::new(blue);

    let diff = frame1.calculate_diff(&frame2);
    assert!(
        diff > 150,
        "Red vs Blue should have high diff, got {}",
        diff
    );
}

#[test]
fn test_calculate_diff_gradient() {
    let gradient = make_gradient_image(100, 100);
    let shifted = make_shifted_image(&gradient, 2);

    let frame1 = CapturedFrame::new(gradient);
    let frame2 = CapturedFrame::new(shifted);

    let diff = frame1.calculate_diff(&frame2);
    assert!(
        diff < 50,
        "Shifted gradient should have low diff, got {}",
        diff
    );
}

#[test]
fn test_find_overlap_exact() {
    let img1 = make_test_image(100, 100, 128, 128, 128);
    let img2 = make_shifted_image(&img1, 50);

    let frame1 = CapturedFrame::new(img1);
    let frame2 = CapturedFrame::new(img2);

    let overlap = frame1.find_overlap(&frame2, 0.1);
    assert!(
        overlap.is_some(),
        "Should find overlap between shifted identical images"
    );
    let (offset, _) = overlap.unwrap();
    assert!(
        offset >= 40 && offset <= 60,
        "Overlap should be around 50px, got {}",
        offset
    );
}

#[test]
fn test_find_overlap_no_match() {
    let top_half_red = make_test_image(100, 100, 255, 0, 0);
    let bottom_half_blue = make_test_image(100, 100, 0, 0, 255);

    let frame1 = CapturedFrame::new(top_half_red);
    let frame2 = CapturedFrame::new(bottom_half_blue);

    let overlap = frame1.find_overlap(&frame2, 0.1);
    assert!(
        overlap.is_none(),
        "Completely different content should not find overlap, got {:?}",
        overlap
    );
}

#[test]
fn test_find_overlap_similar() {
    let gradient1 = make_gradient_image(100, 100);
    let gradient2 = make_shifted_image(&gradient1, 30);

    let frame1 = CapturedFrame::new(gradient1);
    let frame2 = CapturedFrame::new(gradient2);

    let overlap = frame1.find_overlap(&frame2, 0.1);
    assert!(
        overlap.is_some(),
        "Similar content should find overlap, got {:?}",
        overlap
    );
}

#[test]
fn test_stitch_frames_simple() {
    let config = ScrollCaptureConfig::default();

    let mut top = make_test_image(100, 100, 255, 0, 0);
    let bottom = make_test_image(100, 100, 0, 255, 0);

    image::imageops::replace(&mut top, &bottom, 0, 50);

    let frame1 = CapturedFrame::new(top);
    let frame2 = CapturedFrame::new(bottom);

    let result = stitch_frames(&[frame1, frame2], &config).unwrap();

    assert_eq!(result.image.width(), 100, "Width should be 100");
    assert!(
        result.image.height() >= 100,
        "Stitched image should be at least 100px tall, got {}",
        result.image.height()
    );
}

#[test]
fn test_stitch_frames_no_overlap() {
    let config = ScrollCaptureConfig::default();

    let red = make_test_image(100, 100, 255, 0, 0);
    let blue = make_test_image(100, 100, 0, 0, 255);

    let frame1 = CapturedFrame::new(red);
    let frame2 = CapturedFrame::new(blue);

    let result = stitch_frames(&[frame1, frame2], &config).unwrap();

    assert_eq!(result.image.width(), 100, "Width should be 100");
    assert_eq!(
        result.image.height(),
        200,
        "Non-overlapping frames should concatenate to 200px, got {}",
        result.image.height()
    );
}
