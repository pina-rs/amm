//! Command-line arguments.
//!
//! Amounts are always raw base units (the integer a token account stores),
//! never UI amounts, so a command means the same thing for every mint.

use clap::Args;
use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use solana_pubkey::Pubkey;

/// Command-line interface for the Pina AMM.
#[derive(Debug, Parser)]
#[command(name = "pina-amm", version, propagate_version = true, about)]
pub struct Cli {
	/// Options shared by every command.
	#[command(flatten)]
	pub global: GlobalArgs,

	/// The command to run.
	#[command(subcommand)]
	pub command: Command,
}

/// Options shared by every command.
#[derive(Debug, Args)]
pub struct GlobalArgs {
	/// RPC endpoint: `mainnet`, `devnet`, `testnet`, `localhost`, or an https
	/// URL.
	#[arg(
		short = 'u',
		long,
		global = true,
		default_value = "devnet",
		env = "SOLANA_URL"
	)]
	pub url: String,

	/// Fee payer and signer keypair, in the Solana CLI's JSON byte-array
	/// format.
	#[arg(
		short = 'k',
		long,
		global = true,
		default_value = "~/.config/solana/id.json",
		env = "PINA_AMM_KEYPAIR"
	)]
	pub keypair: String,

	/// Simulate and print the result instead of sending a transaction.
	#[arg(long, global = true)]
	pub simulate: bool,

	/// Print machine-readable JSON.
	#[arg(long, global = true)]
	pub json: bool,
}

/// Top-level commands.
#[derive(Debug, Subcommand)]
pub enum Command {
	/// Create, update, or inspect fee tiers.
	#[command(subcommand)]
	Config(ConfigCommand),
	/// Create a pool, inspect it, or hand over its creator fees.
	#[command(subcommand)]
	Pool(PoolCommand),
	/// Quote a swap without sending it.
	Quote(SwapArgs),
	/// Swap one pool token for the other.
	Swap(SwapArgs),
	/// Add liquidity in proportion to the pool's reserves.
	Deposit(LiquidityArgs),
	/// Remove liquidity in proportion to the pool's reserves.
	Withdraw(LiquidityArgs),
	/// Collect accrued protocol or creator fees.
	#[command(subcommand)]
	Fees(FeesCommand),
}

/// `pina-amm config ...`
#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
	/// Create a fee tier. The signer must be the program's upgrade authority.
	Create(CreateConfigArgs),
	/// Update a tier's authority and the rates future pools snapshot.
	Update(UpdateConfigArgs),
	/// Print a tier.
	Show(ConfigIndexArgs),
}

/// Fee rates, in parts per million (`2500` is 0.25%). The field names match
/// the program's instruction fields, so the shared suffix is deliberate.
#[derive(Debug, Args)]
#[allow(clippy::struct_field_names)]
pub struct RateArgs {
	/// Fee charged on every swap; LPs keep what the protocol does not take.
	#[arg(long)]
	pub trade_fee_rate: u32,
	/// Protocol share of the trade fee.
	#[arg(long, default_value_t = 0)]
	pub protocol_fee_rate: u32,
	/// Additional fee paid to each pool's creator.
	#[arg(long, default_value_t = 0)]
	pub creator_fee_rate: u32,
}

/// `pina-amm config create`
#[derive(Debug, Args)]
pub struct CreateConfigArgs {
	/// Tier index; tiers are addressed by it.
	#[arg(long)]
	pub index: u16,
	/// The tier's fee rates.
	#[command(flatten)]
	pub rates: RateArgs,
	/// Tier authority. Defaults to the signer.
	#[arg(long)]
	pub authority: Option<Pubkey>,
	/// Restrict pool creation in this tier to one signer, such as a launchpad
	/// program's PDA.
	#[arg(long)]
	pub pool_creator_authority: Option<Pubkey>,
}

/// `pina-amm config update`
#[derive(Debug, Args)]
pub struct UpdateConfigArgs {
	/// Tier index.
	#[arg(long)]
	pub index: u16,
	/// The rates future pools snapshot.
	#[command(flatten)]
	pub rates: RateArgs,
	/// Hand the tier to a new authority. Defaults to keeping the signer.
	#[arg(long)]
	pub new_authority: Option<Pubkey>,
}

/// A tier, addressed by index.
#[derive(Debug, Args)]
pub struct ConfigIndexArgs {
	/// Tier index.
	#[arg(long)]
	pub index: u16,
}

