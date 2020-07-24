#[derive(Clone, Copy, Default, Debug, Hash, PartialOrd, PartialEq, Ord, Eq)]
pub struct Coord {
	pub x: u8,
	pub y: u8,
}

#[derive(Clone, Copy, Default, Debug, Hash, PartialOrd, PartialEq, Ord, Eq)]
pub struct CoordIdx(pub(super) usize);

impl Coord {
	pub fn new(x: u8, y: u8) -> Coord {
		Coord { x, y }
	}

	pub fn idx(&self) -> CoordIdx {
		CoordIdx(((self.x as usize) << 8) + self.y as usize)
	}

	pub fn from_idx(idx: CoordIdx) -> Coord {
		Coord {
			x: (idx.0 >> 8) as u8,
			y: (idx.0 & 0xFF) as u8,
		}
	}

	pub fn iterate_coords_to(&self, to: Coord) -> CoordsRangeIterator {
		CoordsRangeIterator {
			from: self.clone(),
			to,
			current: self.clone(),
			done: false,
		}
	}
}

pub struct CoordsRangeIterator {
	from: Coord,
	to: Coord,
	current: Coord,
	done: bool,
}
impl Iterator for CoordsRangeIterator {
	type Item = Coord;

	fn next(&mut self) -> Option<Self::Item> {
		if self.done {
			return None;
		}
		let ret = self.current;

		if self.current.x == self.to.x {
			if self.current.y == self.to.y {
				self.done = true;
				return Some(ret);
			}
			self.current.x = self.from.x;
			self.current.y = self.current.y.wrapping_add(1);
		} else {
			self.current.x = self.current.x.wrapping_add(1);
		}

		Some(ret)
	}

	// fn size_hint(&self) -> (usize, Option<usize>) {
	// 	let remaining = x * y * z;
	// 	(remaining, Some(remaining))
	// }
}
