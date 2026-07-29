//! 直播流URL解析器 — 基于 streamget-rs

use crate::models::{StreamInfo, VideoQuality};

/// 根据 URL 识别平台并解析直播流信息
pub async fn resolve_stream(
    url: &str,
    quality: &VideoQuality,
    proxy: Option<&str>,
    cookies: Option<&str>,
) -> Result<StreamInfo, String> {
    let url_lower = url.to_lowercase();
    let quality_str = quality.as_streamget_quality();

    if url_lower.contains("douyin.com") {
        resolve_douyin(url, quality_str, proxy, cookies).await
    } else if url_lower.contains("bilibili.com") || url_lower.contains("b23.tv") {
        resolve_bilibili(url, quality_str, proxy, cookies).await
    } else if url_lower.contains("twitch.tv") {
        resolve_twitch(url, quality_str, proxy, cookies).await
    } else if url_lower.contains("youtube.com") || url_lower.contains("youtu.be") {
        resolve_youtube(url, quality_str, proxy, cookies).await
    } else if url_lower.contains("huya.com") {
        resolve_huya(url, quality_str, proxy, cookies).await
    } else if url_lower.contains("kuaishou.com") {
        resolve_kuaishou(url, quality_str, proxy, cookies).await
    } else if url_lower.contains("douyu.com") {
        resolve_douyu(url, quality_str, proxy, cookies).await
    } else if url_lower.contains("tiktok.com") {
        resolve_tiktok(url, quality_str, proxy, cookies).await
    } else if url_lower.contains("xiaohongshu.com") || url_lower.contains("xhslink.com") {
        resolve_rednote(url, quality_str, proxy, cookies).await
    } else {
        // 尝试作为直接M3U8/FLV URL
        resolve_direct_url(url).await
    }
}

/// 从 StreamData 提取录制用 URL（优先 record_url → m3u8_url → flv_url）
fn get_stream_url(data: &streamget::data::StreamData) -> Result<String, String> {
    data.record_url
        .clone()
        .or_else(|| data.m3u8_url.clone())
        .or_else(|| data.flv_url.clone())
        .ok_or_else(|| "未获取到直播流URL".to_string())
}

/// 判断是否为 FLV 流
fn is_flv_stream(stream_url: &str) -> bool {
    let lower = stream_url.to_lowercase();
    lower.contains(".flv") || lower.ends_with("flv")
}

async fn resolve_douyin(
    url: &str,
    quality: &str,
    proxy: Option<&str>,
    cookies: Option<&str>,
) -> Result<StreamInfo, String> {
    use streamget::base::LiveStream;
    use streamget::platforms::DouyinLiveStream;

    let stream = DouyinLiveStream::new(proxy, cookies, Some(1));
    let data = stream
        .fetch_web_stream_data(url, true)
        .await
        .map_err(|e| format!("获取抖音直播信息失败: {}", e))?;
    let result = stream
        .fetch_stream_url(&data, Some(quality))
        .await
        .map_err(|e| format!("解析抖音直播流失败: {}", e))?;

    let stream_url = get_stream_url(&result)?;

    Ok(StreamInfo {
        stream_url: stream_url.clone(),
        platform: "douyin".to_string(),
        anchor_name: result.anchor_name.unwrap_or_default(),
        title: result.title.unwrap_or_default(),
        is_flv: is_flv_stream(&stream_url),
    })
}

async fn resolve_bilibili(
    url: &str,
    quality: &str,
    proxy: Option<&str>,
    cookies: Option<&str>,
) -> Result<StreamInfo, String> {
    use streamget::base::LiveStream;
    use streamget::platforms::BilibiliLiveStream;

    let stream = BilibiliLiveStream::new(proxy, cookies);
    let data = stream
        .fetch_web_stream_data(url, true)
        .await
        .map_err(|e| format!("获取B站直播信息失败: {}", e))?;
    let result = stream
        .fetch_stream_url(&data, Some(quality))
        .await
        .map_err(|e| format!("解析B站直播流失败: {}", e))?;

    let stream_url = get_stream_url(&result)?;

    Ok(StreamInfo {
        stream_url: stream_url.clone(),
        platform: "bilibili".to_string(),
        anchor_name: result.anchor_name.unwrap_or_default(),
        title: result.title.unwrap_or_default(),
        is_flv: false, // B站一般是HLS
    })
}

