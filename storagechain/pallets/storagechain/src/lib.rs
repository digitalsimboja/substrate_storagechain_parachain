#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::traits::Hash,
		traits::{Currency, Get, Randomness},
	};
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;

	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};

	// Write a Struct to hold Storage information.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct Storage<T: Config> {
		pub num: u32,
		pub action: Action,
		pub storer: <T as frame_system::Config>::AccountId,
	}

	// Enum declaration for Action.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum Action {
		Increment,
		Decrement,
		Idle,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types it depends on.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		#[pallet::constant]

		/// The minimum value which must not be below zero
		type StorageMinimum: Get<u32>;

		/// The Currency handler for the Storagechain pallet.
		type Currency: Currency<Self::AccountId>;

		// Specify the type for Randomness we want to specify for runtime.
		type StorageRandomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	// Counts of number of storage
	#[pallet::storage]
	#[pallet::getter(fn counts_of_storage)]
	pub(super) type CountsOfStorage<T: Config> = StorageValue<_, u32, ValueQuery>;
	/*
	#[pallet::storage]
	#[pallet::getter(fn single_action)]
	pub(super) type SingleAction<T: Config> = StorageValue<_, u32, ValueQuery>;

	*/

	// create storage instance for our storages struct
	#[pallet::storage]
	#[pallet::getter(fn storages)]
	pub(super) type Storages<T: Config> = StorageMap<_, Twox64Concat, T::Hash, Storage<T>>;

	// Events.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// When a new item is Stored
		Stored(Option<Action>, T::Hash, T::AccountId),
		/// In case we wish to clear the storage after some blocks
		Cleared(u32),

		/// When a new action is updated on the storage
		ActionChanged(T::AccountId, T::Hash, Option<Action>),

		// When a value is incremented
		Incremented(T::AccountId, T::Hash, Action),

		// When a value is decremented
		Decremented(T::AccountId, T::Hash, Action),
	}

	// Errors.
	#[pallet::error]
	pub enum Error<T> {
		// When a user is attempting to store a negative number
		NegativeNumber,

		// Invalid action type
		InvalidActionType,

		/// An operation would lead to an overflow
		Overflow,

		/// Allow the storage to be changed by the person who stored the item
		NotStorageOwner,

		/// Item storage Id not existing in storage
		ItemNotExist,
	}

	// Extrinsics callable from outside the runtime.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// Store a single value and an action
		#[pallet::weight(1_000)]
		pub fn store(
			origin: OriginFor<T>,
			val_to_add: u32,
			action: Option<Action>,
		) -> DispatchResult {
			let storer = ensure_signed(origin)?;
			let action_match = action;
			let number = val_to_add;
			ensure!(val_to_add >= T::StorageMinimum::get(), <Error<T>>::NegativeNumber);
			let stored_id = Self::add_to_storage(storer, number, action_match);

			// We simply log the stored Id
			log::info!("An storage item with ID: {:?} has been added to store.", stored_id);

			Ok(())
		}

		// Define an extrinsic function to change a stored action
		#[pallet::weight(1_000)]
		pub fn change_action(
			origin: OriginFor<T>,
			storage_id: T::Hash,
			new_action: Option<Action>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Aside: Let's ensure the right owner is the one making the changes on storage
			// This is just an added edge case
			ensure!(Self::is_storage_owner(&storage_id, &sender)?, <Error<T>>::NotStorageOwner);
			let mut storage_item = Self::storages(&storage_id).ok_or(<Error<T>>::ItemNotExist)?;
			storage_item.action = new_action.clone().unwrap();

			let updated_id = Self::update_storage(sender, storage_id, storage_item, new_action);

			// We simply log the id
			log::info!("An storage item with ID: {:?} has been updated", updated_id);

			Ok(())
		}

		// Define an Extrinsic to execute the action stored on storage
		#[pallet::weight(1_000)]
		pub fn execute(
			origin: OriginFor<T>,
			storage_id: T::Hash,
			action: Option<Action>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Aside: Let's ensure the right owner is the one making the changes on storage
			ensure!(Self::is_storage_owner(&storage_id, &sender)?, <Error<T>>::NotStorageOwner);

			// Get the stored item
			let storage_item = Self::storages(&storage_id).ok_or(<Error<T>>::ItemNotExist)?;
			let call_action = action.clone();

			match call_action {
				Some(call_type @ Action::Increment) => {
					_ = Self::_increment(storage_item, storage_id, sender, call_type);
				},
				Some(call_type @ Action::Decrement) => {
					_ = Self::_decrement(storage_item, storage_id, sender, call_type);
				},
				_ => (),
			}



			Ok(())
		}
	}

	//** Our helper functions.**//

	impl<T: Config> Pallet<T> {
		// helper function for Storage struct for changing the action
		fn gen_action() -> Action {
			let random = T::StorageRandomness::random(&b"action"[..]).0;
			match random.as_ref()[0] % 3 {
				0 => Action::Increment,
				1 => Action::Decrement,
				_ => Action::Idle,
			}
		}

		fn add_to_storage(
			storer: T::AccountId,
			val_to_add: u32,
			action: Option<Action>,
		) -> Result<T::Hash, Error<T>> {
			let storage_item = Storage::<T> {
				num: val_to_add,
				storer: storer.clone(),
				action: action.clone().unwrap_or_else(|| Self::gen_action()),
			};
			let storage_id = T::Hashing::hash_of(&storage_item);

			// Check the number of items in the store
			// We could go further to check if maximum storage is reached assuming we want to
			let new_cnt = Self::counts_of_storage().checked_add(1).ok_or(<Error<T>>::Overflow)?;
			
			<Storages<T>>::insert(storage_id, storage_item);
			<CountsOfStorage<T>>::put(new_cnt);

	

			Self::deposit_event(Event::Stored(action, storage_id, storer));

			Ok(storage_id)
		}

		fn is_storage_owner(
			storage_id: &T::Hash,
			account: &T::AccountId,
		) -> Result<bool, Error<T>> {
			match Self::storages(storage_id) {
				Some(item) => Ok(item.storer == *account),
				None => Err(<Error<T>>::ItemNotExist),
			}
		}

		fn update_storage(
			storer: T::AccountId,
			item_id: T::Hash,
			storage: Storage<T>,
			action: Option<Action>,
		) -> Result<T::Hash, Error<T>> {
			<Storages<T>>::insert(&item_id, storage);

			Self::deposit_event(Event::ActionChanged(storer, item_id, action));

			Ok(item_id)
		}

		fn _increment(
			storage: Storage<T>,
			id: T::Hash,
			sender: T::AccountId,
			action: Action,
		) -> Result<T::Hash, Error<T>> {
			let action_id = T::Hashing::hash_of(&storage);
			let mut storage = storage;
			let num_inc = storage.num + 1;
			storage.num = num_inc;
			<Storages<T>>::insert(&id, storage);

			Self::deposit_event(Event::Incremented(sender, id, action));

			Ok(action_id)
		}

		fn _decrement(
			storage: Storage<T>,
			id: T::Hash,
			sender: T::AccountId,
			action: Action,
		) -> Result<T::Hash, Error<T>> {
			let action_id = T::Hashing::hash_of(&storage);
			let mut storage = storage;
			storage.num = storage.num - 1;
			<Storages<T>>::insert(&id, storage);

			Self::deposit_event(Event::Decremented(sender, id, action));

			Ok(action_id)
		}
	}
}
