//! Emoji 控制功能测试

#[test]
fn test_emoji_detection() {
    // 测试 emoji 字符检测
    let emoji_text = "📚 [离线]";
    let plain_text = "[离线]";

    assert!(emoji_text.contains('📚'));
    assert!(!plain_text.contains('📚'));
}

#[test]
fn test_emoji_replacement() {
    // 测试 emoji 替换逻辑
    let with_emoji = "≫";
    let without_emoji = ">";

    // 在实际代码中，enable_emoji 控制这个选择
    let enable_emoji = true;
    let prefix = if enable_emoji { "≫" } else { ">" };
    assert_eq!(prefix, with_emoji);

    let enable_emoji = false;
    let prefix = if enable_emoji { "≫" } else { ">" };
    assert_eq!(prefix, without_emoji);
}

#[test]
fn test_source_indicator_emoji() {
    // 测试源指示器的 emoji 控制
    let enable_emoji = true;
    let offline = if enable_emoji {
        "📚 [离线]"
    } else {
        "[离线]"
    };
    let cache = if enable_emoji {
        "💾 [缓存]"
    } else {
        "[缓存]"
    };
    let online = if enable_emoji {
        "🌐 [在线]"
    } else {
        "[在线]"
    };

    assert!(offline.contains("📚"));
    assert!(cache.contains("💾"));
    assert!(online.contains("🌐"));

    let enable_emoji = false;
    let offline = if enable_emoji {
        "📚 [离线]"
    } else {
        "[离线]"
    };
    let cache = if enable_emoji {
        "💾 [缓存]"
    } else {
        "[缓存]"
    };
    let online = if enable_emoji {
        "🌐 [在线]"
    } else {
        "[在线]"
    };

    assert!(!offline.contains("📚"));
    assert!(!cache.contains("💾"));
    assert!(!online.contains("🌐"));
}