async fn resolve_twitch(
    url: &str,
    quality: &str,
    proxy: Option<&str>,
    cookies: Option<&str>,
) -> Result<StreamInfo, String> {
    use streamget::base::LiveStream;
    use streamget::platforms::TwitchLiveStream;

    let stream = TwitchLiveStream::new(proxy, cookies, None);
    let data = stream
        .fetch_web_stream_data(url, true)
        .await
        .map_err(|e| format!("获取Twitch直播信息失败: {}", e))?;
    let result = stream
        .fetch_stream_url(&data, Some(quality))
        .await
        .map_err(|e| format!("解析Twitch直播流失败: {}", e))?;

    let stream_url = get_stream_url(&result)?;

    Ok(StreamInfo {
        stream_url: stream_url.clone(),
        platform: "twitch".to_string(),
        anchor_name: result.anchor_name.unwrap_or_default(),
        title: result.title.unwrap_or_default(),
        is_flv: false,
    })
}

async fn resolve_youtube(
    url: &str,
    quality: &str,
    proxy: Option<&str>,
    cookies: Option<&str>,
) -> Result<StreamInfo, String> {
    use streamget::base::LiveStream;
    use streamget::platforms::YoutubeLiveStream;

    let stream = YoutubeLiveStream::new(proxy, cookies);
    let data = stream
        .fetch_web_stream_data(url, true)
        .await
        .map_err(|e| format!("获取YouTube直播信息失败: {}", e))?;
    let result = stream
        .fetch_stream_url(&data, Some(quality))
        .await
        .map_err(|e| format!("解析YouTube直播流失败: {}", e))?;

    let stream_url = get_stream_url(&result)?;

    Ok(StreamInfo {
        stream_url: stream_url.clone(),
        platform: "youtube".to_string(),
        anchor_name: result.anchor_name.unwrap_or_default(),
        title: result.title.unwrap_or_default(),
        is_flv: false,
    })
}

async fn resolve_huya(
    url: &str,
    quality: &str,
    proxy: Option<&str>,
    cookies: Option<&str>,
) -> Result<StreamInfo, String> {
    use streamget::base::LiveStream;
    use streamget::platforms::HuyaLiveStream;

    let stream = HuyaLiveStream::new(proxy, cookies);
    let data = stream
        .fetch_web_stream_data(url, true)
        .await
        .map_err(|e| format!("获取虎牙直播信息失败: {}", e))?;
    let result = stream
        .fetch_stream_url(&data, Some(quality))
        .await
        .map_err(|e| format!("解析虎牙直播流失败: {}", e))?;

    let stream_url = get_stream_url(&result)?;

    Ok(StreamInfo {
        stream_url: stream_url.clone(),
        platform: "huya".to_string(),
        anchor_name: result.anchor_name.unwrap_or_default(),
        title: result.title.unwrap_or_default(),
        is_flv: is_flv_stream(&stream_url),
    })
}

async fn resolve_kuaishou(
    url: &str,
    quality: &str,
    proxy: Option<&str>,
    cookies: Option<&str>,
) -> Result<StreamInfo, String> {
    use streamget::base::LiveStream;
    use streamget::platforms::KwaiLiveStream;

    let stream = KwaiLiveStream::new(proxy, cookies);
    let data = stream
        .fetch_web_stream_data(url, true)
        .await
        .map_err(|e| format!("获取快手直播信息失败: {}", e))?;
    let result = stream
        .fetch_stream_url(&data, Some(quality))
        .await
        .map_err(|e| format!("解析快手直播流失败: {}", e))?;

    let stream_url = get_stream_url(&result)?;

    Ok(StreamInfo {
        stream_url: stream_url.clone(),
        platform: "kuaishou".to_string(),
        anchor_name: result.anchor_name.unwrap_or_default(),
        title: result.title.unwrap_or_default(),
        is_flv: is_flv_stream(&stream_url),
    })
}

