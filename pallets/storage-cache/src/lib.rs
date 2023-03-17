//! A pallet that demonstrates caching values from storage in memory
//! Takeaway: minimize calls to runtime storage

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

//#[cfg(test)]
//mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_std::prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Max size for vector
		type MaxSize: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn some_copy_value)]
	pub type ValueX<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn admin)]
	pub type Admin<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn members)]
	pub type Members<T: Config> = StorageValue<_, BoundedVec<T::AccountId, T::MaxSize>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// swap old value with new value (new_value, time_now)
		InefficientValueChange(u32, T::BlockNumber),
		// '' (new_value, time_now)
		BetterValueChange(u32, T::BlockNumber),
		// swap old admin with new admin (old, new)
		InefficientAdminSwap(T::AccountId),
		// '' (old, new)
		BetterAdminSwap(T::AccountId),
	}
	// Errors inform users that usercount went wrong.
	#[pallet::error]
	pub enum Error<T> {
		AlreadyMember,
		NotMember,
		InsertNewMember,
		MembershipLimitReached,
	}
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		///  (Copy) inefficient way of updating value in storage
		///
		/// storage value -> storage_value * 2 + input_val
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn increase_value_no_cache(origin: OriginFor<T>, some_val: u32) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			let old_valuex = <ValueX<T>>::get();
			let some_calculation =
				old_valuex.checked_add(some_val).ok_or("addition overflowed1")?;
			// this next storage call is unnecessary and is wasteful
			let unnecessary_call = <ValueX<T>>::get();
			// should've just used `old_valuex` here because u32 is copy
			let another_calculation =
				some_calculation.checked_add(unnecessary_call).ok_or("addition overflowed2")?;
			<ValueX<T>>::put(another_calculation);

			let now = <frame_system::Pallet<T>>::block_number();
			Self::deposit_event(Event::InefficientValueChange(another_calculation, now));
			Ok(())
		}

		/// (Copy) more efficient value change
		///
		/// storage value -> storage_value * 2 + input_val
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn increase_value_w_copy(origin: OriginFor<T>, some_val: u32) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			let old_valuex = <ValueX<T>>::get();
			let some_calculation =
				old_valuex.checked_add(some_val).ok_or("addition overflowed1")?;
			// uses the old_valuex because u32 is copy
			let another_calculation =
				some_calculation.checked_add(old_valuex).ok_or("addition overflowed2")?;
			<ValueX<T>>::put(another_calculation);

			let now = <frame_system::Pallet<T>>::block_number();
			Self::deposit_event(Event::BetterValueChange(another_calculation, now));
			Ok(())
		}

		///  (Clone) inefficient implementation
		/// swaps the admin account with Origin::signed() if
		/// (1) other account is member &&
		/// (2) existing admin isn't
		#[pallet::call_index(2)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn swap_admin_no_cache(origin: OriginFor<T>) -> DispatchResult {
			let new_admin = ensure_signed(origin)?;
			let existing_admin = <Admin<T>>::get();

			if let Some(accountid) = existing_admin {
				ensure!(!Self::is_member(&accountid), Error::<T>::AlreadyMember);
			}
			// only places a new account if
			// (1) the existing account is not a member &&
			// (2) the new account is a member
			ensure!(
				Self::is_member(&new_admin),
				"new admin is not a member so doesn't get priority"
			);

			// BAD (unnecessary) storage call
			let _old_admin = <Admin<T>>::get();
			// place new admin
			<Admin<T>>::put(new_admin.clone());

			Self::deposit_event(Event::InefficientAdminSwap(new_admin));
			Ok(())
		}

		///  (Clone) better implementation
		/// swaps the admin account with Origin::signed() if
		/// (1) other account is member &&
		/// (2) existing admin isn't
		#[pallet::call_index(3)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn swap_admin_with_cache(origin: OriginFor<T>) -> DispatchResult {
			let new_admin = ensure_signed(origin)?;
			let existing_admin = <Admin<T>>::get();
			// prefer to clone previous value rather than repeat call unnecessarily
			let _old_admin = existing_admin.clone();

			if let Some(accountid) = existing_admin {
				// only places a new account if
				// (1) the existing account is not a member &&
				// (2) the new account is a member
				ensure!(
					!Self::is_member(&accountid),
					"current admin is a member so maintains priority"
				);
			}

			ensure!(
				Self::is_member(&new_admin),
				"new admin is not a member so doesn't get priority"
			);

			// <no (unnecessary) storage call here>
			// place new admin
			<Admin<T>>::put(new_admin.clone());

			Self::deposit_event(Event::BetterAdminSwap(new_admin));
			Ok(())
		}

		// ---- for testing purposes ----
		#[pallet::call_index(4)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn set_value(origin: OriginFor<T>, val: u32) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			<ValueX<T>>::put(val);
			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn set_admin(origin: OriginFor<T>) -> DispatchResult {
			let user = ensure_signed(origin)?;
			<Admin<T>>::put(user);
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn is_member(who: &T::AccountId) -> bool {
		<Members<T>>::get().contains(who)
	}
}
