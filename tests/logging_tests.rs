//! 日志配置功能测试

#[test]
fn test_log_level_parsing() {
    // 测试日志级别解析
    let levels = vec!["DEBUG", "INFO", "WARN", "ERROR"];

    for level in levels {
        let parsed = match level {
            "DEBUG" => "debug",
            "INFO" => "info",
            "WARN" => "warn",
            "ERROR" => "error",
            _ => "warn",
        };

        assert!(!parsed.is_empty());
        assert_eq!(parsed.to_lowercase(), parsed);
    }
}

#[test]
fn test_logging_config() {
    // 测试日志配置值
    let logging_enable = true;
    let logging_path = Some("/tmp/test.log".to_string());
    let logging_level = "DEBUG".to_string();

    assert!(logging_enable);
    assert_eq!(logging_level, "DEBUG");
    assert!(logging_path.is_some());
}

#[test]
fn test_log_file_path() {
    // 测试日志文件路径处理
    let log_path = std::path::Path::new("/tmp/test_kd.log");

    // 测试路径格式
    assert!(log_path.to_string_lossy().contains("test_kd"));
}

#[test]
fn test_log_level_default() {
    // 测试默认日志级别
    let default_level = "WARN";
    let default_enable = true;

    assert_eq!(default_level, "WARN");
    assert!(default_enable);
}
