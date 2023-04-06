# iptv-proxy-rs

<div><img src="https://img.shields.io/badge/latest%20version-v1.1.0-blue.svg?style=flat"></img>
<img src="https://img.shields.io/badge/license-GPL%203.0-brightgreen.svg?style=flat"></img>

## What is iptv-proxy-rs

iptv-proxy-rs: A local proxy service for m3u8 files built by rust. The purpose is to solve the CORS problems that occur
when playing m3u8 files in electron, tari, walis and other frameworks

## Usage

1. Start `iptv-proxy-rs` before the video is played.
2. Precede your m3u8 url with `127.0.0.1:25011/m3u8/`. Suppose the url is `https://test.m3u8`, and the proxy url
   is `http://127.0.0.1:25001/m3u8/https%3A%2F%2Ftest.m3u8` (UrlEncode is required).

### Example

`Windows` + `tauri`:

```rust
fn main() {
    // Your code...

    let mut p = Command::new(r#"iptv_proxy_rs.exe"#).spawn().unwrap();
    p.try_wait().unwrap();

    // Your code...
}
```

Before playing:

```typescript
const playUrl = ref("https://test.m3u8");
const proxyUrl = ref("http://127.0.0.1:25011/m3u8/" + encodeURIComponent(playUrl.value));
```

### License

GPL v3.0
