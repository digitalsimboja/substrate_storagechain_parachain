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

		/// The minimum value which must not be below zero
		#[pallet::constant]
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
		Stored(Action, T::Hash, T::AccountId),
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
		/// When a user is attempting to store a negative number
		NegativeNumber,

		/// Invalid action type
		InvalidActionType,

		/// An operation that would lead to an overflow
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
		pub fn store(origin: OriginFor<T>, val_to_add: u32, action: Action) -> DispatchResult {
			let storer = ensure_signed(origin)?;
			ensure!(val_to_add >= T::StorageMinimum::get(), <Error<T>>::NegativeNumber);
			let storage_item =
				Storage { num: val_to_add.clone(), action: action.clone(), storer: storer.clone() };
			let stored_id = Self::add_to_storage(storage_item);

			// We simply log the stored Id
			log::info!("A storage item with ID: {:?} has been added to store.", stored_id);

			Ok(())
		}

		// Define an extrinsic function to change a stored action
		#[pallet::weight(1_000)]
		pub fn change_action(
			origin: OriginFor<T>,
			storage_id: T::Hash,
			new_action: Action,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Aside: Let's ensure the right owner is the one making the changes on storage
			// This is just an added edge case
			ensure!(Self::is_storage_owner(&storage_id, &sender)?, <Error<T>>::NotStorageOwner);

			let updated_id = Self::update_storage(storage_id, new_action);
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
		fn add_to_storage(storage: Storage<T>) -> Result<T::Hash, Error<T>> {
			let storage_id = T::Hashing::hash_of(&storage);

			// Check the number of items in the store
			// We could go further to check if maximum storage is reached assuming we want to
			let new_cnt = Self::counts_of_storage().checked_add(1).ok_or(<Error<T>>::Overflow)?;
			let sender = storage.storer.clone();
			let action = storage.action.clone();
			<Storages<T>>::insert(storage_id, storage);
			<CountsOfStorage<T>>::put(new_cnt);

			Self::deposit_event(Event::Stored(action, storage_id, sender));

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

		pub fn update_storage(storage_id: T::Hash, action: Action) -> DispatchResult {
			let payload = Self::storages(&storage_id).ok_or(<Error<T>>::ItemNotExist)?;
			let new_update = Storage { num: payload.num, action, storer: payload.storer };
			<Storages<T>>::mutate(storage_id, |items| match items {
				None => Err(()),
				Some(val) => {
					*val = new_update;
					Ok(storage_id)
				},
			})
			.map_err(|_| <Error<T>>::ItemNotExist)?;

			Ok(())
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
