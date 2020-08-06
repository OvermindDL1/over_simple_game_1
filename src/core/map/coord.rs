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
pub struct Coord(u8, u8);

impl Coord {
	/// Uses axial coordinates to create a new `Coord`
	///
	/// Axial coordinate, 0,0 is top-left, 255,255 is bottom-right.
	///
	/// In Cubic then `q` is `x` and `r` is `z`, `y` is generated via `-q-r`.
	///
	/// ```
	/// let coord = over_simple_game_1::core::map::coord::Coord::new_axial(0, 1);
	/// assert_eq!(coord.q(), 0);
	/// assert_eq!(coord.r(), 1);
	/// ```
	pub fn new_axial(q: u8, r: u8) -> Coord {
		Coord(q, r)
	}

	pub const CENTER_TO_POINT: f32 = 0.57735026; // 0.5773502691896258; //0.5/(TAU/12.0).cos(); // `cos` is not const capable for some reason...;
	const SQRT3: f32 = 1.7320508; // 1.732050807568877; //3.0f32.sqrt(); // `sqrt` is not const capable either, why?!

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
	/// assert_eq!(Coord::from_linear(-1.0, 0.0), Coord::new_axial(255, 0));
	/// assert_eq!(Coord::from_linear(-0.5, -1.0), Coord::new_axial(0, 255));
	/// assert_eq!(Coord::from_linear(-1.5, -1.0), Coord::new_axial(255, 255));
	/// let (x, y) = Coord::to_linear(Coord::new_axial(7, 28));
	/// assert_eq!(Coord::from_linear(x, y), Coord::new_axial(7, 28));
	/// ```
	pub fn from_linear(x: f32, y: f32) -> Coord {
		let s3 = 3.0f32.sqrt();
		let segment = (x + s3 * y + 1.0).floor();
		let q = (((2.0 * x + 1.0).floor() + segment) / 3.0).floor();
		let r = ((segment + (-x + s3 * y + 1.0).floor()) / 3.0).floor();
		Coord::new_axial((q - r) as i16 as u8, r as i16 as u8)
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

	pub fn q(&self) -> u8 {
		self.0
	}

	pub fn r(&self) -> u8 {
		self.1
	}

	pub fn to_axial_tuple(&self) -> (u8, u8) {
		(self.q(), self.r())
	}

	pub fn x(&self) -> i16 {
		self.0 as i16
	}

	pub fn y(&self) -> i16 {
		self.x().wrapping_neg().wrapping_sub(self.z())
	}

	pub fn z(&self) -> i16 {
		self.1 as i16
	}

	pub fn to_cubic_tuple(&self) -> (i16, i16, i16) {
		(self.x(), self.y(), self.z())
	}

	pub fn idx(self, max_x: u8, max_z: u8, wraps_x: bool) -> Option<usize> {
		if self.1 > max_z || (!wraps_x && self.0 > max_x) {
			return None;
		}
		let x = (self.0 as u8) as usize % (max_x as usize + 1);
		let z = (self.1 as u8) as usize;
		Some((z * (max_x as usize + 1)) + x)
	}

	pub fn offset_by(
		self,
		offset: CoordOrientation,
		width: u8,
		height: u8,
		wraps_x: bool,
	) -> Option<Coord> {
		let width = width as isize + 1;
		let height = height as isize + 1;
		let mut q = self.0 as isize + offset.0 as isize;
		let r = self.1 as isize + offset.1 as isize;
		if r < 0 || r > height {
			return None;
		}
		let r = r as u8;
		if wraps_x {
			q = q.rem_euclid(width);
		} else if q < 0 || q > width {
			return None;
		}
		let q = q as u8;
		Some(Coord::new_axial(q, r))
	}

	pub fn distance_to(self, other: Coord) -> u8 {
		let (dx, dy, dz) = (self - other).to_cubic_tuple();
		std::cmp::max(
			std::cmp::max(dx.abs() as u8, dy.abs() as u8),
			dz.abs() as u8,
		)
	}

	// pub fn as_coord_orientation(self) -> CoordOrientation {
	// 	CoordOrientation(self.0, self.1)
	// }

	pub fn iter_neighbors_ring(self, distance: u8) -> CoordRingIterator {
		CoordRingIterator::new(self, distance)
	}

	pub fn iter_neighbors(self, distance: u8) -> CoordNeighborIterator {
		CoordNeighborIterator::new(self, distance)
	}
}

impl Add<CoordOrientation> for Coord {
	type Output = Coord;

