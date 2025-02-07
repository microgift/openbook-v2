use super::*;

#[tokio::test]
async fn test_place_cancel_order_remaining() -> Result<(), TransportError> {
    let TestInitialize {
        context,
        collect_fee_admin,
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
        bids,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize::default()).await?;
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

    {
        let bids_data = solana.get_account_boxed::<BookSide>(bids).await;
        assert_eq!(bids_data.roots[0].leaf_count, 1);
    }

    // Add remainings, no event on event_queue
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
            max_quote_lots_including_fees: 10004,

            client_order_id: 0,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![account_0],
        },
    )
    .await
    .unwrap();

    {
        let bids_data = solana.get_account_boxed::<BookSide>(bids).await;
        assert_eq!(bids_data.roots[0].leaf_count, 0);
    }

    {
        let open_orders_account_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;

        assert_eq!(open_orders_account_0.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_0.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_0.position.base_free_native, 100);
        assert_eq!(open_orders_account_1.position.base_free_native, 0);
        assert_eq!(open_orders_account_0.position.quote_free_native, 20);
        assert_eq!(open_orders_account_1.position.quote_free_native, 99960);
    }

    // No events on event_queue
    {
        let market_acc = solana.get_account::<Market>(market).await;
        let event_queue = solana
            .get_account::<EventQueue>(market_acc.event_queue)
            .await;

        assert_eq!(event_queue.header.count(), 0);
    }

    // Order with expiry time of 10s
    let now_ts: u64 = solana.get_clock().await.unix_timestamp as u64;
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

            client_order_id: 35,
            expiry_timestamp: now_ts + 10,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();
    {
        let bids_data = solana.get_account_boxed::<BookSide>(bids).await;
        assert_eq!(bids_data.roots[0].leaf_count, 1);
    }

    // Advance clock
    solana.advance_clock(11).await;

    {
        let open_orders_account_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;
        assert_eq!(open_orders_account_0.position.bids_base_lots, 1);
    }

    // Add remainings, no event on event_queue. previous order is canceled
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

            client_order_id: 36,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![account_0],
        },
    )
    .await
    .unwrap();
    // bid has been canceled
    {
        let bids_data = solana.get_account_boxed::<BookSide>(bids).await;
        assert_eq!(bids_data.roots[0].leaf_count, 0);
        let open_orders_account_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;
        assert_eq!(open_orders_account_0.position.bids_base_lots, 0);
    }

    // No events on event_queue
    // {
    //     let market_acc = solana.get_account::<Market>(market).await;
    //     let event_queue = solana
    //         .get_account::<EventQueue>(market_acc.event_queue)
    //         .await;

    //     assert_eq!(event_queue.header.count(), 0);
    // }

    Ok(())
}

#[tokio::test]
async fn test_cancel_order_yourself() -> Result<(), TransportError> {
    let TestInitialize {
        context,
        collect_fee_admin,
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
        bids,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize::default()).await?;
    let solana = &context.solana.clone();

    // Set the initial oracle price
    set_stub_oracle_price(solana, &tokens[1], collect_fee_admin, 1000.0).await;

    // Order with expiry time of 10s
    let now_ts: u64 = solana.get_clock().await.unix_timestamp as u64;
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
            expiry_timestamp: now_ts + 10,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    {
        let bids_data = solana.get_account_boxed::<BookSide>(bids).await;
        assert_eq!(bids_data.roots[0].leaf_count, 1);
    }

    // Advance clock
    solana.advance_clock(11).await;

    // No remainings, same account, previos bid is canceled
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
            max_quote_lots_including_fees: 10004,

            client_order_id: 0,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![account_0],
        },
    )
    .await
    .unwrap();

    {
        let bids_data = solana.get_account_boxed::<BookSide>(bids).await;
        assert_eq!(bids_data.roots[0].leaf_count, 0);
    }

    Ok(())
}
