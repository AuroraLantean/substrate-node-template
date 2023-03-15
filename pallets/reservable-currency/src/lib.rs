//! A pallet to demonstrate the `ReservableCurrency` trait
//! borrows collateral locking logic from pallet_treasury
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
		traits::{Currency, ExistenceRequirement::AllowDeath, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Currency type for this pallet.
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		LockFunds(T::AccountId, BalanceOf<T>, T::BlockNumber),
		UnlockFunds(T::AccountId, BalanceOf<T>, T::BlockNumber),
		// sender, dest, amount, block number
		TransferFunds(T::AccountId, T::AccountId, BalanceOf<T>, T::BlockNumber),
	}

	#[pallet::error]
	pub enum Error<T> {
		AmountZero,
		InsufficientBalance,
		TransferFailed,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Reserves the specified amount of funds from the caller
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn reserve_funds(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			let caller = ensure_signed(origin)?;

			T::Currency::reserve(&caller, amount).map_err(|_| Error::<T>::InsufficientBalance)?;

			let now = <frame_system::Pallet<T>>::block_number();

			Self::deposit_event(Event::LockFunds(caller, amount, now));
			Ok(())
		}

		/// Unreserves the specified amount of funds from the caller
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn unreserve_funds(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			let caller = ensure_signed(origin)?;

			T::Currency::unreserve(&caller, amount);
			// ReservableCurrency::unreserve does not fail

			let now = <frame_system::Pallet<T>>::block_number();

			Self::deposit_event(Event::UnlockFunds(caller, amount, now));
			Ok(())
		}

		/// Transfers funds. a wrapper around the Currency's own transfer method
		#[pallet::call_index(2)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn transfer_funds(
			origin: OriginFor<T>,
			dest: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			T::Currency::transfer(&sender, &dest, amount, AllowDeath)
				.map_err(|_| Error::<T>::TransferFailed)?;

			let now = <frame_system::Pallet<T>>::block_number();

			Self::deposit_event(Event::TransferFunds(sender, dest, amount, now));
			Ok(())
		}

		/// Atomically unreserves funds and and transfers them.
		/// might be useful in closed economic systems
		#[pallet::call_index(3)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn unreserve_and_transfer(
			origin: OriginFor<T>,
			target: T::AccountId,
			dest: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?; // dangerous because can be called with any signature (so dont do this in practice ever!)

			// If amount > reserved ... extra > 0
			let extra = T::Currency::unreserve(&target, amount);

			T::Currency::transfer(&target, &dest, amount - extra, AllowDeath)
				.map_err(|_| Error::<T>::TransferFailed)?;

			let now = <frame_system::Pallet<T>>::block_number();
			Self::deposit_event(Event::TransferFunds(target, dest, amount - extra, now));

			Ok(())
		}
	}
}