	fn add(self, rhs: CoordOrientation) -> Self::Output {
		Coord(
			(self.0 as i8).wrapping_add(rhs.0) as u8,
			(self.1 as i8).wrapping_add(rhs.1) as u8,
		)
	}
}

impl Sub<CoordOrientation> for Coord {
	type Output = Coord;

	fn sub(self, rhs: CoordOrientation) -> Self::Output {
		Coord(
			(self.0 as i8).wrapping_sub(rhs.0) as u8,
			(self.1 as i8).wrapping_sub(rhs.1) as u8,
		)
	}
}

impl Sub<Coord> for Coord {
	type Output = CoordOrientation;

	fn sub(self, rhs: Coord) -> Self::Output {
		CoordOrientation(
			self.0.wrapping_sub(rhs.0) as i8,
			self.1.wrapping_sub(rhs.1) as i8,
		)
	}
}

pub struct CoordRingIterator {
	center: Coord,
	offset: CoordOrientationRingIterator,
}

impl CoordRingIterator {
	fn new(center: Coord, distance: u8) -> CoordRingIterator {
		CoordRingIterator {
			center,
			offset: CoordOrientationRingIterator::new(distance),
		}
	}
}

impl Iterator for CoordRingIterator {
	type Item = Coord;

	fn next(&mut self) -> Option<Self::Item> {
		let offset = self.offset.next()?;
		Some(self.center + offset)
	}
}

pub struct CoordNeighborIterator {
	center: Coord,
	offset: CoordOrientationNeighborIterator,
}

impl CoordNeighborIterator {
	fn new(center: Coord, distance: u8) -> CoordNeighborIterator {
		CoordNeighborIterator {
			center,
			offset: CoordOrientationNeighborIterator::new(distance),
		}
	}
}

impl Iterator for CoordNeighborIterator {
	type Item = Coord;

	fn next(&mut self) -> Option<Self::Item> {
		let offset = self.offset.next()?;
		Some(self.center + offset)
	}
}

#[derive(Clone, Copy, Default, Debug, Hash, PartialOrd, PartialEq, Ord, Eq)]
pub struct CoordOrientation(i8, i8);

impl CoordOrientation {
	/// Uses axial coordinates to create a new `CoordOrientation`
	///
	/// Axial coordinate, 0,0 is top-left, 255,255 is bottom-right.
	///
	/// ```
	/// let coord = over_simple_game_1::core::map::coord::Coord::new_axial(0, 1);
	/// assert_eq!(coord.q(), 0);
	/// assert_eq!(coord.r(), 1);
	/// ```
	pub fn new_axial(q: i8, r: i8) -> CoordOrientation {
		CoordOrientation(q, r)
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

	pub fn to_linear(self) -> (f32, f32) {
		let q = self.0 as f32;
		let r = self.1 as f32;
		let x = Coord::CENTER_TO_POINT * (Coord::SQRT3 * q + Coord::SQRT3 / 2.0 * r);
		let y = Coord::CENTER_TO_POINT * (3.0 / 2.0 * r);
		(x, y)
	}

	pub fn distance_to(self, other: CoordOrientation) -> u8 {
		let (dx, dy, dz) = (self - other).to_cubic_tuple();
		std::cmp::max(
			std::cmp::max(dx.abs() as u8, dy.abs() as u8),
			dz.abs() as u8,
		)
	}

	pub fn scale(self, scale: i8) -> CoordOrientation {
		CoordOrientation(self.0.wrapping_mul(scale), self.1.wrapping_mul(scale))
	}

	pub fn cw(self) -> CoordOrientation {
		let (_x, y, z) = (-self).to_cubic_tuple();
		// CoordOrientation::new_cubic(z, x, y)
		CoordOrientation::new_axial(z, y)
	}

	pub fn ccw(self) -> CoordOrientation {
		let (x, y, _z) = (-self).to_cubic_tuple();
		// CoordOrientation::new_cubic(y, z, x)
		CoordOrientation::new_axial(y, x)
	}

	// pub fn as_coord(self) -> Coord {
	// 	Coord(self.0, self.1)
	// }

	pub fn iter_neighbors_ring(distance: u8) -> CoordOrientationRingIterator {
		CoordOrientationRingIterator::new(distance)
	}

	pub fn iter_neighbors(distance: u8) -> CoordOrientationNeighborIterator {
		CoordOrientationNeighborIterator::new(distance)
	}
}

impl Add for CoordOrientation {
	type Output = CoordOrientation;

