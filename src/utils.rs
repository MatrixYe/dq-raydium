use std::fs::File;
use std::io::Write;

use serde_json::Value;

/// @Name utils
///
/// @Date 2024/5/9 上午10:42
///
/// @Author Matrix.Ye



/// ### 写入json字符串到文件中
pub fn write_json_to_file(json_str: &str, file_name: &str) -> Result<(), std::io::Error> {
    // 将字符串解析为JSON，确保它是有效的JSON格式
    let json: Value = serde_json::from_str(json_str)?;

    // 打开一个文件来写入
    let mut file = File::create(file_name)?;

    // 将JSON转换为字符串（美化输出）
    let pretty_json = serde_json::to_string_pretty(&json)?;

    // 写入文件
    file.write_all(pretty_json.as_bytes())?;

    Ok(())
}