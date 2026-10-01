use md5;
use std::collections::HashMap;

/// B站固定的字符重排表
const MIXIN_KEY_ENC_TAB: [usize; 64] = [
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35,
    27, 43, 5, 49, 33, 9, 42, 19, 29, 28, 14, 39, 12, 38, 41, 13,
    37, 48, 7, 16, 24, 55, 40, 61, 26, 17, 0, 1, 60, 51, 30, 4,
    22, 25, 54, 21, 56, 59, 6, 63, 57, 62, 11, 36, 20, 34, 44, 52,
];

/// 从 img_key 和 sub_key 生成 mixin_key（取前32位）
fn get_mixin_key(img_key: &str, sub_key: &str) -> String {
    let raw = format!("{}{}", img_key, sub_key);
    let chars: Vec<char> = raw.chars().collect();
    MIXIN_KEY_ENC_TAB
        .iter()
        .take(32)
        .map(|&i| chars.get(i).copied().unwrap_or(' '))
        .collect()
}

/// 过滤 URL 编码中的特殊字符（B站要求）
/// 保留：字母、数字、-、_、.
fn filter_value(value: &str) -> String {
    value
        .chars()
        .filter(|c| !matches!(c, '!' | '\'' | '(' | ')' | '*'))
        .collect()
}

/// 生成 WBI 签名后的查询字符串
pub fn wbi_sign(
    mut params: HashMap<String, String>,
    img_key: &str,
    sub_key: &str,
) -> String {
    let mixin_key = get_mixin_key(img_key, sub_key);

    // 加入时间戳
    let wts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();
    params.insert("wts".to_string(), wts);

    // 按 key 排序
    let mut sorted: Vec<(String, String)> = params.into_iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));

    // 拼接 query string（值先做过滤，再做 URL 编码）
    let query = sorted
        .iter()
        .map(|(k, v)| {
            let filtered = filter_value(v);
            format!("{}={}", k, urlencoding::encode(&filtered))
        })
        .collect::<Vec<_>>()
        .join("&");

    // 计算 md5(query + mixin_key)
    let digest = md5::compute(format!("{}{}", query, mixin_key).as_bytes());
    let w_rid = format!("{:x}", digest);

    // 拼接最终结果
    format!("{}&w_rid={}", query, w_rid)
}