use opencv::{
	prelude::*,
	imgproc::*,
	core::*,
};

pub fn resize_with_padding(img: &Mat, new_shape: [i32;2]) -> Mat {
	let img_shape = [img.cols(), img.rows()];
	let width: i32;
	let height: i32;
	if img_shape[0] as f64 / img_shape[1] as f64 > new_shape[0] as f64 / new_shape[1] as f64 {
		width = new_shape[0];
		height = (new_shape[0] as f64 / img_shape[0] as f64 * img_shape[1] as f64) as i32;
	} else {
		width = (new_shape[1] as f64 / img_shape[1] as f64 * img_shape[0] as f64) as i32;
		height = new_shape[1];
	}

	let mut resized = Mat::default();
	resize(
		img,
		&mut resized,
		Size { width, height },
		0.0, 0.0,
		INTER_LINEAR)
		.expect("resize_with_padding: resize [FAILED]");

	let delta_w = new_shape[0] - width;
	let delta_h = new_shape[1] - height;
	let (top, bottom) = (delta_h / 2, delta_h - delta_h / 2);
	let (left, right) = (delta_w / 2, delta_w - delta_w / 2);
		
	let mut rslt = Mat::default();
	copy_make_border(
		&resized,
		&mut rslt,
		top, bottom, left, right,
		BORDER_CONSTANT,
		Scalar::new(0.0, 0.0, 0.0, 0.0))
		.expect("resize_with_padding: copy_make_border [FAILED]");
	rslt
}

pub fn yuyv422_to_rgb(yuyv: &[u8]) -> Vec<u8> {
    let mut rgb = vec![0u8; yuyv.len() * 3 / 2];
    for i in 0..(yuyv.len() / 4) {
        let y1 = yuyv[i * 4] as i32;
        let u = yuyv[i * 4 + 1] as i32;
        let y2 = yuyv[i * 4 + 2] as i32;
        let v = yuyv[i * 4 + 3] as i32;

        let c1 = y1 - 16;
        let c2 = y2 - 16;
        let d = u - 128;
        let e = v - 128;

        let r1 = (298 * c1 + 409 * e + 128) >> 8;
        let g1 = (298 * c1 - 100 * d - 208 * e + 128) >> 8;
        let b1 = (298 * c1 + 516 * d + 128) >> 8;

        let r2 = (298 * c2 + 409 * e + 128) >> 8;
        let g2 = (298 * c2 - 100 * d - 208 * e + 128) >> 8;
        let b2 = (298 * c2 + 516 * d + 128) >> 8;

        rgb[i * 6] = b1.clamp(0, 255) as u8;
        rgb[i * 6 + 1] = g1.clamp(0, 255) as u8;
        rgb[i * 6 + 2] = r1.clamp(0, 255) as u8;
        rgb[i * 6 + 3] = b2.clamp(0, 255) as u8;
        rgb[i * 6 + 4] = g2.clamp(0, 255) as u8;
        rgb[i * 6 + 5] = r2.clamp(0, 255) as u8;
    }
    rgb
}

pub fn draw_connections(img: &mut Mat, keypoints: &[f32], threshold: f32) {
    let base: f32 = img.rows().max(img.cols()) as f32;
    let pad_x: i32 = if img.rows() > img.cols() { (img.rows() - img.cols()) / 2 } else { 0 };
    let pad_y: i32 = if img.cols() > img.rows() { (img.cols() - img.rows()) / 2 } else { 0 };

    let connections = [
        (0, 1), (0, 2), (1, 3), (2, 4), // head
        (0, 5), (0, 6), (5, 6), // shoulders
        (5, 7), (7, 9), // left arm
        (6, 8),  (8, 10), // right arm
        (5, 11), (6, 12), (11, 12), // body
        (11, 13), (13, 15), // left leg
        (12, 14), (14, 16), // right leg
    ];

    for &(start, end) in &connections {
        let start_confidence = keypoints[start * 3 + 2];
        let end_confidence = keypoints[end * 3 + 2];

        if start_confidence > threshold && end_confidence > threshold {
            let start_x = base as i32 - ((keypoints[start * 3 + 1] * base) as i32 - pad_x);
            let start_y = (keypoints[start * 3] * base) as i32 - pad_y;
            let end_x = base as i32 - ((keypoints[end * 3 + 1] * base) as i32 - pad_x);
            let end_y = (keypoints[end * 3] * base) as i32 - pad_y;

            line(img, 
                 Point::new(start_x, start_y),
                 Point::new(end_x, end_y),
                 Scalar::new(0.0, 255.0, 0.0, 0.0), // Green color
                 2, // Line thickness
                 LINE_AA, 
                 0).expect("Draw line [FAILED]");
        }
    }
}

pub fn draw_keypoints(img: &mut Mat, keypoints: &[f32], threshold: f32) {
    let base: f32;
    let pad_x: i32;
    let pad_y: i32;

    if img.rows() > img.cols() {
        base = img.rows() as f32;
        pad_x = (img.rows() - img.cols()) / 2;
        pad_y = 0;
    } else {
        base = img.cols() as f32;
        pad_x = 0;
        pad_y = (img.cols() - img.rows()) / 2;
    }

    let mut points = Vec::new();
    for index in 0..17 {
        let y_ratio = keypoints[index * 3];
        let x_ratio = keypoints[index * 3 + 1];
        let confidence = keypoints[index * 3 + 2];

        if confidence > threshold {
            // Adjust the x-coordinate to account for the flip
            let adjusted_x = base as i32 - ((x_ratio * base) as i32 - pad_x);
            let adjusted_y = (y_ratio * base) as i32 - pad_y;

            circle(img,
                Point { x: adjusted_x, y: adjusted_y },
                5, // Circle radius
                Scalar::new(0.0, 0.0, 255.0, 0.0), // Red color for points
                -1, LINE_AA, 0).expect("Draw circle [FAILED]");

            // Store the point for drawing lines later
            points.push(Point { x: adjusted_x, y: adjusted_y });
        } else {
            // Add a placeholder point if confidence is below threshold
            points.push(Point { x: -1, y: -1 });
        }
    }
}
