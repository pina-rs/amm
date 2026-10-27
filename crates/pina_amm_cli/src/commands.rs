//! Command implementations.

use pina_amm_client::accounts::Pool;
use pina_amm_client::events::Swapped;
use pina_amm_client::instructions::CollectCreatorFees;
use pina_amm_client::instructions::CollectCreatorFeesInstructionData;
use pina_amm_client::instructions::CollectProtocolFees;
use pina_amm_client::instructions::CollectProtocolFeesInstructionData;
use pina_amm_client::instructions::CreateConfig;
use pina_amm_client::instructions::CreateConfigInstructionData;
use pina_amm_client::instructions::CreatePool;
use pina_amm_client::instructions::CreatePoolInstructionData;
use pina_amm_client::instructions::Deposit;
use pina_amm_client::instructions::DepositInstructionData;
use pina_amm_client::instructions::SetPoolCreator;
use pina_amm_client::instructions::SetPoolCreatorInstructionData;
use pina_amm_client::instructions::SwapExactIn;
use pina_amm_client::instructions::SwapExactInInstructionData;
use pina_amm_client::instructions::SwapExactOut;
use pina_amm_client::instructions::SwapExactOutInstructionData;
use pina_amm_client::instructions::UpdateConfig;
use pina_amm_client::instructions::UpdateConfigInstructionData;
use pina_amm_client::instructions::Withdraw;
use pina_amm_client::instructions::WithdrawInstructionData;
use serde_json::json;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::accounts::ConfigView;
use crate::accounts::PoolView;
use crate::accounts::TOKEN_PROGRAM;
use crate::accounts::associated_token_address;
use crate::accounts::config_address;
use crate::accounts::create_associated_token_account;
use crate::accounts::load_config;
use crate::accounts::load_pool;
use crate::accounts::lp_mint_address;
use crate::accounts::ordered;
use crate::accounts::program_data_address;
use crate::accounts::token_program_of;
use crate::accounts::vault_address;
use crate::cli::Command;
use crate::cli::ConfigCommand;
use crate::cli::CreateConfigArgs;
use crate::cli::CreatePoolArgs;
use crate::cli::CreatorFeeMode;
use crate::cli::FeesCommand;
use crate::cli::LiquidityArgs;
use crate::cli::PoolCommand;
use crate::cli::QuoteArgs;
use crate::cli::SetCreatorArgs;
use crate::cli::SwapArgs;
use crate::cli::UpdateConfigArgs;
use crate::context::Context;
use crate::error::CliError;

const SWAPPED_DISCRIMINATOR: u8 = 2;

/// Run one command.
pub fn run(context: &Context, command: Command) -> Result<(), CliError> {
	match command {
		Command::Config(ConfigCommand::Create(args)) => create_config(context, &args),
		Command::Config(ConfigCommand::Update(args)) => update_config(context, &args),
		Command::Config(ConfigCommand::Show(args)) => {
			let config = load_config(context, &config_address(args.index))?;
			print_config(context, &config);
			Ok(())
		}
		Command::Pool(PoolCommand::Create(args)) => create_pool(context, &args),
		Command::Pool(PoolCommand::Show(args)) => {
			let pool = load_pool(context, &args.pool)?;
			print_pool(context, &pool);
			Ok(())
		}
		Command::Pool(PoolCommand::SetCreator(args)) => set_creator(context, &args),
		Command::Quote(args) => swap(context, &args, None),
		Command::Swap(args) => {
			let amount = swap_amount(&args);
			swap(context, &args.quote, Some(amount))
		}
		Command::Deposit(args) => {
			liquidity(
				context,
				&args.liquidity,
				true,
				(args.maximum_amount_0, args.maximum_amount_1),
			)
		}
		Command::Withdraw(args) => {
			liquidity(
				context,
				&args.liquidity,
				false,
				(args.minimum_amount_0, args.minimum_amount_1),
			)
		}
		Command::Fees(FeesCommand::CollectProtocol(args)) => {
			collect_fees(context, &args.pool, true)
		}
		Command::Fees(FeesCommand::CollectCreator(args)) => {
			collect_fees(context, &args.pool, false)
		}
	}
}

