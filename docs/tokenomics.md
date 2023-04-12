# Tokenomics [DRAFT]

## Initial Total Supply

`1 Billion GGX`

`1 GGX ~= 0.05 USD`

`18 decimals`

All amounts in this document are in GGX.

## APY

Schedule of APY will be next:

| Date | Yearly APY** |
| -----|-----------------------|
| 2023 |    14.93%             |
| 2024 |    13.93%             |
| ...  |     ...               |
| 2028 |    11.31%             |
| ... |    ...                 |
| 2043| 4% |
| ... | ... |
| 2053 | 2% |
| 2054 | 2% |

** decreases by 6.7% per year

After 30 years the APY stabilizes at 2%

### Current state

* Initial total supply is distributed to active nodes for now.
* 18 decimals is done. (configured by MILLIGGX)
* APY configured to 16%. (configured by runtime/mainnet/src/pos/currency.rs InflationPercent)
* APY decrease ladder scheduled for the runtime.
APY decrease happens every 365.25 days to address leap years. (configured by runtime/mainnet/src/pos/currency.rs InflationPercentDecay)

## Staking

Each new validators when enters `PoS` with `200 thousands` staked for `1 year`.

Staking rewards are distributed each session.

`Session period = 4 hours`
`Election period = quarter`

### Slashing

| Misbehavior | Amount |  Parameters |
| ------------| -------| ------------------
| Consensus Offline               | 0.01% to 44% * Stake | starting from 10% validators offline linear increase |  
| Consensus Equivocation for blocks(double sign) | 0.01% * Stake |  TBD |
| MPC Signature (Wrong message) |  0.01% * Stake | TBD |
| MPC Key generation(Failure to participate) | 0.001% * Stake | TBD |
| Early unsake | 10% * Stake |
| ...TBD... | ...TBD... |

Slashed amounts are sent to treasury.

### Nominations

| Account | Part | Type |
| ------- | ---- | ---- |
| Validator** | 5% | Fixed comission |
| Nominator + Validator** | 95% | Shared between nominators and validator ||

** 10% of validators rewards go to Treasury
|  Account  | Part |
| --------- | ---- |
| Validator | 90%  |
| Treasury  | 10%  |

### Current state

* 1 year withdrawal lock is implemented
* Session period is 4 hours. Era period is 24 hours. Payout at the end of era rather than session.
(configured by EpochDurationInBlocks, SessionsPerEra).
This can be hard to configure using Parity toolchain, cause they use era as payout and validator rotation point.
* Fixed comission is not implemented yet. Currently, validator can set any comission it wants.
* 10% Treasury comission is implemented. (configured by runtime/mainnet/src/pos/currency.rs TreasuryCommission)

## Rewards

### From transaction fees

| Account | Part |
| -------- | ----- |
| Treasury | 25% |
| Validator(block producer) | 75% |
| Burn | 0% |

Fees are distributed each block.

### Current state

* Done

## Fees

| Category | Type | Amount |
|------|--------|-----------|
|Transaction | 1 second of execution(or equivalent prove size)| 10000 |
|Transaction | MPC signatrue | 100 |
|Storage | ED for account | 0.1 |

Proper fees to be defined.

## Parameters configuration

Parameters can be changed by OpenGov.

### Current state

* Implemented most of it.
