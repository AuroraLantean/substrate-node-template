#![cfg_attr(not(feature = "std"), no_std)]

//! A Pallet to demonstrate using currency imbalances
//!
//! WARNING: never use this code in production (for demonstration/teaching purposes only)
//! it only checks for signed extrinsics to enable arbitrary minting/slashing!!!
use frame_support::traits::Currency;
pub use pallet::*;

// balance type using reservable currency type
type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type PositiveImbalanceOf<T> = <<T as Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::PositiveImbalance;
type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{Currency, Imbalance, OnUnbalanced, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		// + Sized
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Currency type for this pallet.
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		/// Handler for the unbalanced increment when rewarding (minting rewards)
		type Reward: OnUnbalanced<PositiveImbalanceOf<Self>>;

		/// Handler for the unbalanced decrement when slashing (burning collateral)
		type Slash: OnUnbalanced<NegativeImbalanceOf<Self>>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SlashFunds(T::AccountId, BalanceOf<T>, T::BlockNumber),
		RewardFunds(T::AccountId, BalanceOf<T>, T::BlockNumber),
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Slashes the specified amount of funds from the specified account
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn slash_funds(
			origin: OriginFor<T>,
			target: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			let imbalance = T::Currency::slash_reserved(&target, amount).0;
			T::Slash::on_unbalanced(imbalance);

			let blocknum = <frame_system::Pallet<T>>::block_number();

			Self::deposit_event(Event::SlashFunds(target, amount, blocknum));
			Ok(())
		}

		/// Awards the specified amount of funds to the specified account
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn reward_funds(
			origin: OriginFor<T>,
			target: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			let mut total_imbalance = <PositiveImbalanceOf<T>>::zero();

			let r = T::Currency::deposit_into_existing(&target, amount).ok();
			total_imbalance.maybe_subsume(r);
			T::Reward::on_unbalanced(total_imbalance);

			let blocknum = <frame_system::Pallet<T>>::block_number();

			Self::deposit_event(Event::RewardFunds(target, amount, blocknum));
			Ok(())
		}
	}
}