	fn add(self, rhs: Self) -> Self::Output {
		CoordOrientation(self.0.wrapping_add(rhs.0), self.1.wrapping_add(rhs.1))
	}
}

impl Sub for CoordOrientation {
	type Output = CoordOrientation;

	fn sub(self, rhs: Self) -> Self::Output {
		CoordOrientation(self.0.wrapping_sub(rhs.0), self.1.wrapping_sub(rhs.1))
	}
}

impl Add<Coord> for CoordOrientation {
	type Output = Coord;

	fn add(self, rhs: Coord) -> Self::Output {
		Coord(
			(self.0 as u8).wrapping_add(rhs.0),
			(self.1 as u8).wrapping_add(rhs.1),
		)
	}
}

impl Sub<Coord> for CoordOrientation {
	type Output = Coord;

	fn sub(self, rhs: Coord) -> Self::Output {
		Coord(
			(self.0 as u8).wrapping_sub(rhs.0),
			(self.1 as u8).wrapping_sub(rhs.1),
		)
	}
}

impl Neg for CoordOrientation {
	type Output = CoordOrientation;

	fn neg(self) -> Self::Output {
		CoordOrientation(self.0.wrapping_neg(), self.1.wrapping_neg())
	}
}

pub struct CoordOrientationRingIterator {
	side: CoordOrientation,
	side_count: u8,
	distance: u8,
	offset: u8,
}

impl CoordOrientationRingIterator {
	pub fn new(distance: u8) -> CoordOrientationRingIterator {
		assert!(distance <= 127);
		if distance == 0 {
			CoordOrientationRingIterator {
				side_count: 5,
				side: CoordOrientation(0, 0),
				distance: 0,
				offset: 0,
			}
		} else {
			CoordOrientationRingIterator {
				side_count: 0,
				side: CoordOrientation(1, 0),
				distance,
				offset: 0,
			}
		}
	}
}

impl Iterator for CoordOrientationRingIterator {
	type Item = CoordOrientation;

	fn next(&mut self) -> Option<Self::Item> {
		if self.side_count > 5 {
			return None;
		}
		let side = self.side.scale(self.distance as i8);
		let offset = (-self.side).ccw().scale(self.offset as i8);
		self.offset += 1;
		if self.offset >= self.distance {
			self.offset = 0;
			self.side = self.side.cw();
			self.side_count += 1;
		}
		Some(side + offset)
	}
}

pub struct CoordOrientationNeighborIterator {
	ring_iter: CoordOrientationRingIterator,
	distance: u8,
}

impl CoordOrientationNeighborIterator {
	pub fn new(distance: u8) -> CoordOrientationNeighborIterator {
		CoordOrientationNeighborIterator {
			ring_iter: CoordOrientationRingIterator::new(0),
			distance,
		}
	}
}

impl Iterator for CoordOrientationNeighborIterator {
	type Item = CoordOrientation;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(coord) = self.ring_iter.next() {
			return Some(coord);
		}
		if self.distance <= self.ring_iter.distance {
			return None;
		}
		self.ring_iter = CoordOrientationRingIterator::new(self.ring_iter.distance + 1);
		self.ring_iter.next()
	}
}

#[cfg(test)]
mod coord_tests {
	use super::*;
	use proptest::prelude::*;
	use std::collections::HashSet;

	fn rand_coord_strategy() -> BoxedStrategy<Coord> {
		(any::<u8>(), any::<u8>())
			.prop_map(|(q, r)| Coord::new_axial(q, r))
			.boxed()
	}

	fn rand_coord_orientation_strategy() -> BoxedStrategy<CoordOrientation> {
		(any::<i8>(), any::<i8>())
			.prop_map(|(q, r)| CoordOrientation::new_axial(q, r))
			.boxed()
	}

