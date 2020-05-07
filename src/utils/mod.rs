mod bruh;

// Functions for line and block tests
// Useful for collision detection and line-of-sight checks

// AABB collision test: https://tutorialedge.net/gamedev/aabb-collision-detection-tutorial/
pub fn aabb_test(x1: i32, y1: i32, s1: i32, x2: i32, y2: i32, s2: i32) -> bool {
	if x1 < x2 + s2 &&
		x1 + s1 > x2 &&
		y1 < y2 + s2 &&
		y1 + s1 > y2 {
		return true;
	}
	return false;
}

// Intersection of two lines in 2D: http://paulbourke.net/geometry/pointlineplane/
pub fn line2line(x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32, x4: i32, y4: i32) -> bool {
	let denominator = (x2 - x1) * (y4 - y3) - (y2 - y1) * (x4 - x3);
	let numerator1  = (x2 - x1) * (y1 - y3) - (y2 - y1) * (x1 - x3);
	let numerator2  = (x4 - x3) * (y1 - y3) - (y4 - y3) * (x1 - x3);

	if denominator == 0 {
		return numerator1 == 0 && numerator2 == 0;
	}

	// Using floats here to avoid roundoff errors
	let division1 = numerator1 as f32 / denominator as f32;
	let division2 = numerator2 as f32 / denominator as f32;

	if division1 >= 0.0 && division1 <= 1.0 && division2 >= 0.0 && division2 <= 1.0 {
		return true;
	}

	return false;
}

// Intersection of a line and a box in 2D
// x1, y1: where the line starts
// x2, y2: where the line finishes
// bx, by, bs: where the box is and its size
pub fn line2box(x1: i32, y1: i32, x2: i32, y2: i32, bx: i32, by: i32, bs: i32) -> bool {
	let left   = line2line(x1, y1, x2, y2, bx,      by, bx,      by + bs);
	let right  = line2line(x1, y1, x2, y2, bx + bs, by, bx + bs, by + bs);

	let top    = line2line(x1, y1, x2, y2, bx, by,      bx + bs, by);
	let bottom = line2line(x1, y1, x2, y2, bx, by + bs, bx + bs, by + bs);

	if left || right || top || bottom {
		return true;
	}

	return false;
}

pub fn distance2d(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
	let tmp = i32::pow(x1 - x2, 2) + i32::pow(y1 - y2, 2);
	return (tmp as f32).sqrt().ceil() as i32;
}

pub fn test()
{
	bruh::bruh();
}
