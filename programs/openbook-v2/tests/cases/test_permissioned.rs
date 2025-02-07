use super::*;

#[tokio::test]
async fn test_permissioned_open_order() -> Result<(), TransportError> {
    let TestInitialize {
        context,
        collect_fee_admin,
        owner,
        owner_token_1,
        market,
        base_vault,
        quote_vault,
        tokens,
        account_0,
        open_orders_admin,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize {
        open_orders_admin_bool: true,
        ..TestNewMarketInitialize::default()
    })
    .await?;
    let solana = &context.solana.clone();

    let price_lots = {
        let market = solana.get_account::<Market>(market).await;
        market.native_price_to_lot(I80F48::from(1000))
    };

    // Set the initial oracle price
    set_stub_oracle_price(solana, &tokens[1], collect_fee_admin, 1000.0).await;

    // First, send in an order w/o the signature of the open order authority, expect failure
    let result = send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_0,
            open_orders_admin: None,
            market,
            owner,
            token_deposit_account: owner_token_1,
            base_vault,
            quote_vault,
            side: Side::Bid,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10000,
            client_order_id: 0,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await;

    assert!(result.is_err());

    // Second, send in an order w/ the signature of the open order authority, expect success
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_0,
            open_orders_admin: Some(open_orders_admin),
            market,
            owner,
            token_deposit_account: owner_token_1,
            base_vault,
            quote_vault,
            side: Side::Bid,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10000,
            client_order_id: 0,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    Ok(())
}

#[tokio::test]
async fn test_permissioned_open_take_order() -> Result<(), TransportError> {
    let TestInitialize {
        context,
        collect_fee_admin,
        open_orders_admin,
        owner,
        owner_token_1,
        market,
        base_vault,
        quote_vault,
        price_lots,
        tokens,
        account_0,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize {
        open_orders_admin_bool: true,
        ..TestNewMarketInitialize::default()
    })
    .await?;
    let solana = &context.solana.clone();

    // Set the initial oracle price
    set_stub_oracle_price(solana, &tokens[1], collect_fee_admin, 1000.0).await;

    let result = send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_0,
            open_orders_admin: None,
            market,
            owner,
            token_deposit_account: owner_token_1,
            base_vault,
            quote_vault,
            side: Side::Bid,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10000,

            client_order_id: 0,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await;

    assert!(result.is_err());

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_0,
            open_orders_admin: Some(open_orders_admin),
            market,
            owner,
            token_deposit_account: owner_token_1,
            base_vault,
            quote_vault,
            side: Side::Bid,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10000,

            client_order_id: 0,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    Ok(())
}

#[tokio::test]
async fn test_consume_events_admin() -> Result<(), TransportError> {
    let TestInitialize {
        context,
        collect_fee_admin,
        consume_events_admin,
        owner,
        owner_token_0,
        owner_token_1,
        market,
        base_vault,
        quote_vault,
        price_lots,
        tokens,
        account_0,
        account_1,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize {
        consume_events_admin_bool: true,
        ..TestNewMarketInitialize::default()
    })
    .await?;
    let solana = &context.solana.clone();

    // Set the initial oracle price
    set_stub_oracle_price(solana, &tokens[1], collect_fee_admin, 1000.0).await;

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_0,
            open_orders_admin: None,
            market,
            owner,
            token_deposit_account: owner_token_1,
            base_vault,
            quote_vault,
            side: Side::Bid,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10000,

            client_order_id: 0,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            owner,
            token_deposit_account: owner_token_0,
            base_vault,
            quote_vault,
            side: Side::Ask,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10000,

            client_order_id: 0,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    let result = send_tx(
        solana,
        ConsumeEventsInstruction {
            consume_events_admin: None,
            market,
            open_orders_accounts: vec![account_0, account_1],
        },
    )
    .await;

    assert!(result.is_err());

    send_tx(
        solana,
        ConsumeEventsInstruction {
            consume_events_admin: Some(consume_events_admin),
            market,
            open_orders_accounts: vec![account_0, account_1],
        },
    )
    .await
    .unwrap();

    Ok(())
}
