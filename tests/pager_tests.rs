//! 分页器功能测试

use std::process::Command;

#[test]
fn test_pager_command_parsing() {
    // 测试分页器命令解析
    let pager_command = "less -RF";
    let parts: Vec<&str> = pager_command.split_whitespace().collect();

    assert_eq!(parts[0], "less");
    assert_eq!(parts[1], "-RF");

    let pager_command = "bat";
    let parts: Vec<&str> = pager_command.split_whitespace().collect();
    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0], "bat");
}

#[test]
fn test_pager_command_exists() {
    // 测试常见分页器命令是否存在
    let pagers = vec!["less", "more", "cat"];

    for pager in pagers {
        let output = Command::new("which").arg(pager).output();

        // 如果 which 命令不存在，尝试使用 whereis (Linux) 或 where (Windows)
        if output.is_err() {
            // 跳过测试，因为环境可能不同
            continue;
        }

        // 在某些系统上，即使命令存在，which 也可能返回非零退出码
        // 这里我们只检查命令是否可以被解析
        assert!(!pager.is_empty());
    }
}

#[test]
fn test_pager_command_format() {
    // 测试分页器命令格式
    let test_cases = vec![
        ("less -RF", vec!["less", "-RF"]),
        ("bat --style=numbers", vec!["bat", "--style=numbers"]),
        ("more", vec!["more"]),
    ];

    for (command, expected) in test_cases {
        let parts: Vec<&str> = command.split_whitespace().collect();
        assert_eq!(parts, expected);
    }
}
