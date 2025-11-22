use serde::{Deserialize, Serialize};

// 词典查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub query: String,
    pub found: bool,
    pub is_long_text: bool,
    pub pronunciation: Option<String>,
    pub pronunciation_us: Option<String>, // 美式音标
    pub pronunciation_uk: Option<String>, // 英式音标
    pub translations: Vec<String>,
    pub examples: Vec<(String, String)>,        // (原文, 译文)
    pub collins_items: Vec<CollinsDisplayItem>, // Collins 词典条目
    pub collins_rank: Option<String>,           // 等级标识 (CET4 TEM4)
    pub source: QuerySource,
    pub cached_at: Option<i64>,
}

// Collins 词典显示条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollinsDisplayItem {
    pub additional: Option<String>,      // 如 [套语], (N-COUNT)
    pub major_trans: Option<String>,     // 主要翻译
    pub examples: Vec<(String, String)>, // 例句
}

impl QueryResult {
    pub fn new(query: String, is_long_text: bool) -> Self {
        Self {
            query,
            found: false,
            is_long_text,
            pronunciation: None,
            pronunciation_us: None,
            pronunciation_uk: None,
            translations: Vec::new(),
            examples: Vec::new(),
            collins_items: Vec::new(),
            collins_rank: None,
            source: QuerySource::Online(OnlineSource::Youdao), // Default source
            cached_at: None,
        }
    }
}

// 查询源枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QuerySource {
    OfflineDb,
    LocalCache,
    Online(OnlineSource),
}

// 在线查询源
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OnlineSource {
    Youdao,
    Bing,
    Google,
}

// 压缩缓存数据结构 (用于存储，保留用于未来优化)
#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct CompressedCache {
    pub data: Vec<u8>, // zstd压缩的数据
    pub compressed_size: usize,
    pub original_size: usize,
}