fn report(context: &Context, action: &str, signature: Option<String>, details: serde_json::Value) {
	if context.json {
		println!(
			"{}",
			json!({ "action": action, "signature": signature, "details": details })
		);
		return;
	}
	match signature {
		Some(signature) => println!("{action}: {signature}"),
		None => println!("{action}: simulated"),
	}
	if let serde_json::Value::Object(fields) = details {
		for (key, value) in fields {
			println!("  {key}: {value}");
		}
	}
}

fn create_config(context: &Context, args: &CreateConfigArgs) -> Result<(), CliError> {
	let address = config_address(args.index);
	let authority = args.authority.unwrap_or_else(|| context.signer());
	let pool_creator_authority = args.pool_creator_authority.unwrap_or_default();
	let data = CreateConfigInstructionData::new(|data| {
		data.index.set(args.index);
		data.trade_fee_rate.set(args.rates.trade_fee_rate);
		data.protocol_fee_rate.set(args.rates.protocol_fee_rate);
		data.creator_fee_rate.set(args.rates.creator_fee_rate);
		data.authority = authority;
		data.pool_creator_authority = pool_creator_authority;
	})
	.map_err(|_| CliError::InvalidInstructionData)?;
	let instruction = CreateConfig::new(
		context.signer(),
		context.signer(),
		program_data_address(),
		address,
	)
	.instruction(data);
	let signature = context.send(&[instruction])?;
	report(
		context,
		"config created",
		signature,
		json!({ "config": address.to_string(), "authority": authority.to_string() }),
	);
	Ok(())
}

fn update_config(context: &Context, args: &UpdateConfigArgs) -> Result<(), CliError> {
	let address = config_address(args.index);
	let new_authority = args.new_authority.unwrap_or_else(|| context.signer());
	let data = UpdateConfigInstructionData::new(|data| {
		data.new_authority = new_authority;
		data.trade_fee_rate.set(args.rates.trade_fee_rate);
		data.protocol_fee_rate.set(args.rates.protocol_fee_rate);
		data.creator_fee_rate.set(args.rates.creator_fee_rate);
	})
	.map_err(|_| CliError::InvalidInstructionData)?;
	let signature =
		context.send(&[UpdateConfig::new(context.signer(), address).instruction(data)])?;
	report(
		context,
		"config updated",
		signature,
		json!({ "config": address.to_string(), "authority": new_authority.to_string() }),
	);
	Ok(())
}

fn create_pool(context: &Context, args: &CreatePoolArgs) -> Result<(), CliError> {
	let config = config_address(args.config);
	let (mint_0, mint_1) = ordered(args.mint_a, args.mint_b)?;
	let a_is_0 = mint_0 == args.mint_a;
	let (amount_0, amount_1) = if a_is_0 {
		(args.amount_a, args.amount_b)
	} else {
		(args.amount_b, args.amount_a)
	};
	let creator_fee_mode = match (args.creator_fee_mode, a_is_0) {
		(CreatorFeeMode::Input, _) => 0,
		(CreatorFeeMode::MintA, true) | (CreatorFeeMode::MintB, false) => 1,
		(CreatorFeeMode::MintA, false) | (CreatorFeeMode::MintB, true) => 2,
	};
	let creator = args.creator.unwrap_or_else(|| context.signer());
	let program_0 = token_program_of(context, &mint_0)?;
	let program_1 = token_program_of(context, &mint_1)?;
	let pool = Pool::find_pda(&config, &mint_0, &mint_1).0;
	let lp_mint = lp_mint_address(&pool);
	let signer = context.signer();
	let data = CreatePoolInstructionData::new(|data| {
		data.amount0.set(amount_0);
		data.amount1.set(amount_1);
		data.creator = creator;
		data.creator_fee_mode = creator_fee_mode;
	})
	.map_err(|_| CliError::InvalidInstructionData)?;
	let instruction = CreatePool::new(
		signer,
		signer,
		signer,
		config,
		mint_0,
		mint_1,
		lp_mint,
		vault_address(&pool, &mint_0),
		vault_address(&pool, &mint_1),
		associated_token_address(&signer, &mint_0, &program_0),
		associated_token_address(&signer, &mint_1, &program_1),
		signer,
		associated_token_address(&signer, &lp_mint, &TOKEN_PROGRAM),
		program_0,
		program_1,
	)
	.instruction(data);
	let signature = context.send(&[instruction])?;
	report(
		context,
		"pool created",
		signature,
		json!({
			"pool": pool.to_string(),
			"lp_mint": lp_mint.to_string(),
			"mint_0": mint_0.to_string(),
			"mint_1": mint_1.to_string(),
		}),
	);
	Ok(())
}

