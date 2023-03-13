//! Pallet that demonstrates a minimal access control check. When a user calls this pallet's
//! only dispatchable function, `check_membership`, the caller is checked against a set of approved
//! callers. If the caller is a member of the set, the pallet's `IsAMember` event is emitted. Otherwise a `NotAMember` error is returned.
//!
//! The list of approved members is provided by the `vec-set` pallet. In order for this pallet to be
//! used, the `vec-set` pallet must also be present in the runtime.

#![allow(clippy::unused_unit)]
pub use pallet::*;

//#[cfg(test)]
//mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	/// The pallet's configuration trait.
	/// Notice the explicit tight coupling to the `vec-set` pallet
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_template::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::event]
	//#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The caller is a member.
		IsAMember{who: T::AccountId},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The caller is not a member
		NotAMember,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
  pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		//------------------==
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn check_membership(origin: OriginFor<T>) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			let members = pallet_template::Pallet::<T>::members();

			// Check whether the caller is a member
			members
				.binary_search(&caller)
				.map_err(|_| Error::<T>::NotAMember)?;

			Self::deposit_event(Event::IsAMember{who: caller});
			Ok(())
		}
	}
}
