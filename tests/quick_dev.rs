use anyhow::Result;
use world_wonders_api::{PORT, WONDERS_ROUTE};

#[ignore = "Only used for convenient development"]
#[tokio::test]
async fn quick_dev() -> Result<()> {
    let hc = httpc_test::new_client(format!("http://localhost:{PORT}"))?;

    hc.do_get(WONDERS_ROUTE).await?.print().await?;
    // hc.do_get(&format!("{WONDERS_ROUTE}/count?category=SevenWonders"))
    //     .await?
    //     .print()
    //     .await?;
    // hc.do_get(&format!("{WONDERS_ROUTE}/name/colosseumm"))
    //     .await?
    //     .print()
    //     .await?;
    // hc.do_get(&format!("{WONDERS_ROUTE}?location=Rome"))
    //     .await?
    //     .print()
    //     .await?;
    // hc.do_get(&format!("{WONDERS_ROUTE}/random"))
    //     .await?
    //     .print()
    //     .await?;
    // hc.do_get(&format!("{WONDERS_ROUTE}/oldest"))
    //     .await?
    //     .print()
    //     .await?;
    // hc.do_get(&format!("{WONDERS_ROUTE}/categories"))
    //     .await?
    //     .print()
    //     .await?;

    Ok(())
}