fn set_creator(context: &Context, args: &SetCreatorArgs) -> Result<(), CliError> {
	let data = SetPoolCreatorInstructionData::new(|data| data.new_creator = args.new_creator)
		.map_err(|_| CliError::InvalidInstructionData)?;
	let signature =
		context.send(&[SetPoolCreator::new(context.signer(), args.pool).instruction(data)])?;
	report(
		context,
		"creator updated",
		signature,
		json!({ "pool": args.pool.to_string(), "creator": args.new_creator.to_string() }),
	);
	Ok(())
}

/// The trader's accounts and the pool vaults for one swap direction.
struct Route {
	input_token: Pubkey,
	output_token: Pubkey,
	input_vault: Pubkey,
	output_vault: Pubkey,
	input_program: Pubkey,
	output_program: Pubkey,
	output_mint: Pubkey,
}

fn route(pool: &PoolView, trader: &Pubkey, sell_token_0: bool) -> Route {
	let (input_mint, output_mint, input_vault, output_vault, input_program, output_program) =
		if sell_token_0 {
			(
				pool.mint_0,
				pool.mint_1,
				pool.vault_0,
				pool.vault_1,
				pool.token_program_0,
				pool.token_program_1,
			)
		} else {
			(
				pool.mint_1,
				pool.mint_0,
				pool.vault_1,
				pool.vault_0,
				pool.token_program_1,
				pool.token_program_0,
			)
		};
	Route {
		input_token: associated_token_address(trader, &input_mint, &input_program),
		output_token: associated_token_address(trader, &output_mint, &output_program),
		input_vault,
		output_vault,
		input_program,
		output_program,
		output_mint,
	}
}

fn swap_instruction(
	pool: &Pubkey,
	trader: &Pubkey,
	route: &Route,
	amount: Amount,
) -> Result<Instruction, CliError> {
	match amount {
		Amount::ExactIn {
			amount_in,
			minimum_out,
		} => {
			let data = SwapExactInInstructionData::new(|data| {
				data.amount_in.set(amount_in);
				data.minimum_amount_out.set(minimum_out);
			})
			.map_err(|_| CliError::InvalidInstructionData)?;
			Ok(SwapExactIn::new(
				*trader,
				*pool,
				route.input_token,
				route.output_token,
				route.input_vault,
				route.output_vault,
				route.input_program,
				route.output_program,
			)
			.instruction(data))
		}
		Amount::ExactOut {
			amount_out,
			maximum_in,
		} => {
			let data = SwapExactOutInstructionData::new(|data| {
				data.amount_out.set(amount_out);
				data.maximum_amount_in.set(maximum_in);
			})
			.map_err(|_| CliError::InvalidInstructionData)?;
			Ok(SwapExactOut::new(
				*trader,
				*pool,
				route.input_token,
				route.output_token,
				route.input_vault,
				route.output_vault,
				route.input_program,
				route.output_program,
			)
			.instruction(data))
		}
	}
}

/// The amount and limit of one swap.
#[derive(Clone, Copy)]
enum Amount {
	ExactIn { amount_in: u64, minimum_out: u64 },
	ExactOut { amount_out: u64, maximum_in: u64 },
}

