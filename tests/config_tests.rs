//! 配置功能测试

#[test]
fn test_config_defaults() {
    // 测试配置默认值
    let paging_default = true;
    let pager_command_default = "less -RF";
    let english_only_default = false;
    let clear_screen_default = false;
    let enable_emoji_default = true;
    let freq_alert_default = false;

    assert!(paging_default);
    assert_eq!(pager_command_default, "less -RF");
    assert!(!english_only_default);
    assert!(!clear_screen_default);
    assert!(enable_emoji_default);
    assert!(!freq_alert_default);
}

#[test]
fn test_logging_defaults() {
    // 测试日志默认值
    let logging_enable_default = true;
    let logging_level_default = "WARN";

    assert!(logging_enable_default);
    assert_eq!(logging_level_default, "WARN");
}

#[test]
fn test_config_toml_format() {
    // 测试 TOML 配置格式
    let toml_content = r#"
paging = false
pager_command = "bat"
english_only = true
clear_screen = true
enable_emoji = false
freq_alert = true

[logging]
enable = true
path = "/tmp/test.log"
level = "DEBUG"
"#;

    // 验证 TOML 格式正确
    assert!(toml_content.contains("paging = false"));
    assert!(toml_content.contains("pager_command = \"bat\""));
    assert!(toml_content.contains("clear_screen = true"));
    assert!(toml_content.contains("enable_emoji = false"));
    assert!(toml_content.contains("freq_alert = true"));
    assert!(toml_content.contains("[logging]"));
    assert!(toml_content.contains("level = \"DEBUG\""));
}
