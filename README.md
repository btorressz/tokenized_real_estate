# tokenized_real_estate
This is a RWA prototype project I made for a private client.

This project implements a Tokenized Real Estate program on the Solana blockchain using Anchor. Each real estate asset (buildings, land, etc.) is represented as a tokenized asset, where individual tokens represent fractional ownership of the property.

## Features

### Property Initialization:
- Create a new tokenized property with metadata (location, value, off-chain metadata URI).
- Property tokens (SPL tokens) represent fractional ownership.

### Minting Property Shares:
- Mint SPL tokens that represent property shares to an owner’s token account.

### Transferring Property Shares:
- Allows property share (SPL tokens) transfers between accounts.

### Distributing Rent:
- Distribute rent or dividends proportionally to token holders.

### Governance and Proposals:
- Create and vote on governance proposals related to the property.

### Escrow for Buying and Selling Property Shares:
- Facilitates secure escrow transactions for property share sales.

## Program Overview

### Instructions
- **initialize_property**: Initializes a new property with metadata such as location, value, and a URI pointing to off-chain metadata.
- **mint_property_shares**: Mints new SPL tokens representing property shares.
- **transfer_property_shares**: Transfers SPL tokens representing property shares from one account to another.
- **distribute_rent**: Distributes rent proportionally to all property token holders.
- **create_proposal**: Allows users to create governance proposals.
- **vote_on_proposal**: Allows token holders to vote on governance proposals.
- **sell_property_shares**: Puts property shares on sale using an escrow mechanism.
- **buy_property_shares**: Facilitates the purchase of property shares via escrow.

## Accounts

- **PropertyAccount**: Stores the metadata of a property (location, value, token mint, and metadata URI).
- **EscrowAccount**: Stores details about escrow transactions for buying and selling property shares.
- **ProposalAccount**: Stores governance proposals, including votes for and against.

## Events

- **PropertyInitialized**: Emitted when a new property is initialized.
- **TokensMinted**: Emitted when property shares (SPL tokens) are minted.
- **RentDistributed**: Emitted when rent is distributed to token holders.

## Error Handling

- **InvalidMint**: Thrown when the provided mint does not match the expected property mint.
- **InsufficientTokens**: Thrown when there are not enough tokens for a transfer.
- **InvalidSalePrice**: Thrown when the sale price provided does not match the escrow's sale price.
- **DivisionByZero**: Thrown when an attempt is made to divide rent by zero tokens.

  ## Tech Stack
  Rust, Typescript Anchor, Solana, Solana Playground ide

  ## License
  This Project is under the: **MIT License**

    

   
   