/// Preserve the caller's absolute limit without consulting an RPC quote.
fn swap_amount(args: &SwapArgs) -> Amount {
	match (
		args.quote.amount_in,
		args.quote.amount_out,
		args.minimum_out,
		args.maximum_in,
	) {
		(Some(amount_in), None, Some(minimum_out), None) => {
			Amount::ExactIn {
				amount_in,
				minimum_out,
			}
		}
		(None, Some(amount_out), None, Some(maximum_in)) => {
			Amount::ExactOut {
				amount_out,
				maximum_in,
			}
		}
		_ => unreachable!("clap requires an exact amount and its matching limit"),
	}
}

fn swap(context: &Context, args: &QuoteArgs, amount: Option<Amount>) -> Result<(), CliError> {
	let pool = load_pool(context, &args.pool)?;
	let trader = context.signer();
	let sell_token_0 = pool.sells_token_0(&args.sell)?;
	let route = route(&pool, &trader, sell_token_0);
	let mut prefix = Vec::new();
	if !context.exists(&route.output_token)? {
		prefix.push(create_associated_token_account(
			&trader,
			&trader,
			&route.output_mint,
			&route.output_program,
		));
	}

	// Security: Only explicit caller limits reach a signed transaction.
	if let Some(amount) = amount {
		prefix.push(swap_instruction(&args.pool, &trader, &route, amount)?);
		let signature = context.send(&prefix)?;
		let limits = match amount {
			Amount::ExactIn {
				amount_in,
				minimum_out,
			} => json!({ "amount_in": amount_in, "minimum_out": minimum_out }),
			Amount::ExactOut {
				amount_out,
				maximum_in,
			} => json!({ "amount_out": amount_out, "maximum_in": maximum_in }),
		};
		report(
			context,
			"swapped",
			signature,
			json!({
				"pool": args.pool.to_string(),
				"sell": args.sell.to_string(),
				"limits": limits,
			}),
		);
		return Ok(());
	}

	// Open limits are used only for advisory quote simulations.
	let open = match (args.amount_in, args.amount_out) {
		(Some(amount_in), _) => {
			Amount::ExactIn {
				amount_in,
				minimum_out: 0,
			}
		}
		(None, Some(amount_out)) => {
			Amount::ExactOut {
				amount_out,
				maximum_in: u64::MAX,
			}
		}
		(None, None) => unreachable!("clap requires one amount"),
	};
	prefix.push(swap_instruction(&args.pool, &trader, &route, open)?);
	let simulation = context.simulate(&prefix)?;
	let event = simulation
		.events
		.iter()
		.find(|bytes| bytes.first() == Some(&SWAPPED_DISCRIMINATOR))
		.ok_or(CliError::MissingEvent)?;
	let quote = Swapped::from_bytes(event).map_err(|_| CliError::MissingEvent)?;
	let (amount_in, amount_out) = (quote.amount_in.get(), quote.amount_out.get());
	if quote.pool != args.pool
		|| quote.trader != trader
		|| quote.zero_for_one != u8::from(sell_token_0)
		|| args.amount_in.is_some_and(|amount| amount != amount_in)
		|| args.amount_out.is_some_and(|amount| amount != amount_out)
	{
		return Err(CliError::MismatchedQuote);
	}
	let details = json!({
		"pool": args.pool.to_string(),
		"sell": args.sell.to_string(),
		"amount_in": amount_in,
		"amount_out": amount_out,
		"trade_fee": quote.trade_fee.get(),
		"protocol_fee": quote.protocol_fee.get(),
		"creator_fee": quote.creator_fee.get(),
		"compute_units": simulation.units,
	});
	report(context, "quote", None, details);
	Ok(())
}

