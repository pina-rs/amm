//! The full-width oracle's account migration preserves the sampling boundary.

#[path = "../migrations/transitions/account_1_02/v0_to_v1.rs"]
mod transition;

use pina_amm_client::accounts::Pool;

#[test]
fn migrated_samples_keep_the_low_limb_clock_and_reserves() {
	let mut data = vec![0; transition::WORKING_SIZE];
	data[0] = 2;
	data[1] = u8::try_from(transition::FROM_VERSION).unwrap();
	data[280..296].copy_from_slice(&(u128::MAX - 7).to_le_bytes());
	data[296..304].copy_from_slice(&1_000u64.to_le_bytes());
	data[304..312].copy_from_slice(&1u64.to_le_bytes());
	data[312..320].copy_from_slice(&(1u64 << 63).to_le_bytes());
	assert!(Pool::from_bytes(&data[..transition::SOURCE_SIZE]).is_err());

	transition::migrate(&mut data);
	// The migration executor commits the version after applying the transition.
	data[1] = u8::try_from(transition::TO_VERSION).unwrap();
	assert_eq!(data.len(), transition::DESTINATION_SIZE);
	let migrated = Pool::from_bytes(&data).expect("current client decodes migrated pool");
	assert_eq!(migrated.price0_cumulative_last.get(), u128::MAX - 7);
	assert_eq!(migrated.price0_cumulative_high_last.get(), 0);
	assert_eq!(migrated.last_update_timestamp.get(), 1_000);
	assert_eq!(migrated.reserve0_last.get(), 1);
	assert_eq!(migrated.reserve1_last.get(), 1 << 63);
}