async fn resolve_douyu(
    url: &str,
    quality: &str,
    proxy: Option<&str>,
    cookies: Option<&str>,
) -> Result<StreamInfo, String> {
    use streamget::base::LiveStream;
    use streamget::platforms::DouyuLiveStream;

    let stream = DouyuLiveStream::new(proxy, cookies);
    let data = stream
        .fetch_web_stream_data(url, true)
        .await
        .map_err(|e| format!("获取斗鱼直播信息失败: {}", e))?;
    let result = stream
        .fetch_stream_url(&data, Some(quality))
        .await
        .map_err(|e| format!("解析斗鱼直播流失败: {}", e))?;

    let stream_url = get_stream_url(&result)?;

    Ok(StreamInfo {
        stream_url: stream_url.clone(),
        platform: "douyu".to_string(),
        anchor_name: result.anchor_name.unwrap_or_default(),
        title: result.title.unwrap_or_default(),
        is_flv: is_flv_stream(&stream_url),
    })
}

async fn resolve_tiktok(
    url: &str,
    quality: &str,
    proxy: Option<&str>,
    cookies: Option<&str>,
) -> Result<StreamInfo, String> {
    use streamget::base::LiveStream;
    use streamget::platforms::TikTokLiveStream;

    let stream = TikTokLiveStream::new(proxy, cookies, true);
    let data = stream
        .fetch_web_stream_data(url, true)
        .await
        .map_err(|e| format!("获取TikTok直播信息失败: {}", e))?;
    let result = stream
        .fetch_stream_url(&data, Some(quality))
        .await
        .map_err(|e| format!("解析TikTok直播流失败: {}", e))?;

    let stream_url = get_stream_url(&result)?;

    Ok(StreamInfo {
        stream_url: stream_url.clone(),
        platform: "tiktok".to_string(),
        anchor_name: result.anchor_name.unwrap_or_default(),
        title: result.title.unwrap_or_default(),
        is_flv: is_flv_stream(&stream_url),
    })
}

async fn resolve_rednote(
    url: &str,
    quality: &str,
    proxy: Option<&str>,
    cookies: Option<&str>,
) -> Result<StreamInfo, String> {
    use streamget::base::LiveStream;
    use streamget::platforms::RedNoteLiveStream;

    let stream = RedNoteLiveStream::new(proxy, cookies);
    let data = stream
        .fetch_web_stream_data(url, true)
        .await
        .map_err(|e| format!("获取小红书直播信息失败: {}", e))?;
    let result = stream
        .fetch_stream_url(&data, Some(quality))
        .await
        .map_err(|e| format!("解析小红书直播流失败: {}", e))?;

    let stream_url = get_stream_url(&result)?;

    Ok(StreamInfo {
        stream_url: stream_url.clone(),
        platform: "rednote".to_string(),
        anchor_name: result.anchor_name.unwrap_or_default(),
        title: result.title.unwrap_or_default(),
        is_flv: is_flv_stream(&stream_url),
    })
}

/// 处理直接 M3U8/FLV URL（非平台模式）
async fn resolve_direct_url(url: &str) -> Result<StreamInfo, String> {
    Ok(StreamInfo {
        stream_url: url.to_string(),
        platform: "custom".to_string(),
        anchor_name: "Custom Stream".to_string(),
        title: String::new(),
        is_flv: is_flv_stream(url),
    })
}

/// 从 URL 识别平台名称（用于 UI 显示）
pub fn detect_platform(url: &str) -> (&'static str, &'static str) {
    let lower = url.to_lowercase();
    if lower.contains("douyin.com") {
        ("douyin", "抖音直播")
    } else if lower.contains("bilibili.com") || lower.contains("b23.tv") {
        ("bilibili", "哔哩哔哩直播")
    } else if lower.contains("twitch.tv") {
        ("twitch", "Twitch")
    } else if lower.contains("youtube.com") || lower.contains("youtu.be") {
        ("youtube", "YouTube")
    } else if lower.contains("huya.com") {
        ("huya", "虎牙直播")
    } else if lower.contains("kuaishou.com") {
        ("kuaishou", "快手直播")
    } else if lower.contains("douyu.com") {
        ("douyu", "斗鱼直播")
    } else if lower.contains("tiktok.com") {
        ("tiktok", "TikTok")
    } else if lower.contains("xiaohongshu.com") || lower.contains("xhslink.com") {
        ("rednote", "小红书直播")
    } else {
        ("custom", "自定义流")
    }
}
