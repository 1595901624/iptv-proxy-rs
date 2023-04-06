use std::convert::Infallible;
use std::string::ToString;
use std::time::Duration;
use m3u8_rs::{Playlist};
use m3u8_rs::AlternativeMediaType::Audio;
use reqwest::header::HeaderMap;
use warp::Filter;
use warp::http::{HeaderValue, Response};
use urlencoding::{decode, encode};

const HOST: &str = "http://127.0.0.1";
const PORT: &str = "25011";

#[tokio::main]
async fn main() {
    let cors = warp::cors().allow_any_origin();
    // let url = "https://live.v1.mk/api/bestv.php?id=cctv1hd8m/8000000";
    // let url = "https://live.v1.mk/api/sxg.php?id=CCTV-6H265_4000";

    // proxy m3u8
    let m3u8_proxy_router = warp::path!("m3u8" / String).and_then(move |url: String| {
        let decoded = decode(url.as_str()).expect("UTF-8");
        // dbg!(&decoded);
        get_m3u8_content_async(decoded.to_string())
    }).with(&cors);

    // proxy ts
    let ts_proxy_router = warp::path!("ts" / String).and_then(move |url: String| {
        let decoded = decode(url.as_str()).expect("UTF-8");
        // dbg!(&decoded);
        get_ts_content_async(decoded.to_string())
    }).with(&cors);

    let routers = m3u8_proxy_router.or(ts_proxy_router);
    warp::serve(routers).run(([127, 0, 0, 1], 25011)).await;
}

/// proxy m3u8
async fn get_m3u8_content_async(url: String) -> Result<impl warp::Reply, Infallible> {
    // let url = "https://live.v1.mk/api/bestv.php?id=cctv1hd8m/8000000";
    // let url = "http://39.135.138.58:18890/PLTV/88888888/224/3221225918/index.m3u8";
    // let url = "https://live.v1.mk/api/sxg.php?id=CCTV-6H265_4000";
    let client = get_default_http_client();
    let result = client.get(&url).send().await;
    if result.is_err() {
        return Ok(Response::builder()
            .status(500).body("".to_string()).unwrap());
    }
    let content = result.unwrap().text().await;
    let m3u8;
    if content.is_err() {
        m3u8 = "".to_string();
    } else {
        // m3u8 = content.unwrap();
        m3u8 = process_m3u8(&url, content.unwrap());
    }

    // dbg!(&m3u8);
    let res = Response::builder()
        .status(200)
        // Are you sure about this one? More like "text/plain"?
        .header("Content-Type", "application/vnd.apple.mpegurl")
        .body(m3u8)
        .unwrap();
    return Ok(res);
}

/// process m3u8
fn process_m3u8(m3u8_path: &String, content: String) -> String {
    if content.is_empty() {
        return "".to_string();
    }
    if let Ok(Playlist::MediaPlaylist(mut pl)) = m3u8_rs::parse_playlist_res(content.as_bytes()) {
        pl.segments.iter_mut().for_each(|segment| {
            // http://39.135.138.58:18890/PLTV/88888888/224/3221225918/index.m3u8
            // http%3A%2F%2F113.207.84.197%3A8090%2F__cl%2Fcg%3Alive%2F__c%2Fcctv17HD%2F__op%2Fdefault%2F__f%2Findex.m3u8
            // No process with BANDWIDTH

            let path_prefix = format!("{}:{}/ts/", HOST, PORT);

            if segment.uri.starts_with("http") {
                segment.uri = format!("{}{}", path_prefix, encode(&segment.uri));
            } else {
                if let Some(position) = m3u8_path.rfind("/") {
                    let url = &m3u8_path[..position + 1];
                    let real_url = format!("{}{}", url, &segment.uri);
                    segment.uri = format!("{}{}", path_prefix, encode(&real_url));
                }
            }
        });
        // dbg!(&pl);
        let mut v: Vec<u8> = Vec::new();
        if let Ok(_) = pl.write_to(&mut v) {
            return String::from_utf8(v).unwrap();
        }
    } else if let Ok(Playlist::MasterPlaylist(mut pl)) = m3u8_rs::parse_playlist_res(content.as_bytes()) {
        // println!("{:?}", pl.alternatives);

        let path_prefix = format!("{}:{}/m3u8/", HOST, PORT);

        // process audio
        pl.alternatives.iter_mut().for_each(|mut media| {
            if media.media_type == Audio && media.uri.is_some() {
                if media.uri.as_ref().unwrap().starts_with("http") {
                    media.uri = Some(format!("{}{}", path_prefix, encode(media.uri.as_ref().unwrap())));
                } else {
                    if let Some(position) = m3u8_path.rfind("/") {
                        let url = &m3u8_path[..position + 1];
                        let real_url = format!("{}{}", url, media.uri.as_ref().unwrap());
                        media.uri = Some(format!("{}{}", &path_prefix, encode(&real_url)));
                    }
                }
            }
        });

        // process m3u8 list
        pl.variants.iter_mut().for_each(|mut variant| {
            // let path_prefix = format!("{}:{}/m3u8/", HOST, PORT);
            if variant.uri.starts_with("http") {
                variant.uri = format!("{}{}", path_prefix, encode(&variant.uri));
            } else {
                if let Some(position) = m3u8_path.rfind("/") {
                    let url = &m3u8_path[..position + 1];
                    let real_url = format!("{}{}", url, &variant.uri);
                    variant.uri = format!("{}{}", &path_prefix, encode(&real_url));
                }
            }
        });
        let mut v: Vec<u8> = Vec::new();
        if let Ok(_) = pl.write_to(&mut v) {
            return String::from_utf8(v).unwrap();
        }
    }

    return content;
}

/// proxy ts
async fn get_ts_content_async(ts: String) -> Result<impl warp::Reply, Infallible> {
    // let url = "https://live.v1.mk/api/bestv.php?id=cctv1hd8m/8000000";
    // let url = "http://39.135.138.58:18890/PLTV/88888888/224/3221225918/index.m3u8";
    // let url = "http://test.8ne5i.10.vs.rxip.sc96655.com/live/8ne5i_sccn,CCTV-6H265_hls_pull_4000K/280/085/429.ts";
    let client = get_default_http_client();
    let result = client.get(&ts).send().await;
    if result.is_err() {
        return Ok(Response::builder()
            .status(500)
            .header("Content-Type", "video/mp2t")
            .body(vec![])
            .unwrap());
    }
    let response = result.unwrap();
    let headers_map = response.headers().clone();

    let mut builder = Response::builder();
    let headers = builder.headers_mut().unwrap();
    for (k, v) in headers_map.into_iter() {
        let h = k.unwrap();
        if h != "content-length" {
            headers.insert(h, HeaderValue::from_str(v.to_str().unwrap()).unwrap());
        }
    }
    // let url = Url::parse(&ts).unwrap();
    // headers.insert("Host", HeaderValue::from_str(url.host().unwrap().to_string().as_str()).unwrap());


    let content = response.bytes().await;
    let ts = content.unwrap();

    let res = builder
        .status(200)
        .body(ts.to_vec())
        .unwrap();
    Ok(res)
}


fn get_default_http_client() -> reqwest::Client {
    let headers = HeaderMap::new();
    return reqwest::Client::builder().default_headers(headers).connect_timeout(Duration::from_secs(8)).build().unwrap();
}

