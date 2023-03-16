//! Double Map Example with remove_prefix
//! `double_map` maps two keys to a single value.
//! the first key might be a team identifier
//! the second key might be a unique identifier
//! `remove_prefix` enables clean removal of all values with the team identifier

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

//#[cfg(test)]
//mod tests;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	//use sp_std::vec::Vec;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Max size for vector
		type MaxSize: Get<u32>;
	}

	pub type TeamIndex = u32; // this is Encode (which is necessary for double_map)

	#[pallet::storage]
	#[pallet::getter(fn member_score)]
	pub type Scores<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		TeamIndex,
		Blake2_128Concat,
		T::AccountId,
		u32,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn teams)]
	pub type Teams<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, TeamIndex, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New member
		NewInMembers(T::AccountId),
		/// Put team score (id, index, score)
		NewInTeams(T::AccountId, TeamIndex, u32),
		/// Remove a single member with AccountId
		RemoveMember(T::AccountId),
		/// Remove all members with TeamId
		RemoveTeam(TeamIndex),
	}
	#[pallet::error]
	pub enum Error<T> {
		AmountZero,
		BalanceOverflow,
		TransferFailed,
		TryPushToBoundedVec,
	}

	#[pallet::storage]
	#[pallet::getter(fn members)]
	pub type Members<T: Config> = StorageValue<_, BoundedVec<T::AccountId, T::MaxSize>, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Join the `Members` vec before joining a team
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn join_members(origin: OriginFor<T>) -> DispatchResult {
			let new_member = ensure_signed(origin)?;
			ensure!(!Self::is_member(&new_member), "already a member");

			let mut bounded_vec = <Members<T>>::get();
			bounded_vec
				.try_push(new_member.clone())
				.map_err(|_| Error::<T>::TryPushToBoundedVec)?;
			<Members<T>>::put(bounded_vec);

			Self::deposit_event(Event::NewInMembers(new_member));
			Ok(())
		}

		/// Put Scores (for testing purposes)
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn join_a_team(origin: OriginFor<T>, index: TeamIndex, score: u32) -> DispatchResult {
			let member = ensure_signed(origin)?;
			ensure!(Self::is_member(&member), "should be a member");

			<Teams<T>>::insert(&member, &index);
			<Scores<T>>::insert(&index, &member, score);

			Self::deposit_event(Event::NewInTeams(member, index, score));
			Ok(())
		}

		/// Remove a member
		#[pallet::call_index(2)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn remove_member(origin: OriginFor<T>) -> DispatchResult {
			let member = ensure_signed(origin)?;
			ensure!(Self::is_member(&member), "should be a member");

			let team_id = <Teams<T>>::take(member.clone());
			<Scores<T>>::remove(&team_id, &member);

			Self::deposit_event(Event::RemoveMember(member));
			Ok(())
		}

		/// Remove score
		#[pallet::call_index(3)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn remove_score(origin: OriginFor<T>, team: TeamIndex) -> DispatchResult {
			let member = ensure_signed(origin)?;

			let team_id = <Teams<T>>::get(member);
			// check that the member is in the team
			ensure!(team_id == team, "team invalid");

			// remove all team members from Scores at once
			//https://paritytech.github.io/substrate/master/frame_support/storage/trait.StorageDoubleMap.html#tymethod.clear_prefix
			let out = <Scores<T>>::clear_prefix(&team_id, 50, None);

			let mut maybe = out.maybe_cursor;

			while maybe.is_some() {
				if let Some(v) = maybe {
					let array: &[u8] = &v[..];

					let out = <Scores<T>>::clear_prefix(&team_id, 50, Some(array));
					maybe = out.maybe_cursor;
				}
			}

			Self::deposit_event(Event::RemoveTeam(team_id));
			Ok(())
		}
	}
}
/*
pub type Scores<T: Config> = StorageDoubleMap<
  _,
  Blake2_128Concat,
  TeamIndex,
  Blake2_128Concat,
  T::AccountId,
  u32,
  ValueQuery,
>;
*/
impl<T: Config> Pallet<T> {
	// for fast membership checks (see check-membership recipe for more details)
	fn is_member(who: &T::AccountId) -> bool {
		Self::members().contains(who)
	}
}
