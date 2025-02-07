use super::*;

#[tokio::test]
async fn test_inmediate_order() -> Result<(), TransportError> {
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

    let balance_base = solana.token_account_balance(owner_token_0).await;
    let balance_quote = solana.token_account_balance(owner_token_1).await;

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
            max_base_lots: 2,
            max_quote_lots_including_fees: 20000,

            client_order_id: 0,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::ImmediateOrCancel,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    {
        let open_orders_account_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;

        assert_eq!(open_orders_account_0.position.bids_base_lots, 1);
        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_0.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_0.position.base_free_native, 0);
        assert_eq!(open_orders_account_1.position.base_free_native, 0);
        assert_eq!(open_orders_account_0.position.quote_free_native, 0);
        assert_eq!(open_orders_account_1.position.quote_free_native, 99960);
        assert_eq!(
            balance_base - 100,
            solana.token_account_balance(owner_token_0).await
        );
        assert_eq!(
            balance_quote,
            solana.token_account_balance(owner_token_1).await
        );
    }

    send_tx(
        solana,
        ConsumeEventsInstruction {
            consume_events_admin: None,
            market,
            open_orders_accounts: vec![account_0, account_1],
        },
    )
    .await
    .unwrap();

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

    send_tx(
        solana,
        SettleFundsInstruction {
            owner,
            market,
            open_orders_account: account_0,
            base_vault,
            quote_vault,
            token_base_account: owner_token_0,
            token_quote_account: owner_token_1,
            referrer: None,
        },
    )
    .await
    .unwrap();

    {
        let open_orders_account_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;

        assert_eq!(open_orders_account_0.position.base_free_native, 0);
        assert_eq!(open_orders_account_0.position.quote_free_native, 0);
    }

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

    let balance_base = solana.token_account_balance(owner_token_0).await;
    let balance_quote = solana.token_account_balance(owner_token_1).await;

    // There is a bid in the book, post only doesn't do anything since there is a match
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
            order_type: PlaceOrderType::PostOnly,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    {
        let open_orders_account_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;

        assert_eq!(open_orders_account_0.position.bids_base_lots, 1);
        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_0.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_0.position.base_free_native, 0);
        assert_eq!(open_orders_account_1.position.base_free_native, 0);
        assert_eq!(open_orders_account_0.position.quote_free_native, 0);
        assert_eq!(
            balance_base,
            solana.token_account_balance(owner_token_0).await
        );
        assert_eq!(
            balance_quote,
            solana.token_account_balance(owner_token_1).await
        );
    }

    // Change the price, so no matching and order posts
    // let price_lots_2 = price_lots - 100;
    // send_tx(
    //     solana,
    //     PlaceOrderInstruction {
    //         open_orders_account: account_1,
    //         market,
    //         owner,
    //         token_deposit_account: owner_token_0,
    //         base_vault,
    //         quote_vault,
    //         side: Side::Ask,
    //         price_lots: price_lots_2,
    //         max_base_lots: 1,
    //         max_quote_lots_including_fees: 10000,
    //
    //         client_order_id: 0,
    //         expiry_timestamp: 0,
    //         order_type: PlaceOrderType::PostOnly,
    // remainings:vec![],
    //     },
    // )
    // .await
    // .unwrap();

    // {
    //     let open_orders_account_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;
    //     let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;

    //     assert_eq!(open_orders_account_0.position.bids_base_lots, 1);
    //     assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
    //     assert_eq!(open_orders_account_0.position.asks_base_lots, 0);
    //     assert_eq!(open_orders_account_1.position.asks_base_lots, 1);
    //     assert_eq!(open_orders_account_0.position.base_free_native, 0);
    //     assert_eq!(open_orders_account_1.position.base_free_native, 0);
    //     assert_eq!(open_orders_account_0.position.quote_free_native, 0);
    //     assert_eq!(
    //         balance_base,
    //         solana.token_account_balance(owner_token_0).await
    //     );
    //     assert_eq!(
    //         balance_quote,
    //         solana.token_account_balance(owner_token_1).await
    //     );
    // }

    // PostOnlySlide always post on book
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
            max_quote_lots_including_fees: 10040,

            client_order_id: 0,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::PostOnlySlide,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    {
        let open_orders_account_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;

        assert_eq!(open_orders_account_0.position.bids_base_lots, 1);
        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_0.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 1);
        assert_eq!(open_orders_account_0.position.base_free_native, 0);
        assert_eq!(open_orders_account_1.position.base_free_native, 0);
        assert_eq!(open_orders_account_0.position.quote_free_native, 0);
        assert_eq!(
            balance_base - 100,
            solana.token_account_balance(owner_token_0).await
        );
        assert_eq!(
            balance_quote,
            solana.token_account_balance(owner_token_1).await
        );
    }

    Ok(())
}
