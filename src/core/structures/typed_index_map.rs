use indexmap::{map::*, *};
use serde::export::PhantomData;
use std::cmp::Ordering;
use std::collections::hash_map::RandomState;
use std::fmt;
use std::hash::{BuildHasher, Hash};
use std::ops::RangeFull;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct TypedIndexMapIndex<T>(usize, PhantomData<T>);

impl<T> TypedIndexMapIndex<T> {
	pub fn index(&self) -> usize {
		self.0
	}
}

pub struct TypedIndexMap<T, K, V, S = RandomState> {
	index_map: IndexMap<K, V, S>,
	_phantom: PhantomData<T>,
}

impl<T, K: Clone, V: Clone, S: Clone> Clone for TypedIndexMap<T, K, V, S> {
	#[inline]
	fn clone(&self) -> Self {
		TypedIndexMap {
			index_map: Clone::clone(&self.index_map),
			_phantom: Default::default(),
		}
	}

	#[inline]
	fn clone_from(&mut self, source: &Self) {
		Clone::clone_from(&mut self.index_map, &source.index_map)
	}
}

impl<T, K, V, S> fmt::Debug for TypedIndexMap<T, K, V, S>
where
	K: fmt::Debug + Hash + Eq,
	V: fmt::Debug,
	S: BuildHasher,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(&self.index_map, f)
	}
}

impl<T, K, V> TypedIndexMap<T, K, V> {
	/// Create a new map. (Does not allocate.)
	#[inline]
	pub fn new() -> Self {
		Self::with_capacity(0)
	}

	/// Create a new map with capacity for `n` key-value pairs. (Does not
	/// allocate if `n` is zero.)
	///
	/// Computes in **O(n)** time.
	#[inline]
	pub fn with_capacity(n: usize) -> Self {
		Self::with_capacity_and_hasher(n, <_>::default())
	}
}

impl<T, K, V, S> TypedIndexMap<T, K, V, S> {
	/// Create a new map with capacity for `n` key-value pairs. (Does not
	/// allocate if `n` is zero.)
	///
	/// Computes in **O(n)** time.
	#[inline]
	pub fn with_capacity_and_hasher(n: usize, hash_builder: S) -> Self
	where
		S: BuildHasher,
	{
		TypedIndexMap {
			index_map: IndexMap::with_capacity_and_hasher(n, hash_builder),
			_phantom: Default::default(),
		}
	}

	/// Return the number of key-value pairs in the map.
	///
	/// Computes in **O(1)** time.
	#[inline]
	pub fn len(&self) -> usize {
		self.index_map.len()
	}

	/// Returns true if the map contains no elements.
	///
	/// Computes in **O(1)** time.
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Create a new map with `hash_builder`
	#[inline]
	pub fn with_hasher(hash_builder: S) -> Self
	where
		S: BuildHasher,
	{
		Self::with_capacity_and_hasher(0, hash_builder)
	}

	/// Return a reference to the map's `BuildHasher`.
	#[inline]
	pub fn hasher(&self) -> &S
	where
		S: BuildHasher,
	{
		&self.index_map.hasher()
	}

	/// Computes in **O(1)** time.
	#[inline]
	pub fn capacity(&self) -> usize {
		self.index_map.capacity()
	}
}

