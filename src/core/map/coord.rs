use std::ops::{Add, Neg, Sub};

/// Hex Coordinates, cubic notation but axial stored.
///
/// This creates a rhombus shape of hex tiles, 0,0 in top left, 255,255 in bottom right, each row
/// down the rhombus shifts a half tile right compared to the prior.  Wraps cleanly left/right.
///
/// Coordinates can be acquired either via cubic via x/y/z (x/z stored, y calculated) or via axial q/r.
///
/// Axial coordinates are just cubic's x/z where y is -x-z since x+y+z=0 to get the cubic plane.
///
/// ```
/// let coord = over_simple_game_1::core::map::coord::Coord::new_axial(0, 1);
/// assert_eq!(coord.x(), 0);
/// assert_eq!(coord.y(), -1);
/// assert_eq!(coord.z(), 1);
/// assert_eq!(coord.q(), 0);
/// assert_eq!(coord.r(), 1);
/// ```
#[derive(Clone, Copy, Default, Debug, Hash, PartialOrd, PartialEq, Ord, Eq)]
pub struct Coord(i8, i8);

impl Coord {
	/// Uses axial coordinates to create a new `Coord`
	///
	/// Axial coordinate, 0,0 is top-left, 255,255 is bottom-right.
	///
	/// ```
	/// let coord = over_simple_game_1::core::map::coord::Coord::new_axial(0, 1);
	/// assert_eq!(coord.q(), 0);
	/// assert_eq!(coord.r(), 1);
	/// ```
	pub fn new_axial(q: i8, r: i8) -> Coord {
		Coord(q, r)
	}

	/// Uses cubic coordinates to create a new `Coord`
	///
	/// cubic coordinates are the 3 axis of a 3d Cube, though constrained to the diagonal plane as
	/// `x + y + z = 0`.
	///
	/// ```
	/// let coord = over_simple_game_1::core::map::coord::Coord::new_axial(0, 1);
	/// assert_eq!(coord.x(), 0);
	/// assert_eq!(coord.y(), -1);
	/// assert_eq!(coord.z(), 1);
	/// ```
	pub fn new_cubic(x: i8, y: i8, z: i8) -> Coord {
		assert_eq!(x.wrapping_add(y).wrapping_add(z), 0);
		Coord(x, z)
	}

	/// Uses linear (pixel) coordinate to create a new `Coord`
	///
	/// Currently this just treats the hex tiles like they are offset rectangles,
	/// will refine it later.
	///
	/// ```
	/// # use over_simple_game_1::core::map::coord::Coord;
	/// assert_eq!(Coord::from_linear(0.0, 0.0), Coord::new_axial(0, 0));
	/// assert_eq!(Coord::from_linear(1.0, 0.0), Coord::new_axial(1, 0));
	/// assert_eq!(Coord::from_linear(0.5, 1.0), Coord::new_axial(0, 1));
	/// assert_eq!(Coord::from_linear(1.5, 1.0), Coord::new_axial(1, 1));
	/// assert_eq!(Coord::from_linear(-1.0, 0.0), Coord::new_axial(-1, 0));
	/// assert_eq!(Coord::from_linear(-0.5, -1.0), Coord::new_axial(0, -1));
	/// assert_eq!(Coord::from_linear(-1.5, -1.0), Coord::new_axial(-1, -1));
	/// ```
	pub fn from_linear(x: f32, y: f32) -> Coord {
		let x = x - y * 0.5;
		// let q = x.round().rem(256.0) as i8;
		// let r = y.round().rem(256.0) as i8;
		// let q = (x.round() as isize & 0xFF) as u8;
		// let r = (y.round() as isize & 0xFF) as u8;
		let q = x.round() as i8;
		let r = y.round() as i8;
		Coord::new_axial(q, r)
		// let s3 = 3.0f32.sqrt();
		// let is3 = 1.0 / s3;
		// let fx = (-2.0 / 3.0) * x;
		// let fy = (1.0 / 3.0) * x + is3 * y;
		// let fz = (1.0 / 3.0) * x - is3 * y;
		// let a = (fx - fy).ceil();
		// let b = (fy - fz).ceil();
		// let c = (fz - fx).ceil();
		// let x = ((a - c) / 3.0).round() as i8;
		// let y = ((b - a) / 3.0).round() as i8;
		// let z = ((c - b) / 3.0).round() as i8;
		// Coord::new_cubic(x, y, z)
		// let s3 = 3.0f32.sqrt();
		// x /= s3;
		// y /= s3;
		// let p = (x + s3 * y + 1.0).sqrt();
		// let q = (((2.0 * x + 1.0).floor() + p) / 3.0).floor();
		// let r = ((((-x + s3 * y + 1.0).floor()) + p) / 3.0).floor();
		// Coord::new_axial(q as i8, r as i8)
	}