fn liquidity(
	context: &Context,
	args: &LiquidityArgs,
	deposit: bool,
	limits: (u64, u64),
) -> Result<(), CliError> {
	let pool = load_pool(context, &args.pool)?;
	let owner = context.signer();
	let token_0 = associated_token_address(&owner, &pool.mint_0, &pool.token_program_0);
	let token_1 = associated_token_address(&owner, &pool.mint_1, &pool.token_program_1);
	let lp = associated_token_address(&owner, &pool.lp_mint, &TOKEN_PROGRAM);
	let mut prefix = Vec::new();
	for (account, mint, program) in [
		(token_0, pool.mint_0, pool.token_program_0),
		(token_1, pool.mint_1, pool.token_program_1),
		(lp, pool.lp_mint, TOKEN_PROGRAM),
	] {
		if !context.exists(&account)? {
			prefix.push(create_associated_token_account(
				&owner, &owner, &mint, &program,
			));
		}
	}
	let build = |limit_0: u64, limit_1: u64| -> Result<Instruction, CliError> {
		if deposit {
			let data = DepositInstructionData::new(|data| {
				data.lp_amount.set(args.lp_amount);
				data.maximum_amount0.set(limit_0);
				data.maximum_amount1.set(limit_1);
			})
			.map_err(|_| CliError::InvalidInstructionData)?;
			Ok(Deposit::new(
				owner,
				args.pool,
				pool.vault_0,
				pool.vault_1,
				pool.lp_mint,
				token_0,
				token_1,
				lp,
				pool.token_program_0,
				pool.token_program_1,
			)
			.instruction(data))
		} else {
			let data = WithdrawInstructionData::new(|data| {
				data.lp_amount.set(args.lp_amount);
				data.minimum_amount0.set(limit_0);
				data.minimum_amount1.set(limit_1);
			})
			.map_err(|_| CliError::InvalidInstructionData)?;
			Ok(Withdraw::new(
				owner,
				args.pool,
				pool.vault_0,
				pool.vault_1,
				pool.lp_mint,
				token_0,
				token_1,
				lp,
				pool.token_program_0,
				pool.token_program_1,
			)
			.instruction(data))
		}
	};

	let mut instructions = prefix;
	instructions.push(build(limits.0, limits.1)?);
	let signature = context.send(&instructions)?;
	report(
		context,
		if deposit { "deposited" } else { "withdrew" },
		signature,
		json!({
			"pool": args.pool.to_string(),
			"lp_amount": args.lp_amount,
			"mint_0": pool.mint_0.to_string(),
			"mint_1": pool.mint_1.to_string(),
			"limit_0": limits.0,
			"limit_1": limits.1,
		}),
	);
	Ok(())
}

fn collect_fees(context: &Context, pool_address: &Pubkey, protocol: bool) -> Result<(), CliError> {
	let pool = load_pool(context, pool_address)?;
	let signer = context.signer();
	let recipient_0 = associated_token_address(&signer, &pool.mint_0, &pool.token_program_0);
	let recipient_1 = associated_token_address(&signer, &pool.mint_1, &pool.token_program_1);
	let mut instructions = vec![
		create_associated_token_account(&signer, &signer, &pool.mint_0, &pool.token_program_0),
		create_associated_token_account(&signer, &signer, &pool.mint_1, &pool.token_program_1),
	];
	let (amount_0, amount_1) = if protocol {
		pool.protocol_fees
	} else {
		pool.creator_fees
	};
	if protocol {
		let data = CollectProtocolFeesInstructionData::new(|data| {
			data.maximum_amount0.set(u64::MAX);
			data.maximum_amount1.set(u64::MAX);
		})
		.map_err(|_| CliError::InvalidInstructionData)?;
		instructions.push(
			CollectProtocolFees::new(
				signer,
				pool.amm_config,
				*pool_address,
				pool.vault_0,
				pool.vault_1,
				recipient_0,
				recipient_1,
				pool.token_program_0,
				pool.token_program_1,
			)
			.instruction(data),
		);
	} else {
		let data = CollectCreatorFeesInstructionData::new(|data| {
			data.maximum_amount0.set(u64::MAX);
			data.maximum_amount1.set(u64::MAX);
		})
		.map_err(|_| CliError::InvalidInstructionData)?;
		instructions.push(
			CollectCreatorFees::new(
				signer,
				*pool_address,
				pool.vault_0,
				pool.vault_1,
				recipient_0,
				recipient_1,
				pool.token_program_0,
				pool.token_program_1,
			)
			.instruction(data),
		);
	}
	let signature = context.send(&instructions)?;
	report(
		context,
		if protocol {
			"protocol fees collected"
		} else {
			"creator fees collected"
		},
		signature,
		json!({ "pool": pool_address.to_string(), "amount_0": amount_0, "amount_1": amount_1 }),
	);
	Ok(())
}

