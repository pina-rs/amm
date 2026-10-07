/**
 * TypeScript client for the [Pina AMM](https://github.com/pina-rs/amm).
 *
 * Everything under `./generated` is rendered from the program's IDL by
 * `pina generate`; regenerate it instead of editing it. Decoders check
 * discriminators and schema versions only, so compare an account's owner with
 * `PINA_AMM_PROGRAM_ADDRESS` before trusting decoded data.
 *
 * @packageDocumentation
 */
export * from "./generated";