	pub fn to_linear(self) -> (f32, f32) {
		let q = self.0 as f32;
		let r = self.1 as f32;
		let offset_x = r * 0.5;
		(q + offset_x, r)
	}

	pub fn q(&self) -> i8 {
		self.0
	}

	pub fn r(&self) -> i8 {
		self.1
	}

	pub fn to_axial_tuple(&self) -> (i8, i8) {
		(self.q(), self.r())
	}

	pub fn x(&self) -> i8 {
		self.0
	}

	pub fn y(&self) -> i8 {
		self.0.wrapping_neg().wrapping_sub(self.1)
	}

	pub fn z(&self) -> i8 {
		self.1
	}

	pub fn to_cubic_tuple(&self) -> (i8, i8, i8) {
		(self.x(), self.y(), self.z())
	}

	pub fn idx(self, max_x: u8, max_z: u8, wraps_x: bool) -> Option<usize> {
		if self.1 as u8 > max_z || (!wraps_x && self.0 as u8 > max_x) {
			return None;
		}
		let x = (self.0 as u8) as usize % (max_x as usize + 1);
		let z = (self.1 as u8) as usize;
		Some((z * max_x as usize) + x)

		// let x = (self.0 as u8) as usize % (max_x as usize + 1);
		// let z = (self.1 as u8) as usize % (max_z as usize + 1);
		// (z * max_x as usize) + x

		// let y1 = self.1 as u8;
		// let y2 = y1 as usize;
		// let m1 = max_x as usize;
		// let x1 = self.0 as u8;
		// let x2 = x1 as usize;
		// let y3 = y2 * m1;
		// let r = y3 + x2;
		// r

		// (self.1 as u8 as usize * max_x as usize) + self.0 as u8 as usize
	}

	pub fn scale(self, scale: i8) -> Coord {
		Coord(self.0.wrapping_mul(scale), self.1.wrapping_mul(scale))
	}

	pub fn cw(self) -> Coord {
		let (x, y, z) = (-self).to_cubic_tuple();
		Coord::new_cubic(z, x, y)
	}

	pub fn ccw(self) -> Coord {
		let (x, y, z) = (-self).to_cubic_tuple();
		Coord::new_cubic(y, z, x)
	}

	pub fn cw_offset(self, center: Coord) -> Coord {
		(center - self).cw() + center
	}

	pub fn ccw_offset(self, center: Coord) -> Coord {
		(center - self).ccw() + center
	}

	// pub fn iterate_linear_box(self, to: Coord) -> CoordLinearViewIterator {
	// 	let offset_x: i8 = to.1.wrapping_sub(self.1).wrapping_add(1).wrapping_div(2);
	// 	let stride: i8 = to.0.wrapping_sub(self.0).wrapping_add(offset_x);
	// 	CoordLinearViewIterator {
	// 		stride,
	// 		stride_remaining: stride,
	// 		from: self,
	// 		current: self,
	// 		to,
	// 		done: false,
	// 	}
	// }

	pub fn iter_neighbors_ring(self, distance: i8) -> CoordRingIterator {
		CoordRingIterator::new(self, distance)
	}

