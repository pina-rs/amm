//! Proof that a signer is this program's upgrade authority.
//!
//! Fee tiers are protocol-level configuration, so only the key that can
//! upgrade the program may create them. Reading the authority from the
//! loader's program-data account keeps the gate correct for every deployment
//! of this source, with no compiled-in admin key and no initialization race.

use pina::*;

use crate::ID;
use crate::errors::AmmError;

/// The upgradeable BPF loader that owns every upgradeable program.
pub const BPF_LOADER_UPGRADEABLE_ID: Address =
	address!("BPFLoaderUpgradeab1e11111111111111111111111");

/// `UpgradeableLoaderState::ProgramData` tag, little-endian `u32`.
const PROGRAM_DATA_TAG: [u8; 4] = [3, 0, 0, 0];

/// Offset of the `Option<Address>` upgrade authority in program data:
/// a four-byte tag followed by the eight-byte deployment slot.
const AUTHORITY_OPTION_OFFSET: usize = 12;

/// Require `authority` to sign and to be the upgrade authority recorded in
/// this program's canonical program-data account.
pub fn assert_upgrade_authority(
	program_data: &AccountView,
	authority: &AccountView,
) -> ProgramResult {
	authority.assert_signer()?;
	program_data
		.assert_owner(&BPF_LOADER_UPGRADEABLE_ID)
		.map_err(|_| AmmError::InvalidProgramData)?;
	let (expected, _) = try_find_program_address(&[ID.as_ref()], &BPF_LOADER_UPGRADEABLE_ID)
		.ok_or(AmmError::InvalidProgramData)?;
	if program_data.address() != &expected {
		return Err(AmmError::InvalidProgramData.into());
	}

	let data = program_data.try_borrow()?;
	let authority_start = AUTHORITY_OPTION_OFFSET + 1;
	let recorded = data
		.get(authority_start..authority_start + ADDRESS_BYTES)
		.ok_or(AmmError::InvalidProgramData)?;
	if data.get(..PROGRAM_DATA_TAG.len()) != Some(&PROGRAM_DATA_TAG)
		|| data.get(AUTHORITY_OPTION_OFFSET) != Some(&1)
	{
		return Err(AmmError::InvalidProgramData.into());
	}
	if recorded != authority.address().as_ref() {
		return Err(AmmError::Unauthorized.into());
	}
	Ok(())
}