impl<T, K, V, S> TypedIndexMap<T, K, V, S>
where
	K: Hash + Eq,
	S: BuildHasher,
{
	/// Remove all key-value pairs in the map, while preserving its capacity.
	///
	/// Computes in **O(n)** time.
	#[inline]
	pub fn clear(&mut self) {
		self.index_map.clear();
	}

	/// Reserve capacity for `additional` more key-value pairs.
	///
	/// Computes in **O(n)** time.
	#[inline]
	pub fn reserve(&mut self, additional: usize) {
		self.index_map.reserve(additional);
	}

	/// Shrink the capacity of the map as much as possible.
	///
	/// Computes in **O(n)** time.
	#[inline]
	pub fn shrink_to_fit(&mut self) {
		self.index_map.shrink_to_fit();
	}

	/// Insert a key-value pair in the map.
	///
	/// If an equivalent key already exists in the map: the key remains and
	/// retains in its place in the order, its corresponding value is updated
	/// with `value` and the older value is returned inside `Some(_)`.
	///
	/// If no equivalent key existed in the map: the new key-value pair is
	/// inserted, last in order, and `None` is returned.
	///
	/// Computes in **O(1)** time (amortized average).
	///
	/// See also [`entry`](#method.entry) if you you want to insert *or* modify
	/// or if you need to get the index of the corresponding key-value pair.
	#[inline]
	pub fn insert(&mut self, key: K, value: V) -> Option<V> {
		self.insert_full(key, value).1
	}

	/// Insert a key-value pair in the map, and get their index.
	///
	/// If an equivalent key already exists in the map: the key remains and
	/// retains in its place in the order, its corresponding value is updated
	/// with `value` and the older value is returned inside `(index, Some(_))`.
	///
	/// If no equivalent key existed in the map: the new key-value pair is
	/// inserted, last in order, and `(index, None)` is returned.
	///
	/// Computes in **O(1)** time (amortized average).
	///
	/// See also [`entry`](#method.entry) if you you want to insert *or* modify
	/// or if you need to get the index of the corresponding key-value pair.
	#[inline]
	pub fn insert_full(&mut self, key: K, value: V) -> (TypedIndexMapIndex<T>, Option<V>) {
		let (idx, res) = self.index_map.insert_full(key, value);
		(TypedIndexMapIndex(idx, Default::default()), res)
	}

	/// Get the given key’s corresponding entry in the map for insertion and/or
	/// in-place manipulation.
	///
	/// Computes in **O(1)** time (amortized average).
	#[inline]
	pub fn entry(&mut self, key: K) -> Entry<K, V> {
		self.index_map.entry(key)
	}

	/// Return an iterator over the key-value pairs of the map, in their order
	#[inline]
	pub fn iter(&self) -> Iter<K, V> {
		self.index_map.iter()
	}

	/// Return an iterator over the key-value pairs of the map, in their order
	#[inline]
	pub fn iter_mut(&mut self) -> IterMut<K, V> {
		self.index_map.iter_mut()
	}

	/// Return an iterator over the keys of the map, in their order
	#[inline]
	pub fn keys(&self) -> Keys<K, V> {
		self.index_map.keys()
	}

	/// Return an iterator over the values of the map, in their order
	#[inline]
	pub fn values(&self) -> Values<K, V> {
		self.index_map.values()
	}

	/// Return an iterator over mutable references to the the values of the map,
	/// in their order
	#[inline]
	pub fn values_mut(&mut self) -> ValuesMut<K, V> {
		self.index_map.values_mut()
	}

	/// Return `true` if an equivalent to `key` exists in the map.
	///
	/// Computes in **O(1)** time (average).
	#[inline]
	pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
	where
		Q: Hash + Equivalent<K>,
	{
		self.index_map.contains_key(key)
	}

	/// Return a reference to the value stored for `key`, if it is present,
	/// else `None`.
	///
	/// Computes in **O(1)** time (average).
	#[inline]
	pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
	where
		Q: Hash + Equivalent<K>,
	{
		self.index_map.get(key)
	}

	/// Return references to the key-value pair stored for `key`,
	/// if it is present, else `None`.
	///
	/// Computes in **O(1)** time (average).
	#[inline]
	pub fn get_key_value<Q: ?Sized>(&self, key: &Q) -> Option<(&K, &V)>
	where
		Q: Hash + Equivalent<K>,
	{
		self.index_map.get_key_value(key)
	}

	/// Return item index, key and value
	#[inline]
	pub fn get_full<Q: ?Sized>(&self, key: &Q) -> Option<(TypedIndexMapIndex<T>, &K, &V)>
	where
		Q: Hash + Equivalent<K>,
	{
		self.index_map
			.get_full(key)
			.map(|(idx, k, v)| (TypedIndexMapIndex(idx, Default::default()), k, v))
	}

	/// Return item index, if it exists in the map
	#[inline]
	pub fn get_index_of<Q: ?Sized>(&self, key: &Q) -> Option<TypedIndexMapIndex<T>>
	where
		Q: Hash + Equivalent<K>,
	{
		self.index_map
			.get_index_of(key)
			.map(|idx| TypedIndexMapIndex(idx, Default::default()))
	}

	#[inline]
	pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
	where
		Q: Hash + Equivalent<K>,
	{
		self.index_map.get_mut(key)
	}

	#[inline]
	pub fn get_full_mut<Q: ?Sized>(
		&mut self,
		key: &Q,
	) -> Option<(TypedIndexMapIndex<T>, &K, &mut V)>
	where
		Q: Hash + Equivalent<K>,
	{
		self.index_map
			.get_full_mut(key)
			.map(|(idx, k, v)| (TypedIndexMapIndex(idx, Default::default()), k, v))
	}

	/// Remove the key-value pair equivalent to `key` and return
	/// its value.
	///
	/// **NOTE:** This is equivalent to `.swap_remove(key)`, if you need to
	/// preserve the order of the keys in the map, use `.shift_remove(key)`
	/// instead.
	///
	/// Computes in **O(1)** time (average).
	#[inline]
	pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
	where
		Q: Hash + Equivalent<K>,
	{
		self.swap_remove(key)
	}

	/// Remove and return the key-value pair equivalent to `key`.
	///
	/// **NOTE:** This is equivalent to `.swap_remove_entry(key)`, if you need to
	/// preserve the order of the keys in the map, use `.shift_remove_entry(key)`
	/// instead.
	///
	/// Computes in **O(1)** time (average).
	#[inline]
	pub fn remove_entry<Q: ?Sized>(&mut self, key: &Q) -> Option<(K, V)>
	where
		Q: Hash + Equivalent<K>,
	{
		self.swap_remove_entry(key)
	}

	/// Remove the key-value pair equivalent to `key` and return
	/// its value.
	///
	/// Like `Vec::swap_remove`, the pair is removed by swapping it with the
	/// last element of the map and popping it off. **This perturbs
	/// the postion of what used to be the last element!**
	///
	/// Return `None` if `key` is not in map.
	///
	/// Computes in **O(1)** time (average).
	#[inline]
	pub fn swap_remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
	where
		Q: Hash + Equivalent<K>,
	{
		self.index_map.swap_remove(key)
	}

	/// Remove and return the key-value pair equivalent to `key`.
	///
	/// Like `Vec::swap_remove`, the pair is removed by swapping it with the
	/// last element of the map and popping it off. **This perturbs
	/// the postion of what used to be the last element!**
	///
	/// Return `None` if `key` is not in map.
	///
	/// Computes in **O(1)** time (average).
	#[inline]
	pub fn swap_remove_entry<Q: ?Sized>(&mut self, key: &Q) -> Option<(K, V)>
	where
		Q: Hash + Equivalent<K>,
	{
		self.index_map.swap_remove_entry(key)
	}

	/// Remove the key-value pair equivalent to `key` and return it and
	/// the index it had.
	///
	/// Like `Vec::swap_remove`, the pair is removed by swapping it with the
	/// last element of the map and popping it off. **This perturbs
	/// the postion of what used to be the last element!**
	///
	/// Return `None` if `key` is not in map.
	///
	/// Computes in **O(1)** time (average).
	#[inline]
	pub fn swap_remove_full<Q: ?Sized>(&mut self, key: &Q) -> Option<(TypedIndexMapIndex<T>, K, V)>
	where
		Q: Hash + Equivalent<K>,
	{
		self.index_map
			.swap_remove_full(key)
			.map(|(idx, k, v)| (TypedIndexMapIndex(idx, Default::default()), k, v))
	}

	/// Remove the key-value pair equivalent to `key` and return
	/// its value.
	///
	/// Like `Vec::remove`, the pair is removed by shifting all of the
	/// elements that follow it, preserving their relative order.
	/// **This perturbs the index of all of those elements!**
	///
	/// Return `None` if `key` is not in map.
	///
	/// Computes in **O(n)** time (average).
	#[inline]
	pub fn shift_remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
	where
		Q: Hash + Equivalent<K>,
	{
		self.index_map.shift_remove(key)
	}

	/// Remove and return the key-value pair equivalent to `key`.
	///
	/// Like `Vec::remove`, the pair is removed by shifting all of the
	/// elements that follow it, preserving their relative order.
	/// **This perturbs the index of all of those elements!**
	///
	/// Return `None` if `key` is not in map.
	///
	/// Computes in **O(n)** time (average).
	#[inline]
	pub fn shift_remove_entry<Q: ?Sized>(&mut self, key: &Q) -> Option<(K, V)>
	where
		Q: Hash + Equivalent<K>,
	{
		self.index_map.shift_remove_entry(key)
	}

	/// Remove the key-value pair equivalent to `key` and return it and
	/// the index it had.
	///
	/// Like `Vec::remove`, the pair is removed by shifting all of the
	/// elements that follow it, preserving their relative order.
	/// **This perturbs the index of all of those elements!**
	///
	/// Return `None` if `key` is not in map.
	///
	/// Computes in **O(n)** time (average).
	#[inline]
	pub fn shift_remove_full<Q: ?Sized>(&mut self, key: &Q) -> Option<(TypedIndexMapIndex<T>, K, V)>
	where
		Q: Hash + Equivalent<K>,
	{
		self.index_map
			.shift_remove_full(key)
			.map(|(idx, k, v)| (TypedIndexMapIndex(idx, Default::default()), k, v))
	}

	/// Remove the last key-value pair
	///
	/// Computes in **O(1)** time (average).
	#[inline]
	pub fn pop(&mut self) -> Option<(K, V)> {
		self.index_map.pop()
	}

	/// Scan through each key-value pair in the map and keep those where the
	/// closure `keep` returns `true`.
	///
	/// The elements are visited in order, and remaining elements keep their
	/// order.
	///
	/// Computes in **O(n)** time (average).
	#[inline]
	pub fn retain<F>(&mut self, keep: F)
	where
		F: FnMut(&K, &mut V) -> bool,
	{
		self.index_map.retain(keep)
	}

	/// Sort the map’s key-value pairs by the default ordering of the keys.
	///
	/// See `sort_by` for details.
	#[inline]
	pub fn sort_keys(&mut self)
	where
		K: Ord,
	{
		self.index_map.sort_keys()
	}

	/// Sort the map’s key-value pairs in place using the comparison
	/// function `compare`.
	///
	/// The comparison function receives two key and value pairs to compare (you
	/// can sort by keys or values or their combination as needed).
	///
	/// Computes in **O(n log n + c)** time and **O(n)** space where *n* is
	/// the length of the map and *c* the capacity. The sort is stable.
	#[inline]
	pub fn sort_by<F>(&mut self, cmp: F)
	where
		F: FnMut(&K, &V, &K, &V) -> Ordering,
	{
		self.index_map.sort_by(cmp)
	}

	/// Sort the key-value pairs of the map and return a by value iterator of
	/// the key-value pairs with the result.
	///
	/// The sort is stable.
	#[inline]
	pub fn sorted_by<F>(self, cmp: F) -> IntoIter<K, V>
	where
		F: FnMut(&K, &V, &K, &V) -> Ordering,
	{
		self.index_map.sorted_by(cmp)
	}

	/// Reverses the order of the map’s key-value pairs in place.
	///
	/// Computes in **O(n)** time and **O(1)** space.
	#[inline]
	pub fn reverse(&mut self) {
		self.index_map.reverse()
	}

	/// Clears the `IndexMap`, returning all key-value pairs as a drain iterator.
	/// Keeps the allocated memory for reuse.
	#[inline]
	pub fn drain(&mut self, range: RangeFull) -> Drain<K, V> {
		self.index_map.drain(range)
	}
}