	pub fn iter_neighbors(self, distance: i8) -> CoordNeighborIterator {
		CoordNeighborIterator::new(self, distance)
	}
}

impl Add for Coord {
	type Output = Coord;

	fn add(self, rhs: Self) -> Self::Output {
		Coord(self.0.wrapping_add(rhs.0), self.1.wrapping_add(rhs.1))
	}
}

impl Sub for Coord {
	type Output = Coord;

	fn sub(self, rhs: Self) -> Self::Output {
		Coord(self.0.wrapping_sub(rhs.0), self.1.wrapping_sub(rhs.1))
	}
}

impl Neg for Coord {
	type Output = Coord;

	fn neg(self) -> Self::Output {
		Coord(-self.0, -self.1)
	}
}

// // Unsure if this is good...
// pub struct CoordLinearViewIterator {
// 	stride: i8,
// 	stride_remaining: i8,
// 	from: Coord,
// 	current: Coord,
// 	to: Coord,
// 	done: bool,
// }
//
// impl Iterator for CoordLinearViewIterator {
// 	type Item = Coord;
//
// 	fn next(&mut self) -> Option<Self::Item> {
// 		if self.done {
// 			return None;
// 		}
// 		let coord = if self.stride_remaining == 0 {
// 			if self.current.1 == self.to.1 {
// 				self.done = true;
// 				self.current
// 			} else {
// 				let coord = self.current;
// 				self.current.1 = self.current.1.wrapping_add(1);
// 				let offset_x = self
// 					.current
// 					.1
// 					.wrapping_sub(self.from.1)
// 					.wrapping_add(1)
// 					.wrapping_div(2);
// 				self.current.0 = self.from.0.wrapping_add(offset_x);
// 				self.stride_remaining = self.stride;
// 				coord
// 			}
// 		} else {
// 			let coord = self.current;
// 			self.current.0 = self.current.0.wrapping_add(1);
// 			self.stride_remaining -= 1;
// 			coord
// 		};
// 		Some(coord)
// 	}
// }

pub struct CoordRingIterator {
	point: Option<Coord>,
	side: Coord,
	distance: i8,
	offset: i8,
}

impl CoordRingIterator {
	fn new(center: Coord, distance: i8) -> CoordRingIterator {
		if distance == 0 {
			CoordRingIterator {
				point: Some(center),
				side: Coord(-1, 1).ccw(),
				distance: 0,
				offset: 0,
			}
		} else {
			let side = Coord(1, 0);
			CoordRingIterator {
				point: Some(center + side.scale(distance)),
				side: (-side).ccw(),
				distance,
				offset: 0,
			}
		}
	}
}

impl Iterator for CoordRingIterator {
	type Item = Coord;

	fn next(&mut self) -> Option<Self::Item> {
		let point = self.point? + self.side.scale(self.offset);
		if self.offset >= self.distance {
			self.offset = 1;
			self.side = self.side.cw();
			if self.side == Coord(-1, 1) {
				self.point = None;
			} else {
				self.point = Some(point)
			}
		} else {
			self.offset += 1;
		}
		Some(point)
	}
}

pub struct CoordNeighborIterator {
	ring_iter: CoordRingIterator,
	center: Coord,
	distance: i8,
}

impl CoordNeighborIterator {
	fn new(center: Coord, distance: i8) -> CoordNeighborIterator {
		CoordNeighborIterator {
			ring_iter: CoordRingIterator::new(center, 0),
			center,
			distance,
		}
	}
}

impl Iterator for CoordNeighborIterator {
	type Item = Coord;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(coord) = self.ring_iter.next() {
			return Some(coord);
		}
		if self.distance <= self.ring_iter.distance {
			return None;
		}
		self.ring_iter = CoordRingIterator::new(self.center, self.ring_iter.distance + 1);
		self.ring_iter.next()
	}
}

// #[cfg(test)]
// mod coord_hex_tests {
// 	#[test]
// 	fn from_linear_test() {
// 	}
// }
