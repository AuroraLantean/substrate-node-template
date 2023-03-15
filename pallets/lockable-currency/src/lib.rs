//! A pallet to demonstrate the `LockableCurrency` trait
//! borrows collateral locking logic from pallet_staking
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

//#[cfg(test)]
//mod tests;

use frame_support::traits::Currency;
type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{LockIdentifier, LockableCurrency, WithdrawReasons},
	};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	const EXAMPLE_ID: LockIdentifier = *b"example ";

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The lockable currency type
		type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Locked(T::AccountId, BalanceOf<T>),
		ExtendedLock(T::AccountId, BalanceOf<T>),
		Unlocked(T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		AmountZero,
		InsufficientBalance,
		TransferFailed,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Locks the specified amount of tokens from the caller
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn lock_capital(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			let user = ensure_signed(origin)?;

			// If the new lock is valid (i.e. not already expired), it will push the struct to the Locks vec in storage. Note that you can lock more funds than a user has. If the lock id already exists, this will update it.
			T::Currency::set_lock(EXAMPLE_ID, &user, amount, WithdrawReasons::all());

			Self::deposit_event(Event::Locked(user, amount));
			Ok(())
		}

		/// Extends the lock period
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn extend_lock(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			let user = ensure_signed(origin)?;

			/*Changes a balance lock (selected by id) so that it becomes less liquid in all parameters or creates a new one if it does not exist.

			Calling extend_lock on an existing lock id differs from set_lock in that it applies the most severe constraints of the two, while set_lock replaces the lock with the new parameters. As in, extend_lock will set:
			maximum amount, and bitwise mask of all reasons */
			T::Currency::extend_lock(EXAMPLE_ID, &user, amount, WithdrawReasons::all());

			Self::deposit_event(Event::ExtendedLock(user, amount));
			Ok(())
		}

		/// Releases all locked tokens
		#[pallet::call_index(2)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn unlock_all(origin: OriginFor<T>) -> DispatchResult {
			let user = ensure_signed(origin)?;

			T::Currency::remove_lock(EXAMPLE_ID, &user);

			Self::deposit_event(Event::Unlocked(user));
			Ok(())
		}
	}
}
