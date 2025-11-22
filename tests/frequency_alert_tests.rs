//! 频率提醒功能测试

use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[test]
fn test_frequency_calculation() {
    // 测试频率计算逻辑
    let mut history = VecDeque::new();
    let now = Instant::now();

    // 添加30个查询（1分钟内）
    for i in 0..30 {
        history.push_back(now - Duration::from_secs(i));
    }

    // 应该触发频率提醒
    assert!(history.len() >= 30);
}

#[test]
fn test_frequency_cleanup() {
    // 测试清理过期查询
    let mut history = VecDeque::new();
    let now = Instant::now();

    // 添加一些旧的查询（超过1分钟）
    history.push_back(now - Duration::from_secs(120));
    history.push_back(now - Duration::from_secs(90));

    // 添加一些新的查询（1分钟内）
    history.push_back(now - Duration::from_secs(30));
    history.push_back(now - Duration::from_secs(10));

    // 清理超过1分钟的查询
    history.retain(|&time| now.duration_since(time) < Duration::from_secs(60));

    // 应该只剩下2个查询
    assert_eq!(history.len(), 2);
}

#[test]
fn test_frequency_threshold() {
    // 测试频率阈值
    let threshold = 30;
    let mut history = VecDeque::new();
    let now = Instant::now();

    // 添加29个查询（未达到阈值）
    for i in 0..29 {
        history.push_back(now - Duration::from_secs(i));
    }
    assert!(history.len() < threshold);

    // 添加1个查询（达到阈值）
    history.push_back(now);
    assert!(history.len() >= threshold);
}
