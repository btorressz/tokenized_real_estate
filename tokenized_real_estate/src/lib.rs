use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

//  program ID
declare_id!("HaodcEgNjhjEfZVSDJc1TdWt58e5JUSyuZNENGBjyQve");

#[program]
mod tokenized_real_estate {
    use super::*;

    // Initialize a new property with its metadata and associated token mint
    pub fn initialize_property(
        ctx: Context<InitializeProperty>, 
        location: String, 
        value: u64, 
        metadata_uri: String, // Off-chain metadata storage
        bump: u8
    ) -> Result<()> {
        let property_account = &mut ctx.accounts.property_account;
        property_account.location = location.clone(); // Clone the string to avoid move issues
        property_account.value = value;
        property_account.metadata_uri = metadata_uri.clone(); // Clone the string to avoid move issues
        property_account.mint = ctx.accounts.property_mint.key();
        property_account.bump = bump;

        // Emit event to notify property has been initialized
        emit!(PropertyInitialized {
            property_address: property_account.key(),
            location, // Use the cloned value
            value,
            metadata_uri, // Use the cloned value
        });

        Ok(())
    }

    // Mint tokens representing property shares to an owner
    pub fn mint_property_shares(
        ctx: Context<MintPropertyShares>, 
        amount: u64
    ) -> Result<()> {
        let cpi_accounts = token::MintTo {
            mint: ctx.accounts.property_mint.to_account_info(),
            to: ctx.accounts.owner_token_account.to_account_info(),
            authority: ctx.accounts.property_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::mint_to(cpi_ctx, amount)?;

        // Emit event to notify that tokens have been minted
        emit!(TokensMinted {
            property_address: ctx.accounts.property_account.key(),
            amount,
        });

        Ok(())
    }

    // Transfer property shares (SPL Tokens) between accounts
    pub fn transfer_property_shares(
        ctx: Context<TransferPropertyShares>, 
        amount: u64
    ) -> Result<()> {
        let cpi_accounts = token::Transfer {
            from: ctx.accounts.seller_token_account.to_account_info(),
            to: ctx.accounts.buyer_token_account.to_account_info(),
            authority: ctx.accounts.seller.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }

    // Distribute rent or dividends proportionally to token holders
pub fn distribute_rent<'info>(
    ctx: Context<'_, '_, '_, 'info, DistributeRent<'info>>,
    total_rent: u64
) -> Result<()> {
    let rent_per_token = total_rent
        .checked_div(ctx.accounts.property_mint.supply)
        .ok_or(PropertyError::DivisionByZero)?;

    let property_rent_vault_info: &AccountInfo = ctx.accounts.property_rent_vault.as_ref();
    let property_account_info: &AccountInfo = ctx.accounts.property_account.as_ref();
    let token_program_info: &AccountInfo = ctx.accounts.token_program.as_ref();

    for holder_info in ctx.remaining_accounts.iter() {
        let cpi_accounts = token::Transfer {
            from: property_rent_vault_info.clone(),
            to: holder_info.clone(),
            authority: property_account_info.clone(),
        };

        let cpi_ctx = CpiContext::new(token_program_info.clone(), cpi_accounts);
        token::transfer(cpi_ctx, rent_per_token)?;
    }

    emit!(RentDistributed {
        property_address: ctx.accounts.property_account.key(),
        total_rent,
    });

    Ok(())
}

    // Create a proposal for governance
    pub fn create_proposal(
        ctx: Context<CreateProposal>, 
        proposal_text: String, 
        end_time: i64
    ) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        proposal.proposal_text = proposal_text;
        proposal.votes_for = 0;
        proposal.votes_against = 0;
        proposal.end_time = end_time;

        Ok(())
    }

    // Vote on a proposal
    pub fn vote_on_proposal(
        ctx: Context<VoteOnProposal>, 
        vote_for: bool
    ) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        let voter_tokens = ctx.accounts.voter_token_account.amount;

        // Ensure voting period is still valid
        if Clock::get()?.unix_timestamp > proposal.end_time {
            return err!(PropertyError::ProposalVotingEnded);
        }

        // Tally votes
        if vote_for {
            proposal.votes_for += voter_tokens;
        } else {
            proposal.votes_against += voter_tokens;
        }