	#[test]
	fn coord_orientation_ring_iterator_small_count() {
		{
			let mut iter = CoordOrientationRingIterator::new(0);
			assert_eq!(iter.next(), Some(CoordOrientation::new_axial(0, 0)));
			assert_eq!(iter.next(), None);
		}
		{
			let coords =
				CoordOrientationRingIterator::new(1).collect::<HashSet<CoordOrientation>>();
			assert!(coords.contains(&CoordOrientation::new_axial(1, 0)));
			assert!(coords.contains(&CoordOrientation::new_axial(0, 1)));
			assert!(coords.contains(&CoordOrientation::new_axial(-1, 1)));
			assert!(coords.contains(&CoordOrientation::new_axial(-1, 0)));
			assert!(coords.contains(&CoordOrientation::new_axial(0, -1)));
			assert!(coords.contains(&CoordOrientation::new_axial(1, -1)));
			assert_eq!(coords.len(), 6);
		}
	}

	proptest!(
		#![proptest_config(ProptestConfig::with_cases(30))]
		#[test]
		fn coord_orientation_ring_iterator_big_count(distance in 2..128u8) {
			let around = CoordOrientationRingIterator::new(distance)
				.collect::<HashSet<CoordOrientation>>();
			assert_eq!(
				around.len(),
				(distance as usize * 6),
				"Distance {} missing/extra values, generated: {:?}",
				distance,
				around
			);
		}
	);

	#[test]
	fn coord_orientation_neighbor_iterator_small_count() {
		let mut iter = CoordOrientationNeighborIterator::new(0);
		assert_eq!(iter.next(), Some(CoordOrientation::new_axial(0, 0)));
		assert_eq!(iter.next(), None);
	}

	proptest!(
		#![proptest_config(ProptestConfig::with_cases(30))]
		#[test]
		fn coord_orientation_neighbor_iterator_big_count(distance in 1..128u8) {
			let around = CoordOrientationNeighborIterator::new(distance)
				.collect::<HashSet<CoordOrientation>>();
			assert_eq!(
				around.len(),
				3 * ((distance as usize).pow(2) + distance as usize) + 1,
				"Distance {} missing/extra values, generated: {:?}",
				distance,
				around
			);
		}
	);

	proptest!(
		#[test]
		fn coord_iterator_should_give_equal_dists(
			coord in rand_coord_strategy(),
			distance in 0..128u8
		) {
			for i in coord.iter_neighbors_ring(distance) {
				prop_assert_eq!(
					coord.distance_to(i),
					distance,
					"other Coord: {:?}",
					i
				);
			}
		}
	);

	proptest!(
		#[test]
		fn coord_to_linear_from_linear(axial in rand_coord_strategy()) {
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

		#[test]
		fn sum_xyz_orientation(coord in rand_coord_orientation_strategy()) {
			prop_assert_eq!(
				coord.x().wrapping_add(
				coord.y().wrapping_add(
				coord.z())),
				0
			);
		}
	);

	proptest!(
		#[test]
		fn wrapping_get_always_returns_when_wrapping(
			coord in rand_coord_strategy(),
			max_x: u8
		) {
			prop_assert_ne!(coord.idx(max_x, 255, true), None);
		}
	);

	proptest!(
		#[test]
		fn six_rights_make_itself(coord in rand_coord_orientation_strategy()) {
			prop_assert_eq!(
				coord.cw().cw().cw().cw().cw().cw(),
				coord
			);
		}
	);

	proptest!(
		#[test]
		fn six_lefts_make_itself(coord in rand_coord_orientation_strategy()) {
			prop_assert_eq!(
				coord.ccw().ccw().ccw().ccw().ccw().ccw(),
				coord
			);
		}
	);

	proptest!(
		#[test]
		fn three_lefts_make_three_rights(coord in rand_coord_orientation_strategy()) {
			prop_assert_eq!(
				coord.ccw().ccw().ccw(),
				coord.cw().cw().cw()
			);
		}
	);

	proptest!(
		#[test]
		fn three_rights_negate(coord in rand_coord_orientation_strategy()) {
			prop_assert_eq!(coord.cw().cw().cw(), -coord);
		}
	);
}
