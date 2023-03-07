#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
//use frame_support::traits::{Currency, OnUnbalanced, ReservableCurrency};
pub use pallet::*;
//use sp_runtime::traits::{StaticLookup, Zero};
//use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/*const LOG_TARGET: &str = "runtime::uniques";
type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
type NegativeImbalanceOf<T> =
	<<T as Config>::Currency as Currency<AccountIdOf<T>>>::NegativeImbalance;
type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;
*/
#[frame_support::pallet]
pub mod pallet {
  use super::*;
	use frame_support::{inherent::Vec, pallet_prelude::{*, ValueQuery}};
	use frame_system::pallet_prelude::*;
  use log::info;//error, warn, 

	#[pallet::pallet]
	//#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);
  //pub struct Pallet<T, I = ()>(_);
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		#[pallet::constant]
		type StringMax: Get<u8>;
	}

	#[derive(Default, Encode, Decode, Clone, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
	//#[scale_info(skip_type_params(T))]
	//#[codec(mel_bound())]
	pub struct UserInfo {
		pub id: u32,
		pub username: [u8; 20],
    //BoundedVec<u8, T::StringMax>,
    pub staked: u32,
	}

	//#[pallet::unbounded]
	#[pallet::storage]
	#[pallet::getter(fn info)]
	/// Info on all of the info.
	pub type AccountToUserInfo<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, UserInfo, OptionQuery>;//<T>

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn usercount)]
	pub type UserCount<T> = StorageValue<_, u32>;

  #[pallet::storage]
  #[pallet::getter(fn totalsupply)]
  pub type TotalSupply<T> = StorageValue<_, u64, ValueQuery>;

  #[pallet::storage]
  #[pallet::getter(fn init)]
  pub type Init<T: Config> = StorageValue<_, bool, ValueQuery>;

  #[pallet::type_value]
  pub(super) fn TotalSupplyDefaultValue<T: Config>() -> u64 {
    21000000 //Zero::zero()
  }

	#[pallet::storage]
	#[pallet::getter(fn balances)]
	pub(super) type Balances<T: Config> =
  StorageMap<_, Blake2_128Concat, T::AccountId, u64, ValueQuery>;
	//pub type BalanceOf<T> =
	//	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

  #[pallet::hooks]
  impl <T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
      //call on each block execution
  }

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [usercount, who]
		ResetUserCount {
			usercount: u32,
			who: T::AccountId,
		},
		UserAdded {
			user_index: u32,//from 1
			user: T::AccountId,
		},//deposit: BalanceOf<T>
    /// Tokens was initialized by user
    Initialized {initializer: T::AccountId, amount: u64},
    /// Tokens successfully transferred between users
    TransferSucceeded{
      from: T::AccountId, to: T::AccountId, amount: u64},
    //BalanceOf<T>),
	}

	// Errors inform users that usercount went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
    VecToArray,
    StringTooShort,
    StringTooLong,
    /// Attempted to initialize the token after it had already been initialized.
    AlreadyInitialized,
    /// Attempted to transfer more funds than there are available
    InsufficientFunds,
    /// BalanceOverflow
    BalanceOverflow
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
    //------------------==
		#[pallet::call_index(2)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn add_user(
			origin: OriginFor<T>,
			num_one: u32,
			username: Vec<u8>
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
      username2.resize(explen, 32);//ASCII code table
      info!("username2 len: {:?}", username2.len());

      let arr = username2.try_into().map_err(|_| Error::<T>::VecToArray)?;

      let new_uidx = match <UserCount<T>>::get() {
				None => 1,
        //return Err(Error::<T>::NoneValue.into()),
				Some(old) => 
					old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?,
			};
      <UserCount<T>>::put(new_uidx);
      info!("new_uidx: {:?}", new_uidx);
			let user = UserInfo { id: new_uidx, username: arr, staked: 0u32};
			<AccountToUserInfo<T>>::insert(&from, user);

			Self::deposit_event(Event::UserAdded { user_index: new_uidx, user: from });
			Ok(())
		}

    //------------------==
		#[pallet::call_index(3)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn initialize(origin: OriginFor<T>, totalsupply: u64) -> DispatchResult {
			let from = ensure_signed(origin)?;
      ensure!(!<Init<T>>::get(), Error::<T>::AlreadyInitialized);
      <TotalSupply<T>>::put(totalsupply);
      info!("totalsupply: {:?}", totalsupply);
      <Balances<T>>::insert(&from, totalsupply);

      Init::<T>::put(true);//only once initialisation
      Self::deposit_event(Event::Initialized{initializer: from, amount: totalsupply});
      Ok(())
		}

    //------------------==
		#[pallet::call_index(4)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn transfer(origin: OriginFor<T>, to: T::AccountId, amount: u64) -> DispatchResult {
			let from = ensure_signed(origin)?;
      let from_balance = <Balances<T>>::get(&from);
      let to_balance = <Balances<T>>::get(&to);

      let new_from_balance = from_balance.checked_sub(amount).ok_or(Error::<T>::InsufficientFunds)?;
      //return Err(Error::<T>::NoneValue.into()),

      let new_to_balance = to_balance.checked_add(amount).ok_or(Error::<T>::BalanceOverflow)?;

      <Balances<T>>::insert(&from, new_from_balance);
      <Balances<T>>::insert(&to, new_to_balance);

      Self::deposit_event(Event::TransferSucceeded{
        from, to, amount});
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
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<UserCount<T>>::put(new);
					Ok(())
				},
			}
		}
	}
}