        Ok(())
    }

    // Sell property shares via escrow
    pub fn sell_property_shares(
        ctx: Context<SellPropertyShares>, 
        amount: u64, 
        sale_price: u64
    ) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;
        escrow.seller = ctx.accounts.seller.key();
        escrow.amount = amount;
        escrow.sale_price = sale_price;
        escrow.buyer = None;
        escrow.is_active = true;

        // Transfer tokens to escrow
        let cpi_accounts = token::Transfer {
            from: ctx.accounts.seller_token_account.to_account_info(),
            to: ctx.accounts.escrow_token_account.to_account_info(),
            authority: ctx.accounts.seller.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }

    // Buy property shares from escrow
   pub fn buy_property_shares(
    ctx: Context<BuyPropertyShares>,
    sale_price: u64
) -> Result<()> {
    // Check escrow conditions
    require!(ctx.accounts.escrow_account.is_active, PropertyError::EscrowInactive);
    require!(ctx.accounts.escrow_account.sale_price == sale_price, PropertyError::InvalidSalePrice);

    // Transfer funds from buyer to seller
    **ctx.accounts.seller.lamports.borrow_mut() += sale_price;
    **ctx.accounts.buyer.to_account_info().lamports.borrow_mut() -= sale_price;

    // Prepare transfer of tokens
    let escrow_account_info = ctx.accounts.escrow_account.to_account_info();
    let cpi_accounts = token::Transfer {
        from: ctx.accounts.escrow_token_account.to_account_info(),
        to: ctx.accounts.buyer_token_account.to_account_info(),
        authority: escrow_account_info,
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, ctx.accounts.escrow_account.amount)?;

    // Update escrow account
    let escrow = &mut ctx.accounts.escrow_account;
    escrow.buyer = Some(ctx.accounts.buyer.key());
    escrow.is_active = false;

    Ok(())
}
}

// Account Definitions and Structs

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct InitializeProperty<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 32 + 8 + 64 + 32 + 1, // Add space for metadata_uri
        seeds = [b"property", property_mint.key().as_ref()],
        bump
    )]
    pub property_account: Account<'info, PropertyAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub property_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}



#[derive(Accounts)]
pub struct MintPropertyShares<'info> {
    #[account(mut)]
    pub property_account: Account<'info, PropertyAccount>,
    #[account(mut)]
    pub property_mint: Account<'info, Mint>,
    #[account(mut)]
    pub owner_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct TransferPropertyShares<'info> {
    #[account(mut)]
    pub seller_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,
    pub seller: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct DistributeRent<'info> {
    #[account(mut)]
    pub property_account: Account<'info, PropertyAccount>,
    #[account(mut)]
    pub property_rent_vault: Account<'info, TokenAccount>,
    pub property_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}


#[derive(Accounts)]
pub struct CreateProposal<'info> {
    #[account(init, payer = proposer, space = 8 + 64 + 8 + 8 + 8)]
    pub proposal: Account<'info, ProposalAccount>,
    #[account(mut)]
    pub proposer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VoteOnProposal<'info> {
    #[account(mut)]
    pub proposal: Account<'info, ProposalAccount>,
    #[account(mut)]
    pub voter_token_account: Account<'info, TokenAccount>,
    pub voter: Signer<'info>,
}

#[derive(Accounts)]
pub struct SellPropertyShares<'info> {
    #[account(init, payer = seller, space = 8 + 32 + 32 + 8 + 8 + 1)]
    pub escrow_account: Account<'info, EscrowAccount>,
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(mut)]
    pub seller_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub escrow_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BuyPropertyShares<'info> {
    #[account(mut)]
    pub escrow_account: Account<'info, EscrowAccount>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub escrow_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub seller: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

// Structs

#[account]
pub struct PropertyAccount {
    pub location: String,      // Location of the property
    pub value: u64,            // Value of the property
    pub mint: Pubkey,          // Mint address of the property SPL Token
    pub metadata_uri: String,  // Off-chain metadata URI
    pub bump: u8,              // PDA bump
}

#[account]
pub struct ProposalAccount {
    pub proposal_text: String,  // Proposal description
    pub votes_for: u64,         // Votes in favor
    pub votes_against: u64,     // Votes against
    pub end_time: i64,          // Voting end time
}

#[account]
pub struct EscrowAccount {
    pub seller: Pubkey,         // Seller's pubkey
    pub buyer: Option<Pubkey>,  // Buyer's pubkey (if any)
    pub amount: u64,            // Amount of tokens being sold
    pub sale_price: u64,        // Sale price
    pub is_active: bool,        // Is the escrow active
}

// Errors
#[error_code]
pub enum PropertyError {
    #[msg("The provided token mint does not match the expected property mint.")]
    InvalidMint,
    #[msg("Insufficient tokens for transfer.")]
    InsufficientTokens,
    #[msg("Invalid or incomplete property metadata.")]
    InvalidPropertyMetadata,
    #[msg("Proposal voting has ended.")]
    ProposalVotingEnded,
    #[msg("The escrow is not active.")]
    EscrowInactive,
    #[msg("The sale price provided does not match the escrow.")]
    InvalidSalePrice,
    #[msg("Division by zero error.")]
    DivisionByZero,
}

// Events

#[event]
pub struct PropertyInitialized {
    pub property_address: Pubkey,
    pub location: String,
    pub value: u64,
    pub metadata_uri: String,
}

#[event]
pub struct TokensMinted {
    pub property_address: Pubkey,
    pub amount: u64,
}

#[event]
pub struct RentDistributed {
    pub property_address: Pubkey,
    pub total_rent: u64,
}
