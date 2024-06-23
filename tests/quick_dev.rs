use anyhow::Result;
use world_wonders_api::PORT;

#[tokio::test]
async fn quick_dev() -> Result<()> {
    let hc = httpc_test::new_client(format!("http://localhost:{PORT}"))?;

    // hc.do_get("/v0/wonders").await?.print().await?;
    // hc.do_get("/v0/wonders/count?category=SevenWonders")
    //     .await?
    //     .print()
    //     .await?;
    // hc.do_get("/v0/wonders/name/colosseumm")
    //     .await?
    //     .print()
    //     .await?;
    // hc.do_get("/v0/wonders?location=Rome")
    //     .await?
    //     .print()
    //     .await?;
    hc.do_get("/v0/wonders/random?name=").await?.print().await?;
    // hc.do_get("/v0/wonders/oldest").await?.print().await?;
    // hc.do_get("/v0/categories").await?.print().await?;

    Ok(())
}
