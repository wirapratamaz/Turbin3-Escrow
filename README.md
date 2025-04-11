# Solana Escrow Program from Turbin3

## Overview
This Escrow Program is a smart contract built on Solana using Anchor. It facilitates secure token swaps between users without requiring direct trust. The escrow holds tokens from a maker until a taker fulfills the exchange conditions, at which point the tokens are transferred accordingly. If the taker does not fulfill the agreement, the maker can withdraw the tokens.

## Directory Structure
```
└── Turbin3-Escrow/
    ├── README.md
    ├── Anchor.toml
    ├── Cargo.toml
    ├── package.json
    ├── tsconfig.json
    ├── .prettierignore
    ├── migrations/
    │   └── deploy.ts
    ├── programs/
    │   └── escrow/
    │       ├── Cargo.toml
    │       ├── Xargo.toml
    │       └── src/
    │           ├── lib.rs
    │           ├── instructions/
    │           │   ├── make.rs
    │           │   ├── mod.rs
    │           │   ├── refund.rs
    │           │   └── take.rs
    │           └── state/
    │               ├── escrow.rs
    │               └── mod.rs
    └── tests/
        └── escrow.ts
```

## Key Features

1. **Secure Token Escrow**
   - Uses Anchor PDAs (Program Derived Addresses) for secure token storage
   - Implements proper authentication checks for all operations

2. **Full Token Swap Functionality**
   - `make`: Create an escrow and deposit tokens
   - `take`: Complete a swap by providing the required tokens
   - `refund`: Allow the maker to withdraw their tokens if the swap hasn't been taken

3. **Optimized for Performance**
   - Uses `Box<>` for large account structures to prevent stack overflow errors
   - Efficiently manages memory to handle complex transactions

4. **Token Standard Support**
   - Compatible with both SPL Token and Token-2022 standards via Token Interface

## Technical Optimizations

1. **Stack Size Management**
   - The program uses Rust's `Box<>` to move large accounts to the heap instead of the stack
   - This prevents the "stack offset exceeded max offset" error that can occur with complex account structures
   - Applied to both instruction contexts and account data to ensure optimal memory usage

2. **Code Structure**
   - Clean separation of concerns with modular design
   - Each instruction (make, take, refund) has its own module
   - State management is separated from instruction logic

### 🧪 Test Results

```
  escrow
Initialize escrow transaction signature 3qxHmpfKWZ9FHpNRkWkoWnF7qqcXW9Wc4GE3bRRD9ZdUNwdSqhFCXiWH2H7pNGCrn8kic1GpvVineTR7bCYqK2JM
Escrow state: {
  seed: <BN: 28d31>,
  maker: PublicKey [PublicKey(CAE9xjcvE66PoXMNvdZJ84476mbFmpcC9ocRKpW2jbtx)] {
    _bn: <BN: a5cda84589751b6a447f54f36f2116a260b3a0afb71e23ccc08cb356b1c2660d>
  },
  mintA: PublicKey [PublicKey(5XqkgK9SgEse7c67L42j1pCHrWENNBpr2JjWfxVsHDo7)] {
    _bn: <BN: 43554c183b246679b666cff9c86e998cf96e9d30ec367dba469dc03a275d3902>
  },
  mintB: PublicKey [PublicKey(CarSt5RwPYMJCt2cJ4URztg82YSEyr7Y1fJebQbb5NAf)] {
    _bn: <BN: ac1ca1c496c3208c19ee843fabaec9adb538604ab4e0b7baa6d538207e06d5e4>
  },
  receiveAmount: <BN: 64>,
  bump: 255
}
    ✔ Initialize escrow (474ms)
Take escrow transaction signature ofnuGfWtggTpHDQqiZkQgzJL23b3txXBaBJeia1w3nxgUBscZC7LvEZD6eLi3eNGArdkZUJVgMVfh4fsUtRQjZj
    ✔ Take escrow (502ms)
Refund escrow transaction signature J2WDCCkBDQUJwuQLC555ZpGQJAURTh5vcL55qXJGn4WCrUe9umcQKdN2vjwD6qJQ77zss169JSBxN3DidvrJMSs
    ✔ Refund escrow (9517ms)


  3 passing (15s)
```

The test results show:
1. **Successful creation** of an escrow with proper state initialization 
2. **Successful token swap** through the take instruction
3. **Successful refund** of tokens to the maker

All three core functions pass their tests, confirming the program works as designed.

## Building and Testing

```bash
# Build the program
anchor build

# Run tests
anchor test
```

## Deployment Steps
```bash
# Build & Deploy the Program
anchor build
anchor deploy

# Generate IDL
anchor idl parse --file target/idl/escrow.json
```

## Security Considerations

The program implements several security measures:
- PDAs for secure token custody
- Proper validation of all accounts
- Checks to ensure only authorized parties can access funds
- Constraints to validate token mint addresses and amounts

## Future Improvements

1. **Fee Structure**: Add a small fee for the escrow service
2. **Time-locks**: Implement expiration for escrows that aren't claimed within a certain timeframe
3. **Multi-token Escrows**: Extend the program to support trading multiple tokens at once
4. **Front-end Integration**: Build a web interface to interact with the escrow program

## Conclusion
This Escrow Program provides a robust and trustless way to exchange tokens securely on Solana, leveraging Anchor's powerful framework. 