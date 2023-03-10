use anchor_lang::prelude::*;

declare_id!("3o5VrciviJWYnB39NNfmsWTNqSa4aooXKUf5AzZMdWXu");

#[program]
pub mod synthius_v0 {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