impl<T, K, V, S> TypedIndexMap<T, K, V, S> {
	/// Get a key-value pair by index
	///
	/// Valid indices are *0 <= index < self.len()*
	///
	/// Computes in **O(1)** time.
	#[inline]
	pub fn get_index(&self, index: TypedIndexMapIndex<T>) -> Option<(&K, &V)> {
		self.index_map.get_index(index.0)
	}

	/// Get a key-value pair by index
	///
	/// Valid indices are *0 <= index < self.len()*
	///
	/// Computes in **O(1)** time.
	#[inline]
	pub fn get_index_mut(&mut self, index: TypedIndexMapIndex<T>) -> Option<(&mut K, &mut V)> {
		self.index_map.get_index_mut(index.0)
	}

	/// Remove the key-value pair by index
	///
	/// Valid indices are *0 <= index < self.len()*
	///
	/// Like `Vec::swap_remove`, the pair is removed by swapping it with the
	/// last element of the map and popping it off. **This perturbs
	/// the postion of what used to be the last element!**
	///
	/// Computes in **O(1)** time (average).
	#[inline]
	pub fn swap_remove_index(&mut self, index: TypedIndexMapIndex<T>) -> Option<(K, V)> {
		self.index_map.swap_remove_index(index.0)
	}

	/// Remove the key-value pair by index
	///
	/// Valid indices are *0 <= index < self.len()*
	///
	/// Like `Vec::remove`, the pair is removed by shifting all of the
	/// elements that follow it, preserving their relative order.
	/// **This perturbs the index of all of those elements!**
	///
	/// Computes in **O(n)** time (average).
	#[inline]
	pub fn shift_remove_index(&mut self, index: TypedIndexMapIndex<T>) -> Option<(K, V)> {
		self.index_map.shift_remove_index(index.0)
	}
}
