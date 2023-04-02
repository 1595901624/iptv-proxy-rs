use std::convert::Infallible;
use std::time::Duration;
use reqwest::header::HeaderMap;
use warp::Filter;
use warp::http::{Response, Uri};

#[tokio::main]
async fn main() {
    let cors = warp::cors().allow_any_origin();
    let url = "https://live.v1.mk/api/bestv.php?id=cctv1hd8m/8000000";

    let hello = warp::path!("hello_1.m3u8").and_then(move || {
        get_content_async()
    }).with(cors);
    warp::serve(hello).run(([127, 0, 0, 1], 3030)).await;
}

fn get_default_http_client() -> reqwest::Client {
    let headers = HeaderMap::new();
    return reqwest::Client::builder().default_headers(headers).connect_timeout(Duration::from_secs(8)).build().unwrap();
}
//
// async fn get_content() -> String {
//     let url = "https://live.v1.mk/api/bestv.php?id=cctv1hd8m/8000000";
//     // let rt = tokio::runtime::Runtime::new().unwrap();
//     // rt.block_on(async {
//     let client = get_default_http_client();
//     let result = client.get(url).send().await;
//     let content = result.unwrap().text().await;
//     content.unwrap()
//     // })
// }

async fn get_content_async() -> Result<impl warp::Reply, Infallible> {
    let url = "https://live.v1.mk/api/bestv.php?id=cctv1hd8m/8000000";
    // let url = "http://39.135.138.58:18890/PLTV/88888888/224/3221225918/index.m3u8";
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

