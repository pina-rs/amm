# Security policy

The Pina AMM is pre-release software. It has not been independently audited and has not been deployed by this repository. Do not deposit material value until an audit is published.

## Reporting a vulnerability

Please do not open a public issue. Report privately through [GitHub Security Advisories](https://github.com/pina-rs/amm/security/advisories/new) and include:

- the affected commit and component (program, a client, or the CLI);
- the impact and the realistic prerequisites for an attack;
- reproduction steps or a minimal failing test;
- any mitigation you propose.

Only test against accounts and funds you own. We will acknowledge a complete report, assess its severity, prepare a regression test and a fix, and coordinate disclosure once users have had a reasonable window to upgrade.

The threat model, invariants, and the tests that enforce them are in [docs/security.md](docs/security.md).
