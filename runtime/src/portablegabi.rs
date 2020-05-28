use rstd::vec::Vec;
use support::{decl_event, decl_module, decl_storage, dispatch::Result, StorageMap};
use system::ensure_signed;

use crate::error;

/// The pallet's configuration trait.
pub trait Trait: system::Trait + error::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
		/// The AccumulatorList contains all accumulator. It is a map which
		/// maps an account id and an index to an accumulator
		AccumulatorList get(accumulator_list): map (T::AccountId, u64) => Option<Vec<u8>>;

		/// The AccumulatorCounter stores for each attester the number of
		/// accumulator updates.
		AccumulatorCount get(accumulator_count): map T::AccountId => u64;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		fn deposit_event<T>() = default;

		/// Updates the attestation
		pub fn update_accumulator(origin, accumulator: Vec<u8>) -> Result {
			let attester = ensure_signed(origin)?;

			// if attester didn't store any accumulators, this will be 0
			// 0 is the default value for new keys.
			let counter = <AccumulatorCount<T>>::get(&attester);

			// new counter value
			let next = <error::Module<T>>::ok_or_deposit_err(
				counter.checked_add(1),
				Self::ERROR_OVERFLOW
			)?;

			// set bytes at index `counter` to accumulator
			// update counter to `next`
			if !<AccumulatorList<T>>::exists((attester.clone(), counter)) {
				<AccumulatorList<T>>::insert((attester.clone(), counter), &accumulator);
				<AccumulatorCount<T>>::insert(&attester, next);
	
				Self::deposit_event(RawEvent::Updated(attester, next, accumulator));
				Ok(())
			} else {
				Self::error(Self::ERROR_INCONSISTENT)
			}
		}
	}
}

impl<T: Trait> Module<T> {
	pub const ERROR_BASE: u16 = 4000;
	pub const ERROR_OVERFLOW: error::ErrorType = (Self::ERROR_BASE + 1, "accumulator overflow");
	pub const ERROR_INCONSISTENT: error::ErrorType = (Self::ERROR_BASE + 1, "inconsistent accumulator counter");

	/// Create an error using the error module
	pub fn error(error_type: error::ErrorType) -> Result {
		<error::Module<T>>::error(error_type)
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
	{
		/// An accumulator has been updated. Therefore an attestation has be revoked
		Updated(AccountId, u64, Vec<u8>),
	}
);

/// tests for this pallet
#[cfg(test)]
mod tests {
	use super::*;

	use primitives::{Blake2Hasher, H256};
	use runtime_io::with_externalities;
	use support::{assert_ok, impl_outer_origin};

	use runtime_primitives::{
		testing::{Digest, DigestItem, Header},
		traits::{BlakeTwo256, IdentityLookup},
		BuildStorage,
	};

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	impl system::Trait for Test {
		type Origin = Origin;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type Digest = Digest;
		type Log = DigestItem;
	}

	impl Trait for Test {
		type Event = ();
	}

	impl error::Trait for Test {
		type Event = ();
		type ErrorCode = u16;
	}

	type PortablegabiModule = Module<Test>;

	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::<Test>::default()
			.build_storage()
			.unwrap()
			.0
			.into()
	}

	#[test]
	fn it_works_for_default_value() {
		with_externalities(&mut new_test_ext(), || {
			// Just a dummy test for the dummy function `do_something`
			// calling the `do_something` function with a value 42
			assert_ok!(PortablegabiModule::update_accumulator(
				Origin::signed(1),
				vec![1u8, 2u8, 3u8]
			));
			assert_ok!(PortablegabiModule::update_accumulator(
				Origin::signed(1),
				vec![4u8, 5u8, 6u8]
			));
			assert_ok!(PortablegabiModule::update_accumulator(
				Origin::signed(1),
				vec![7u8, 8u8, 9u8]
			));

			// There should be three accumulators inside the store
			assert_eq!(PortablegabiModule::accumulator_count(1), 3);

			// asserting that the stored value is equal to what we stored
			assert_eq!(
				PortablegabiModule::accumulator_list((1, 0)),
				Some(vec![1u8, 2u8, 3u8])
			);
			assert_eq!(
				PortablegabiModule::accumulator_list((1, 1)),
				Some(vec![4u8, 5u8, 6u8])
			);
			assert_eq!(
				PortablegabiModule::accumulator_list((1, 2)),
				Some(vec![7u8, 8u8, 9u8])
			);
		});
	}
}
