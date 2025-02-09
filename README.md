# Contribution Tracker Program

A Solana program built with Anchor that implements a blockchain-based contribution tracking and reward system. This system tracks contributors' efforts on-chain, calculates points based on contribution type and impact, and allocates tokens proportionally.

## Features

- Record and track user contributions on Solana blockchain
- Point-based reward system with different contribution categories
- Monthly token distribution based on points earned
- Fairness mechanism with thresholds and reserve pool
- Admin review system for contributions
- Automated reward distribution

## Contribution Categories & Points

The system tracks the following contribution types with associated point ranges:

| Contribution Type   | Points Range | Description                                             |
| ------------------- | ------------ | ------------------------------------------------------- |
| Bug Fixes           | 1-10         | Minor (1-2), Medium (3-5), Major (6-8), Critical (9-10) |
| Feature Development | 2-10         | Simple (2-4), Medium (5-7), Complex (8-10)              |
| Code Optimization   | 1-8          | Minor (1-2), Medium (3-5), Major (6-8)                  |
| Bug Reporting       | 1-9          | Minor (1-2), Medium (3-5), Critical (6-8), Exploit (9)  |
| Test Contributions  | 2-7          | Basic Testing (2-4), Test Case Development (5-7)        |

## Prerequisites

- Rust (latest stable)
- Solana CLI (latest version)
- Node.js (v14 or later)
- Anchor CLI (v0.30.1)
- Yarn or npm

## Setup

1. Install dependencies:

```bash
yarn install
```

2. Build the program:

```bash
anchor build
```

3. Deploy locally:

```bash
solana-test-validator
anchor deploy
```

4. Run tests:

```bash
anchor test
```

## Program Structure

```bash

contribution-tracker/
├── programs/
│   └── contribution-tracker
    ├── Cargo.toml
    ├── Xargo.toml
    └── src
        ├── constants.rs
        ├── error.rs
        ├── handlers
        │   ├── admin
        │   │   ├── initialize.rs
        │   │   ├── manage_period.rs
        │   │   └── mod.rs
        │   ├── contribution
        │   │   ├── mod.rs
        │   │   ├── record.rs
        │   │   └── review.rs
        │   ├── distribution
        │   │   ├── claim.rs
        │   │   ├── mod.rs
        │   │   └── process_period.rs
        │   └── mod.rs
        ├── lib.rs
        ├── state
        │   ├── contribution.rs
        │   ├── contribution_tracker.rs
        │   ├── contributor.rs
        │   ├── distribution_period.rs
        │   ├── enums.rs
        │   └── mod.rs
        └── utils.rs
├── tests/
│   └── contribution-tracker.ts
├── Anchor.toml
└── package.json

```

## Key Components

### State Accounts

**ContributionTracker:** Global state tracking total points and thresholds
**Contributor:** Individual contributor accounts with points and claims
**Contribution:** Individual contribution records
**DistributionPeriod:** Period-specific tracking and distribution data

### Instructions

1. Admin Instructions:

   - Initialize tracker
   - Start new period
   - Review contributions
   - Process period distribution

2. User Instructions:

   - Record contribution
   - Claim rewards

3. Distribution Logic:

   - Monthly token pool unlocking
   - Proportional distribution based on points
   - Reserve pool management
   - Threshold-based distribution

### Testing

The program includes comprehensive tests covering:

- Program initialization
- Contribution recording
- Admin review process
- Period management
- Reward distribution
- Token handling

## Development Status

Currently implemented:

✅ Basic program structure
✅ State management
✅ Core instructions
✅ Point calculation
✅ Token distribution
✅ Basic tests

## TODO

 - Additional validation
 - More comprehensive tests
 - UI Integration
 - Documentation improvements
