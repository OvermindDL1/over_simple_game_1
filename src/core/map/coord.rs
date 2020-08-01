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

	pub const CENTER_TO_POINT: f32 = 0.5773502691896258; //0.5/(TAU/12.0).cos(); // `cos` is not const capable for some reason...;
	const SQRT3: f32 = 1.732050807568877; //3.0f32.sqrt(); // `sqrt` is not const capable either, why?!

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
	/// let (x, y) = Coord::to_linear(Coord::new_axial(7, 28));
	/// assert_eq!(Coord::from_linear(x, y), Coord::new_axial(7, 28));
	/// ```
	pub fn from_linear(x: f32, y: f32) -> Coord {
		let s3 = 3.0f32.sqrt();
		let segment = (x + s3 * y + 1.0).floor();
		let q = (((2.0 * x + 1.0).floor() + segment) / 3.0).floor();
		let r = ((segment + (-x + s3 * y + 1.0).floor()) / 3.0).floor();
		Coord::new_axial((q - r) as i8, r as i8)
	}

	/// Get this hex coordinate in linear space where the point is centered on the hex coordinate.
	///
	/// Since hex tiles are pointy-top then their top height is 1.0 but the side width is a bit
	/// thinner, about 0.8660254.
	///
	/// ```
	/// # use assert_approx_eq::assert_approx_eq;
	/// # use over_simple_game_1::core::map::coord::Coord;
	/// let c00 = Coord::new_axial(0, 0).to_linear();
	/// let c01 = Coord::new_axial(0, 1).to_linear();
	/// let c02 = Coord::new_axial(0, 2).to_linear();
	/// assert_approx_eq!(c00.0, 0.0);
	/// assert_approx_eq!(c00.1, 0.0);
	/// assert_approx_eq!(c01.0, 0.5);
	/// assert_approx_eq!(c01.1, 0.8660254);
	/// assert_approx_eq!(c02.0, 1.0);
	/// assert_approx_eq!(c02.1, 1.7320508);
	/// ```
	pub fn to_linear(self) -> (f32, f32) {
		let q = self.0 as f32;
		let r = self.1 as f32;
		let x = Self::CENTER_TO_POINT * (Self::SQRT3 * q + Self::SQRT3 / 2.0 * r);
		let y = Self::CENTER_TO_POINT * (3.0 / 2.0 * r);
		(x, y)
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

pub struct MapCoord {
	pub map: u32,
	pub coord: Coord,
}

#[cfg(test)]
mod coord_tests {
	use super::*;
	use proptest::prelude::*;

	fn rand_coord_strategy() -> BoxedStrategy<Coord> {
		(any::<i8>(), any::<i8>())
			.prop_map(|(q, r)| Coord::new_axial(q, r))
			.boxed()
	}

	proptest!(
		#[test]
		fn coord_to_linear_from_linear(q: i8, r: i8) {
			let axial = Coord::new_axial(q, r);

			let (x, y) = axial.to_linear();
			let axial_to_from = Coord::from_linear(x, y);

			prop_assert_eq!((x, y, axial), (x, y, axial_to_from));
		}
	);

	proptest!(
		#[test]
		fn sum_xyz(coord in rand_coord_strategy()) {
			prop_assert_eq!(
				coord.x().wrapping_add(
				coord.y().wrapping_add(
				coord.z())),
				0
			);
		}
	);

	// I think this should work, but that will require wrapping z and
	// negative coords working. TODO: uncomment when fully implemented
	//
	// proptest!(
	//     #[test]
	//     fn wrapping_get_always_returns(
	//         coord in rand_coord_strategy(),
	//         max_x: u8,
	//         max_z: u8
	//     ) {
	//         prop_assert_ne!(coord.idx(max_x, max_z, true), None);
	//     }
	// );
}
