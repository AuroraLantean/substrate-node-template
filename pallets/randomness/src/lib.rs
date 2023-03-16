//! Generating (insecure) randomness
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::PalletId;
pub use pallet::*;
use sp_std::vec::Vec;
const PALLET_ID: PalletId = PalletId(*b"Staking!");

//#[cfg(test)]
//mod tests;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::DispatchResult, pallet_prelude::*, sp_runtime::app_crypto::sp_core::H256,
		traits::Randomness,
	};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The pallet doesn't know what the source of randomness is; it can be anything that
		/// implements the trait. When installing this pallet in a runtime, you must make sure to give it a randomness source that suits its needs.
		type RandomnessSource: Randomness<H256, Self::BlockNumber>;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// First element is raw seed, second is using nonce.
		RandomnessConsumed(H256, H256), //raw seed and nonce
	}

	#[pallet::error]
	pub enum Error<T> {
		AmountZero,
		InsufficientBalance,
		TransferFailed,
	}

	#[pallet::storage]
	#[pallet::getter(fn nonce)]
	pub type Nonce<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Grab a random seed and random value from the randomness collective flip pallet
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn consume_randomness(origin: OriginFor<T>) -> DispatchResult {
			let caller = ensure_signed(origin)?;

			//This seed changes per block
			let random_seed = T::RandomnessSource::random_seed();

			// subject does not add security or entropy)
			let subject = Self::get_random_subject(caller);
			let random_result = T::RandomnessSource::random(&subject);

			Self::deposit_event(Event::RandomnessConsumed(random_seed.0, random_result.0));
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Reads the nonce from storage, increments the stored nonce, and returns the encoded nonce to the caller.
	fn get_random_subject(caller: T::AccountId) -> Vec<u8> {
		let binding = caller.to_string();
		let caller_u8 = binding.as_bytes();

		let blocknum = <frame_system::Pallet<T>>::block_number();
		let binding = blocknum.to_string();
		let blocknum_u8 = binding.as_bytes();

		let nonce = Nonce::<T>::get();
		let nonce_u8 = nonce.to_be_bytes();
		Nonce::<T>::put(nonce.wrapping_add(1));

		let pallet_id = PALLET_ID.0;
		let sum = [caller_u8, blocknum_u8, &nonce_u8, &pallet_id].concat();
		sum
	}
}