fn print_config(context: &Context, config: &ConfigView) {
	let value = json!({
		"address": config.address.to_string(),
		"index": config.index,
		"authority": config.authority.to_string(),
		"pool_creator_authority": (config.pool_creator_authority != Pubkey::default())
			.then(|| config.pool_creator_authority.to_string()),
		"trade_fee_rate": config.trade_fee_rate,
		"protocol_fee_rate": config.protocol_fee_rate,
		"creator_fee_rate": config.creator_fee_rate,
	});
	print_value(context, &value);
}

fn print_pool(context: &Context, pool: &PoolView) {
	let price = if pool.reserve_0 == 0 {
		0.0
	} else {
		pool.reserve_1 as f64 / pool.reserve_0 as f64
	};
	let value = json!({
		"address": pool.address.to_string(),
		"amm_config": pool.amm_config.to_string(),
		"creator": pool.creator.to_string(),
		"mint_0": pool.mint_0.to_string(),
		"mint_1": pool.mint_1.to_string(),
		"lp_mint": pool.lp_mint.to_string(),
		"lp_supply": pool.lp_supply,
		"reserve_0": pool.reserve_0,
		"reserve_1": pool.reserve_1,
		"price_1_per_0_base_units": price,
		"protocol_fees": [pool.protocol_fees.0, pool.protocol_fees.1],
		"creator_fees": [pool.creator_fees.0, pool.creator_fees.1],
		"trade_fee_rate": pool.trade_fee_rate,
		"protocol_fee_rate": pool.protocol_fee_rate,
		"creator_fee_rate": pool.creator_fee_rate,
		"creator_fee_mode": pool.creator_fee_mode,
	});
	print_value(context, &value);
}

fn print_value(context: &Context, value: &serde_json::Value) {
	if context.json {
		println!("{value}");
		return;
	}
	if let serde_json::Value::Object(fields) = value {
		for (key, value) in fields {
			println!("{key}: {value}");
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn swap_instructions_preserve_explicit_limits() {
		use clap::Parser;

		use crate::cli::Cli;

		let pool = Pubkey::new_unique();
		let trader = Pubkey::new_unique();
		let sell = Pubkey::new_unique();
		let route = Route {
			input_token: Pubkey::new_unique(),
			output_token: Pubkey::new_unique(),
			input_vault: Pubkey::new_unique(),
			output_vault: Pubkey::new_unique(),
			input_program: TOKEN_PROGRAM,
			output_program: TOKEN_PROGRAM,
			output_mint: Pubkey::new_unique(),
		};

		for (flags, expected) in [
			(
				["--amount-in", "1000", "--minimum-out", "975"],
				(1000_u64, 975_u64),
			),
			(
				["--amount-out", "1000", "--maximum-in", "1025"],
				(1000, 1025),
			),
		] {
			let cli = Cli::parse_from(
				[
					&[
						"pina-amm",
						"swap",
						"--pool",
						&pool.to_string(),
						"--sell",
						&sell.to_string(),
					][..],
					&flags,
				]
				.concat(),
			);
			let Command::Swap(args) = cli.command else {
				panic!("swap")
			};
			let instruction =
				swap_instruction(&pool, &trader, &route, swap_amount(&args)).expect("instruction");
			// Swap instruction fields follow the unversioned discriminator.
			assert_eq!(&instruction.data[1..9], &expected.0.to_le_bytes());
			assert_eq!(&instruction.data[9..17], &expected.1.to_le_bytes());
		}
	}
}
