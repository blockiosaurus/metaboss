use crate::cache::NewValue;

use super::*;

pub struct UpdateSymbolAllArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub mint_list: Option<String>,
    pub cache_file: Option<String>,
    pub new_symbol: String,
    pub rate_limit: usize,
    pub retries: u8,
}

pub struct UpdateSymbolArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
    pub mint_account: String,
    pub new_symbol: String,
}

pub async fn update_symbol(args: UpdateSymbolArgs) -> Result<Signature, ActionError> {
    let mut current_md = decode_metadata_from_mint(&args.client, args.mint_account.clone())
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

    // Token Metadata UpdateArgs enum.
    let mut update_args = UpdateArgs::default_v1();

    // Update the symbol on the data struct.
    current_md.data.symbol = args.new_symbol.clone();
    if let UpdateArgs::V1 { ref mut data, .. } = update_args {
        *data = Some(current_md.data);
    } else {
        return Err(ActionError::ActionFailed(
            args.mint_account,
            "UpdateArgs enum is not V1!".to_string(),
        ));
    }

    // Metaboss UpdateAssetArgs enum.
    let update_args = UpdateAssetArgs::V1 {
        payer: None,
        authority: &args.keypair,
        mint: args.mint_account.clone(),
        token: None::<String>,
        delegate_record: None::<String>, // Not supported yet in update.
        update_args,
    };

    update_asset(&args.client, update_args)
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))
}

pub struct UpdateSymbolAll {}

#[async_trait]
impl Action for UpdateSymbolAll {
    fn name() -> &'static str {
        "update-symbol-all"
    }

    async fn action(args: RunActionArgs) -> Result<(), ActionError> {
        update_symbol(UpdateSymbolArgs {
            client: args.client.clone(),
            keypair: args.keypair.clone(),
            mint_account: args.mint_account,
            new_symbol: args.new_value,
        })
        .await
        .map(|_| ())
    }
}

pub async fn update_symbol_all(args: UpdateSymbolAllArgs) -> AnyResult<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(args.keypair, solana_opts);

    let mint_list = parse_mint_list(args.mint_list, &args.cache_file)?;

    // We don't support an optional payer for this action currently.
    let payer = None;

    let args = BatchActionArgs {
        client: args.client,
        keypair,
        payer,
        mint_list,
        cache_file: args.cache_file,
        new_value: NewValue::Single(args.new_symbol),
        rate_limit: args.rate_limit,
        retries: args.retries,
    };
    UpdateSymbolAll::run(args).await?;

    Ok(())
}