/// `pina-amm pool ...`
#[derive(Debug, Subcommand)]
pub enum PoolCommand {
	/// Create a pool and make its first deposit.
	Create(CreatePoolArgs),
	/// Print a pool, its reserves, and its price.
	Show(PoolArgs),
	/// Hand the pool's creator-fee rights to another address.
	SetCreator(SetCreatorArgs),
}

/// Which token pays a pool's creator fee.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum CreatorFeeMode {
	/// The input token of each swap.
	Input,
	/// Always the first mint given (`--mint-a`).
	MintA,
	/// Always the second mint given (`--mint-b`).
	MintB,
}

/// `pina-amm pool create`
#[derive(Debug, Args)]
pub struct CreatePoolArgs {
	/// Tier index to create the pool under.
	#[arg(long)]
	pub config: u16,
	/// One mint of the pair.
	#[arg(long)]
	pub mint_a: Pubkey,
	/// The other mint of the pair.
	#[arg(long)]
	pub mint_b: Pubkey,
	/// Initial deposit of `mint_a`, in base units.
	#[arg(long)]
	pub amount_a: u64,
	/// Initial deposit of `mint_b`, in base units.
	#[arg(long)]
	pub amount_b: u64,
	/// Recipient of creator fees. Defaults to the signer.
	#[arg(long)]
	pub creator: Option<Pubkey>,
	/// Which token pays the creator fee.
	#[arg(long, value_enum, default_value_t = CreatorFeeMode::Input)]
	pub creator_fee_mode: CreatorFeeMode,
}

/// A pool, addressed by its address.
#[derive(Debug, Args)]
pub struct PoolArgs {
	/// Pool address.
	#[arg(long)]
	pub pool: Pubkey,
}

/// `pina-amm pool set-creator`
#[derive(Debug, Args)]
pub struct SetCreatorArgs {
	/// Pool address.
	#[arg(long)]
	pub pool: Pubkey,
	/// New recipient of the pool's creator fees.
	#[arg(long)]
	pub new_creator: Pubkey,
}

/// `pina-amm quote` and `pina-amm swap`
#[derive(Debug, Args)]
#[command(group(clap::ArgGroup::new("amount").required(true).args(["amount_in", "amount_out"])))]
pub struct SwapArgs {
	/// Pool address.
	#[arg(long)]
	pub pool: Pubkey,
	/// The mint being sold.
	#[arg(long)]
	pub sell: Pubkey,
	/// Sell exactly this many base units.
	#[arg(long)]
	pub amount_in: Option<u64>,
	/// Buy exactly this many base units of the other mint.
	#[arg(long)]
	pub amount_out: Option<u64>,
	/// Slippage tolerance in basis points (`50` is 0.5%).
	#[arg(long, default_value_t = 50)]
	pub slippage_bps: u16,
}

/// `pina-amm deposit` and `pina-amm withdraw`
#[derive(Debug, Args)]
pub struct LiquidityArgs {
	/// Pool address.
	#[arg(long)]
	pub pool: Pubkey,
	/// LP base units to mint or burn.
	#[arg(long)]
	pub lp_amount: u64,
	/// Slippage tolerance in basis points (`50` is 0.5%).
	#[arg(long, default_value_t = 50)]
	pub slippage_bps: u16,
}

/// `pina-amm fees ...`
#[derive(Debug, Subcommand)]
pub enum FeesCommand {
	/// Collect a pool's protocol fees. The signer must be the tier authority.
	CollectProtocol(PoolArgs),
	/// Collect a pool's creator fees. The signer must be the pool creator.
	CollectCreator(PoolArgs),
}

#[cfg(test)]
mod tests {
	use clap::CommandFactory;

	use super::*;

	#[test]
	fn command_definition_is_consistent() {
		Cli::command().debug_assert();
	}

	#[test]
	fn swap_requires_exactly_one_amount() {
		let pool = Pubkey::new_unique().to_string();
		let sell = Pubkey::new_unique().to_string();
		let base = ["pina-amm", "swap", "--pool", &pool, "--sell", &sell];
		assert!(Cli::try_parse_from(base).is_err());
		assert!(Cli::try_parse_from([&base[..], &["--amount-in", "5"]].concat()).is_ok());
		assert!(
			Cli::try_parse_from([&base[..], &["--amount-in", "5", "--amount-out", "5"]].concat())
				.is_err()
		);
	}
}
