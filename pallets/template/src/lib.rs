#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;
use pallet_timestamp::{self as timestamp};
//use sp_std::collections::btree_set::BTreeSet;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::{traits::Currency, PalletId};

/// To generate pallet controlled account.
const PALLET_ID: PalletId = PalletId(*b"Staking!");
//use sp_runtime::traits::{StaticLookup, Zero};
//use sp_std::prelude::*;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type _PositiveImbalanceOf<T> = <<T as Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::PositiveImbalance;
type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;
/*const LOG_TARGET: &str = "runtime::uniques";
type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;
*/
#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::DispatchResult,
		inherent::Vec,
		log::{info, warn},
		pallet_prelude::*,
		sp_runtime::traits::AccountIdConversion,
		//sp_runtime::SaturatedConversion,
		traits::{Currency, ExistenceRequirement::AllowDeath, Hash, Imbalance, OnUnbalanced},
	};
	use frame_system::pallet_prelude::*;
	use sp_std::prelude::*;

	/// A maximum number of members. When membership reaches this number, no new members may join.
	pub const MAX_MEMBERS: usize = 16;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);
	//pub struct Pallet<T, I = ()>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + timestamp::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		#[pallet::constant]
		type StringMax: Get<u8>;
		/// The currency type this pallet deals with
		type Currency: Currency<Self::AccountId>;

		/// admin account
		type ForceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// Max size for vector
		type MaxSize: Get<u32>;
	}

	#[derive(Encode, Decode, Clone, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)] //Default, Eq
	#[scale_info(skip_type_params(T))]
	pub struct PostComment<T: Config> {
		pub content: BoundedVec<u8, T::MaxSize>,
		pub post_id: T::Hash,
		pub who: T::AccountId,
		pub hash: Hash,
		pub balance: BalanceOf<T>,
		pub block_number: BlockNumberFor<T>,
	} //<T as frame_system::Config>
	#[pallet::storage]
	#[pallet::getter(fn blog_post_comments)]
	pub(super) type PostComments<T: Config> =
		StorageMap<_, Twox64Concat, T::Hash, BoundedVec<PostComment<T>, T::MaxSize>>;

	#[derive(Default, Encode, Decode, Clone, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
	//#[scale_info(skip_type_params(T))]
	//#[codec(mel_bound())]
	pub struct UserInfo {
		pub id: u32,
		pub username: [u8; 20],
		//BoundedVec<u8, T::StringMax>,
		pub staked: u64,
		pub staked_at: u64,
		pub unstaked: u64,
		pub reward: u64,
	}

	//#[pallet::unbounded]
	#[pallet::storage]
	#[pallet::getter(fn userstakes)]
	/// Info on all of the info.
	pub type UserStakes<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, UserInfo, OptionQuery>; //<T>

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn usercount)] //(super)...Self::some_value()
	pub type UserCount<T> = StorageValue<_, u32, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn totalsupply)]
	pub type TotalSupply<T> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn rewardrate)]
	pub type RewardRate<T> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn init)]
	pub type IsInitial<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn balances)]
	pub(super) type Balances<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn members)]
	pub(super) type Members<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxSize>, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		//call on each block execution
	}

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		ResetUserCount {
			usercount: u32,
			who: T::AccountId,
		},
		UserAdded {
			user_index: u32, //from 1
			user: T::AccountId,
		}, //deposit: BalanceOf<T>, T::BlockNumber
		/// Tokens was initialized by user
		Initialized {
			initializer: T::AccountId,
			amount: u64,
		},
		/// Tokens successfully transferred between users
		TransferSucceeded {
			from: T::AccountId,
			to: T::AccountId,
			amount: u64,
		},
		/// Staking
		Staking {
			from: T::AccountId,
			amount: u64,
			pallet_balance: BalanceOf<T>,
			timestamp: u64,
		},
		Unstake {
			from: T::AccountId,
			amount: u64,
			timestamp: u64,
			duration: u64,
			reward_rate: u64,
			reward: u64,
		},
		Withdraw {
			from: T::AccountId,
			amount: u64,
			pallet_balance: BalanceOf<T>,
		},
		/// An imbalance from elsewhere in the runtime has been absorbed by the pallet
		ImbalanceAbsorbed {
			from_balance: BalanceOf<T>,
			to_balance: BalanceOf<T>,
		},
		NowTime(u64, T::BlockNumber),
		SetRewardRate(u64),
		/// Added a member
		MemberAdded(T::AccountId),
		AlreadyMember {},
		/// Removed a member
		MemberRemoved(T::AccountId),
	}

	// Errors inform users that usercount went wrong.
	#[pallet::error]
	pub enum Error<T> {
		AmountZero,
		BalanceOverflow,
		TransferFailed,
		RewardRateZero,
		NoneValue,
		VecToArray,
		StringTooShort,
		StringTooLong,
		InitializedBf,
		UserCountOverflow,
		StakedOverflow,
		InsufficientStaked,
		InsufficientBalance,
		UserDoesNotExist,
		ConvertU64ToBalance,
		InsufficientTokens,
		TokenOverflow,
		ConvertNowToU64,
		ShouldStakeFirst,
		MultiplyOverflow,
		RewardOverflow,
		InsufficientReward,
		InsufficientUnstaked,
		AlreadyMember,
		NotMember,
		InsertNewMember,
		MembershipLimitReached,
	}

	impl<T: Config> Pallet<T> {
		/// The account ID that holds funds
		pub fn account_id() -> T::AccountId {
			PALLET_ID.into_account_truncating()
			//T::PalletId::get().into_account_truncating()
		}

		/// The pallet balance
		pub fn pot() -> BalanceOf<T> {
			T::Currency::free_balance(&Self::account_id())
		}
	}

	// to receive NegativeImbalances, which are generated when a validator is slashed for violating consensus rules, transaction fees are collected, or token burnt ... while positive imbalance is generated when tokens are minted
	impl<T: Config> OnUnbalanced<NegativeImbalanceOf<T>> for Pallet<T> {
		fn on_nonzero_unbalanced(amount: NegativeImbalanceOf<T>) {
			let numeric_amount = amount.peek();

			// Must resolve into existing but better to be safe
			let _ = T::Currency::resolve_creating(&Self::account_id(), amount);

			Self::deposit_event(Event::ImbalanceAbsorbed {
				from_balance: numeric_amount,
				to_balance: Self::pot(),
			});
		}
	}
	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		//------------------==
		#[pallet::call_index(10)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn add_member_into_vec(origin: OriginFor<T>) -> DispatchResult {
			let new_member = ensure_signed(origin)?;
			let mut members = Members::<T>::get();
			ensure!(members.len() < T::MaxSize::get() as usize, Error::<T>::MembershipLimitReached);
			match members.binary_search(&new_member) {
				Ok(_) => Err(Error::<T>::AlreadyMember.into()),
				Err(index) => {
					let _out = members
						.try_insert(index, new_member.clone())
						.map_err(|_| Error::<T>::InsertNewMember);
					Members::<T>::put(members);
					Self::deposit_event(Event::MemberAdded(new_member));
					Ok(())
				},
			}
		}
		//------------------==
		#[pallet::call_index(11)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn remove_member_from_vec(origin: OriginFor<T>) -> DispatchResult {
			let old_member = ensure_signed(origin)?;
			let mut members = Members::<T>::get();

			match members.binary_search(&old_member) {
				Ok(index) => {
					members.remove(index);
					Members::<T>::put(members);
					Self::deposit_event(Event::MemberRemoved(old_member));
					Ok(())
				},
				Err(_) => Err(Error::<T>::NotMember.into()),
			}
			// also see `append_or_insert`, `append_or_put` in pallet-elections/phragmen, democracy
		}
		//------------------==
		#[pallet::call_index(9)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn set_reward_rate(origin: OriginFor<T>, reward_rate_new: u64) -> DispatchResult {
			T::ForceOrigin::ensure_origin(origin)?;
			//let caller = ensure_signed(origin)?;
			info!("reward_rate_new: {:?}", reward_rate_new);

			let reward_rate_old = <RewardRate<T>>::get();
			info!("reward_rate_old: {:?}", reward_rate_old);
			<RewardRate<T>>::put(reward_rate_new);

			Self::deposit_event(Event::SetRewardRate(reward_rate_new));
			Ok(())
		}
		//------------------==
		#[pallet::call_index(8)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn get_time(origin: OriginFor<T>) -> DispatchResult {
			let _sender = ensure_signed(origin)?;
			let now = <timestamp::Pallet<T>>::get();
			let now_timestamp = now.try_into().map_err(|_| Error::<T>::ConvertNowToU64)?;
			info!("info now_timestamp: {:?}", now_timestamp);
			warn!("warn now_timestamp: {:?}", now_timestamp);
			let blocknum = <frame_system::Pallet<T>>::block_number();
			Self::deposit_event(Event::<T>::NowTime(now_timestamp, blocknum));
			Ok(())
		}
		//------------------==
		#[pallet::call_index(5)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn stake(origin: OriginFor<T>, amount: u64) -> DispatchResult {
			ensure!(amount > 0, Error::<T>::AmountZero);
			let from = ensure_signed(origin)?;
			info!("from: {:?}, amount:{:?}", from, amount);
			info!("pallet_balance: {:?}", Self::pot());
			let now = <timestamp::Pallet<T>>::get();
			let now_timestamp = now.try_into().map_err(|_| Error::<T>::ConvertNowToU64)?;
			info!("now: {:?}, now_timestamp: {:?}", now, now_timestamp);
			//let amt_bal: BalanceOf<T> = amount_u32.into();
			let amt_bal: BalanceOf<T> =
				amount.try_into().map_err(|_| Error::<T>::ConvertU64ToBalance)?;
			//let amt_bal: BalanceOf<T> = amount_u128.saturated_into::<BalanceOf<T>>();

			//Operation may result in account going out of existence
			T::Currency::transfer(&from, &Self::account_id(), amt_bal, AllowDeath)
				.map_err(|_| Error::<T>::TransferFailed)?;

			let mut user =
				<UserStakes<T>>::get(&from).ok_or_else(|| Error::<T>::UserDoesNotExist)?;
			info!("user: {:?}", user);

			let staked_new =
				user.staked.checked_add(amount).ok_or_else(|| Error::<T>::StakedOverflow)?;
			info!("staked_new: {:?}", staked_new);

			user.staked = staked_new;
			user.staked_at = now_timestamp;
			info!("user updated: {:?}", user);
			info!("pallet_balance: {:?}", Self::pot());

			<UserStakes<T>>::insert(&from, user);

			let tok_bal = <Balances<T>>::get(&from);
			let new_tok_bal =
				tok_bal.checked_add(amount).ok_or_else(|| Error::<T>::TokenOverflow)?;
			info!("old tok_bal: {:?}, new_tok_bal:{:?}", tok_bal, new_tok_bal);
			<Balances<T>>::insert(&from, new_tok_bal);

			Self::deposit_event(Event::<T>::Staking {
				from,
				amount,
				pallet_balance: Self::pot(),
				timestamp: now_timestamp,
			});
			Ok(())
		}
		//------------------==
		#[pallet::call_index(7)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn withdraw(origin: OriginFor<T>, amount: u64) -> DispatchResult {
			ensure!(amount > 0, Error::<T>::AmountZero);
			//T::ForceOrigin::ensure_origin(origin)?;
			let from = ensure_signed(origin)?;
			info!("from: {:?}, amount:{:?}", from, amount);
			info!("pallet_balance: {:?}", Self::pot());

			let mut user =
				<UserStakes<T>>::get(&from).ok_or_else(|| Error::<T>::UserDoesNotExist)?;
			info!("user: {:?}", user);

			let unstaked_new = user
				.unstaked
				.checked_sub(amount)
				.ok_or_else(|| Error::<T>::InsufficientUnstaked)?;
			info!("unstaked_new: {:?}", unstaked_new);
			user.unstaked = unstaked_new;
			info!("user: {:?}", user);
			<UserStakes<T>>::insert(&from, user);

			let amt_bal: BalanceOf<T> =
				amount.try_into().map_err(|_| Error::<T>::ConvertU64ToBalance)?;
			//Operation may result in account going out of existence
			T::Currency::transfer(&Self::account_id(), &from, amt_bal, AllowDeath)
				.map_err(|_| Error::<T>::TransferFailed)?;
			info!("pallet_balance: {:?}", Self::pot());

			let tok_bal = <Balances<T>>::get(&from);
			let new_tok_bal =
				tok_bal.checked_sub(amount).ok_or_else(|| Error::<T>::InsufficientTokens)?;
			info!("old tok_bal: {:?}, new_tok_bal:{:?}", tok_bal, new_tok_bal);
			<Balances<T>>::insert(&from, new_tok_bal);

			Self::deposit_event(Event::<T>::Withdraw { from, amount, pallet_balance: Self::pot() });
			Ok(())
		}
		//------------------==
		#[pallet::call_index(6)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn unstake(origin: OriginFor<T>, amount: u64) -> DispatchResult {
			ensure!(amount > 0, Error::<T>::AmountZero);
			let from = ensure_signed(origin)?;
			info!("from: {:?}, amount:{:?}", from, amount);
			let now = <timestamp::Pallet<T>>::get();
			let now_timestamp: u64 = now.try_into().map_err(|_| Error::<T>::ConvertNowToU64)?;
			info!("now: {:?}, now_timestamp: {:?}", now, now_timestamp);

			let mut user =
				<UserStakes<T>>::get(&from).ok_or_else(|| Error::<T>::UserDoesNotExist)?;
			info!("user: {:?}", user);

			let duration = now_timestamp
				.checked_sub(user.staked_at)
				.ok_or_else(|| Error::<T>::ShouldStakeFirst)?;
			info!("duration: {:?}", duration);

			let reward_rate = <RewardRate<T>>::get();
			info!("reward_rate: {:?}", reward_rate);
			info!("Did you set reward to none zero?");
			ensure!(reward_rate > 0, Error::<T>::RewardRateZero);
			let yyy =
				amount.checked_mul(reward_rate).ok_or_else(|| Error::<T>::MultiplyOverflow)?;
			info!("amount * reward_rate: {:?}", yyy);
			let zzz = yyy.checked_mul(duration).ok_or_else(|| Error::<T>::MultiplyOverflow)?;
			info!("* duration: {:?}", zzz);
			let reward = zzz.checked_div(1000000).ok_or_else(|| Error::<T>::MultiplyOverflow)?;
			info!("reward: {:?}", reward);

			let staked_new =
				user.staked.checked_sub(amount).ok_or_else(|| Error::<T>::InsufficientStaked)?;
			info!("staked_new: {:?}", staked_new);

			let reward_new =
				user.reward.checked_add(reward).ok_or_else(|| Error::<T>::RewardOverflow)?;
			info!("reward_new: {:?}", reward_new);

			user.staked = staked_new;
			user.unstaked = amount;
			user.reward = reward_new;
			info!("user updated: {:?}", user);
			<UserStakes<T>>::insert(&from, user);

			Self::deposit_event(Event::<T>::Unstake {
				from,
				amount,
				timestamp: now_timestamp,
				duration,
				reward_rate,
				reward,
			});

			Ok(())
		}
		//------------------==
		#[pallet::call_index(2)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn add_user(
			origin: OriginFor<T>,
			user_id: u32,
			username: Vec<u8>,
			//[u8; 20], Who is da Jonny Dipp => HexToText without 0x
			//BoundedVec<u8, T::StringMax>,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;
			info!("from: {:?}", from);
			info!("username: {:?}", username);
			let explen = T::StringMax::get() as usize;
			let ulen = username.len();
			info!("username len:{:?}, expected len:{:?}", ulen, explen);
			ensure!(ulen <= explen, Error::<T>::StringTooLong);

			let mut username2 = username;
			username2.resize(explen, 32); //ASCII code table
			info!("username2 len: {:?}", username2.len());

			let arr = username2.try_into().map_err(|_| Error::<T>::VecToArray)?;

			let user_count = match <UserCount<T>>::get() {
				None => 1,
				//return Err(Error::<T>::NoneValue),
				Some(old) => old.checked_add(1).ok_or_else(|| Error::<T>::UserCountOverflow)?,
			};
			info!("user_count: {:?}", user_count);
			let user = UserInfo {
				id: user_id,
				username: arr,
				staked: 0u64,
				staked_at: 0u64,
				unstaked: 0u64,
				reward: 0u64,
			};
			ensure!(!<UserStakes<T>>::contains_key(&from), Error::<T>::AlreadyMember);
			<UserStakes<T>>::insert(&from, user);
			<UserCount<T>>::put(user_count); //update count after insert
			Self::deposit_event(Event::UserAdded { user_index: user_id, user: from });
			Ok(())
		}
		/*
		   fn remove_user(origin) -> DispatchResult {
			 let old_user = ensure_signed(origin)?;
			 ensure!(<UserStakes<T>>::contains_key(&old_user), Error::<T>::NotUser);

			 <UserStakes<T>>::remove(&old_user);
			 UserCount::mutate(|v| *v -= 1);
			 Self::deposit_event(Event::UserRemoved(old_user));
			 Ok(())
		   }
		*/
		//------------------==
		#[pallet::call_index(3)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn initialize_token(origin: OriginFor<T>, totalsupply: u64) -> DispatchResult {
			let from = ensure_signed(origin)?;
			ensure!(!<IsInitial<T>>::get(), Error::<T>::InitializedBf);
			<TotalSupply<T>>::put(totalsupply);
			info!("totalsupply: {:?}", totalsupply);
			<Balances<T>>::insert(&from, totalsupply);

			IsInitial::<T>::put(true);
			Self::deposit_event(Event::Initialized { initializer: from, amount: totalsupply });
			Ok(())
		}

		//------------------==
		#[pallet::call_index(4)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn transfer(origin: OriginFor<T>, to: T::AccountId, amount: u64) -> DispatchResult {
			let from = ensure_signed(origin)?;
			let from_balance = <Balances<T>>::get(&from);
			let to_balance = <Balances<T>>::get(&to);

			let new_from_balance = from_balance
				.checked_sub(amount)
				.ok_or_else(|| Error::<T>::InsufficientBalance)?;
			//return Err(Error::<T>::NoneValue.into()),

			let new_to_balance =
				to_balance.checked_add(amount).ok_or_else(|| Error::<T>::BalanceOverflow)?;

			<Balances<T>>::insert(&from, new_from_balance);
			<Balances<T>>::insert(&to, new_to_balance);

			Self::deposit_event(Event::TransferSucceeded { from, to, amount });
			Ok(())
		}

		//------------------==
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn reset_usercount(origin: OriginFor<T>, usercount: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;
			info!("info usercount: {:?}", usercount);
			warn!("warn usercount: {:?}", usercount);

			// Update storage.
			<UserCount<T>>::put(usercount);

			// Emit an event.
			Self::deposit_event(Event::ResetUserCount { usercount, who });
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		//------------------==
		/// An example dispatchable that may throw a custom error.
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <UserCount<T>>::get() {
				// Return an error if the value has not been set.
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or_else(|| Error::<T>::UserCountOverflow)?;
					// Update the value in storage with the incremented result.
					<UserCount<T>>::put(new);
					Ok(())
				},
			}
		}
	}
}
