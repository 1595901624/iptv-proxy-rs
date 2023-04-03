use std::convert::Infallible;
use std::time::Duration;
use reqwest::header::HeaderMap;
use warp::Filter;
use warp::http::{Response};
use urlencoding::decode;

#[tokio::main]
async fn main() {
    let cors = warp::cors().allow_any_origin();
    // let url = "https://live.v1.mk/api/bestv.php?id=cctv1hd8m/8000000";
    // let url = "https://live.v1.mk/api/sxg.php?id=CCTV-6H265_4000";

    // proxy m3u8
    let m3u8_proxy_router = warp::path!("m3u8" / String).and_then(move |url: String| {
        let decoded = decode(url.as_str()).expect("UTF-8");
        dbg!(&decoded);
        get_content_async(decoded.to_string())
    });

    // proxy ts
    let ts_proxy_router = warp::path!("ts" / String).and_then(move |url| {
        get_content_async_ts(url)
    });

    let routers = m3u8_proxy_router.or(ts_proxy_router).with(cors);
    warp::serve(routers).run(([127, 0, 0, 1], 25011)).await;
}

async fn get_content_async(url: String) -> Result<impl warp::Reply, Infallible> {
    // let url = "https://live.v1.mk/api/bestv.php?id=cctv1hd8m/8000000";
    // // let url = "http://39.135.138.58:18890/PLTV/88888888/224/3221225918/index.m3u8";
    // let url = "https://live.v1.mk/api/sxg.php?id=CCTV-6H265_4000";
    let client = get_default_http_client();
    let result = client.get(url).send().await;
    let content = result.unwrap().text().await;
    let m3u8 = content.unwrap();
    println!("{}", m3u8);
    let res = Response::builder()
        .status(200)
        // Are you sure about this one? More like "text/plain"?
        .header("Content-Type", "txt")
        .body(m3u8)
        .unwrap();
    Ok(res)
}

async fn get_content_async_ts(ts: String) -> Result<impl warp::Reply, Infallible> {
    // let url = "https://live.v1.mk/api/bestv.php?id=cctv1hd8m/8000000";
    // let url = "http://39.135.138.58:18890/PLTV/88888888/224/3221225918/index.m3u8";
    // let url = "http://test.8ne5i.10.vs.rxip.sc96655.com/live/8ne5i_sccn,CCTV-6H265_hls_pull_4000K/280/085/429.ts";
    let client = get_default_http_client();
    let result = client.get(ts).send().await;
    let response = result.unwrap();
    let content = response.text().await;
    let ts = content.unwrap();
    dbg!(&ts);
    let res = Response::builder()
        .status(200)
        // Are you sure about this one? More like "text/plain"?
        .header("Content-Type", "video/mp2t")
        .body(ts)
        .unwrap();
    Ok(res)
}


fn get_default_http_client() -> reqwest::Client {
    let headers = HeaderMap::new();
    return reqwest::Client::builder().default_headers(headers).connect_timeout(Duration::from_secs(8)).build().unwrap();
}

