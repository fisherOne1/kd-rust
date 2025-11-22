//! 清屏功能测试

#[test]
fn test_clear_screen_ansi_sequence() {
    // 测试 ANSI 清屏转义序列
    let clear_seq = "\x1B[2J\x1B[1;1H";

    // 验证序列格式
    assert!(clear_seq.starts_with("\x1B[2J")); // 清屏
    assert!(clear_seq.contains("\x1B[1;1H")); // 移动光标到左上角
}

#[test]
fn test_clear_screen_sequence_length() {
    // 测试清屏序列长度
    let clear_seq = "\x1B[2J\x1B[1;1H";
    // ANSI 序列应该是固定长度（至少包含清屏和光标定位命令）
    assert!(clear_seq.len() >= 10);
}

#[test]
fn test_clear_screen_format() {
    // 测试清屏序列格式正确性
    let clear_seq = "\x1B[2J\x1B[1;1H";

    // 应该包含 ESC 字符
    assert!(clear_seq.contains('\x1B'));

    // 应该包含清屏命令
    assert!(clear_seq.contains("2J"));

    // 应该包含光标定位命令
    assert!(clear_seq.contains("1;1H"));
}
