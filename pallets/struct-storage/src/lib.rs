#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

//! Struct Storage
//! This pallet demonstrates how to declare and store `structs` that contain types
//! that come from the pallet's configuration trait.

pub use pallet::*;

//#[cfg(test)]
//mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::Hash};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: pallet_balances::Config + frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Max size for vector
		type MaxSize: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[derive(Encode, Decode, Clone, Default, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub struct Car<Hash, Balance> {
		pub number: u32,
		pub hash: Hash,
		pub balance: Balance,
	}
	type CarOf<T> = Car<<T as frame_system::Config>::Hash, <T as pallet_balances::Config>::Balance>;

	// value as a type
	#[pallet::storage]
	#[pallet::getter(fn cars_by_numbers)]
	pub type CarsByNumbers<T> = StorageMap<_, Blake2_128Concat, u32, CarOf<T>, ValueQuery>;

	// Train
	#[derive(Encode, Decode, Default, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub struct Train<Hash, Balance> {
		pub train_num: u32,
		pub car: Car<Hash, Balance>,
	}
	// value as a struct with nested struct
	#[pallet::storage]
	#[pallet::getter(fn trains_by_train_nums)]
	pub type TrainsByNumbers<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, Train<T::Hash, T::Balance>, ValueQuery>;

	// Struct has all types
	#[derive(Encode, Decode, Clone, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)] //Default, Eq
	#[scale_info(skip_type_params(T))]
	pub struct PostComment<T: Config> {
		pub content: BoundedVec<u8, T::MaxSize>,
		pub post_id: T::Hash,
		pub who: T::AccountId,
		pub hash: Hash,
		pub balance: T::Balance,
		pub block_number: BlockNumberFor<T>,
	}
	#[pallet::storage]
	#[pallet::getter(fn post_comments)]
	pub type PostComments<T: Config> =
		StorageMap<_, Twox64Concat, T::Hash, BoundedVec<PostComment<T>, T::MaxSize>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// fields of the new car
		NewCar(u32, T::Hash, T::Balance),
		// fields of the train_num and the car fields
		NewTrainByExistingCar(u32, u32, T::Hash, T::Balance),
		// ""
		NewTrainByNewCar(u32, u32, T::Hash, T::Balance),
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Stores an `Car` struct in the storage map
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn add_car(
			origin: OriginFor<T>,
			number: u32,
			hash: T::Hash,
			balance: T::Balance,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			let car = Car { number, hash, balance };
			<CarsByNumbers<T>>::insert(number, car);
			Self::deposit_event(Event::NewCar(number, hash, balance));
			Ok(().into())
		}

		/// Stores a `Train` struct in the storage map using an `Car` that was already stored
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn add_train_with_existing_car(
			origin: OriginFor<T>,
			car_num: u32,
			train_num: u32,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			let car = Self::cars_by_numbers(car_num);
			let train = Train { train_num, car: car.clone() };
			<TrainsByNumbers<T>>::insert(train_num, train);
			Self::deposit_event(Event::NewTrainByExistingCar(
				train_num,
				car.number,
				car.hash,
				car.balance,
			));
			Ok(().into())
		}

		/// Stores a `Train` struct in the storage map using a new `Car`
		#[pallet::call_index(2)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn add_train_with_new_car(
			origin: OriginFor<T>,
			car_num: u32,
			hash: T::Hash,
			balance: T::Balance,
			train_num: u32,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			// construct and insert `car` first
			let car = Car { number: car_num, hash, balance };
			// overwrites any existing `Car` with `number: car_num` by default
			<CarsByNumbers<T>>::insert(car_num, car.clone());
			Self::deposit_event(Event::NewCar(car_num, hash, balance));
			// now construct and insert `train`
			let train = Train { train_num, car };
			<TrainsByNumbers<T>>::insert(train_num, train);
			Self::deposit_event(Event::NewTrainByNewCar(train_num, car_num, hash, balance));
			Ok(().into())
		}
	}
}
