
# GFX SSL v2 CLI

This command-line interface provides a means of:
- Constructing base58 transaction messages for permissioned program admin instructions.
- Executing non-permissioned instructions.
- Introspecting on on-chain state.
- Deriving PDAs for various account types.

Its usage is thoroughly documented in help text:
```
gfx-ssl --help
```

```
gfx-ssl <subcommand> --help
```


## Build
For a standard debug build for local development
(including subcommands for `test-instructions` program),
you can just use a cargo-make command from the root of the repo:

```
cargo make build_cli
```

For a devnet build that includes subcommands for `test-instructions`:
```
cargo build -p gfx-ssl-v2-cli --release --features test-instructions
```

The above command is suitable for mainnet builds as well. If you don't want the `test-instructions` subcommands, just leave out the `--features` arg.

## Example JSON data
Some subcommands require paths to JSON files with configuration state. This allows for easy versioning and for others to procure desired configuration state and delegate the CLI execution to another developer.

Some numerical values are integers in basis-points. See the subsection below for explicit clarification.

### Create SSL
```
{
    "mint": "BNWkCAoNdXmG6Z5jnscA64fjgpu9WSHdkhf7Nc6X6SPM",
    "asset_type": "stable",
    "oracle": "DnFQ5xHiuPk5TQW8KqnGBQ9d8UEcgs9FgNKrAko4Uf7p",
    "oracle_type": "pyth",
    "initial_deposit": 1,
    "max_slot_price_staleness": 1,
    "number_of_slots_throttle": 1,
    "math_params": {
        "mean_window": 21,
        "std_window": 106,
        "max_pool_token_ratio": 1038,
        "std_weight": 5194,
        "fixed_price_distance": 5,
        "minimum_price_distance": 2,
        "latest_price_weight": 9143,
        "fee": 4
    }
 }
```

- `number-of-slots-throttle` controls the minimum difference in number of slots between the latest recorded historical price and the next indexed price. It is nullable, see program code for its default value.
- `max-slot-price-staleness` controls the maximum age of the oracle's latest price before swaps become rejected on the basis of a stale price history. It is nullable, see program code for its default value.
- `oracle_type` is either `"pyth"` or `"switchboard"`
- `asset_type` is either `"stable", `"blue-chip"`, or `"volatile"`
- `initial_deposit` is an integer native token value, i.e. lamports, satoshis, etc.
- All fields in `math_params` are in basis points.

### Create Pair
The fee destinations are external destinations
(i.e. not accounts that are part of the SSLv2 protocol).
They should of course match their associated mint.

The `fee_bps` is in basis-points.

```
[
    {
        "mint": "BNWkCAoNdXmG6Z5jnscA64fjgpu9WSHdkhf7Nc6X6SPM",
        "fee_destination": "EnJxS7xx9q7q75dEAonDYaTXiayk3wucFKhQAiSh8XSq",
        "fee_bps": 10
    },
    {
        "mint": "6jjKDiFUohqfSk6KofB3xEG46ENASWpSvbaPUX7Tbqgq",
        "fee_destination": "AEYnmWJUEEiNH8vd5s5VvSD1aAXkitF2u7xNniRnXrom",
        "fee_bps": 10
    }
]
```

### Basis-points Values
Basis-points are hundredths of a percent.

Examples:
10,000 == 100%
1,000 = 10%
100 == 1%
10 = 0.1%
1 == 0.01%

