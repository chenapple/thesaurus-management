//! API Key 密钥链存储模块
//!
//! 使用系统密钥链（macOS Keychain / Windows Credential Manager / Linux Secret Service）
//! 安全存储 API Key，避免明文存储在 SQLite 数据库中

use keyring::Entry;

const SERVICE_NAME: &str = "com.chenapple.thesaurus-management";

/// 支持的 API Key 名称
pub const API_KEY_NAMES: [&str; 5] = ["deepseek", "openai", "gemini", "qwen", "__setup_wizard_completed"];

/// 将 API Key 保存到系统密钥链
pub fn set_api_key(key_name: &str, api_key: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, key_name)
        .map_err(|e| format!("创建密钥链条目失败: {}", e))?;

    entry.set_password(api_key)
        .map_err(|e| format!("保存到密钥链失败: {}", e))
}

/// 从系统密钥链获取 API Key
pub fn get_api_key(key_name: &str) -> Result<Option<String>, String> {
    let entry = Entry::new(SERVICE_NAME, key_name)
        .map_err(|e| format!("创建密钥链条目失败: {}", e))?;

    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("从密钥链读取失败: {}", e)),
    }
}

/// 从系统密钥链删除 API Key
pub fn delete_api_key(key_name: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, key_name)
        .map_err(|e| format!("创建密钥链条目失败: {}", e))?;

    match entry.delete_credential() {
        Ok(_) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()), // 不存在也视为成功
        Err(e) => Err(format!("从密钥链删除失败: {}", e)),
    }
}

/// 检查 API Key 是否存在于密钥链
pub fn has_api_key(key_name: &str) -> Result<bool, String> {
    match get_api_key(key_name) {
        Ok(Some(_)) => Ok(true),
        Ok(None) => Ok(false),
        Err(e) => Err(e),
    }
}

/// 从 SQLite 迁移 API Key 到密钥链
/// 返回成功迁移的 key 名称列表
pub fn migrate_from_sqlite(get_setting_fn: impl Fn(&str) -> Result<Option<String>, String>, delete_setting_fn: impl Fn(&str) -> Result<(), String>) -> Result<Vec<String>, String> {
    let mut migrated = Vec::new();

    for key_name in API_KEY_NAMES.iter() {
        // 检查密钥链中是否已存在
        if has_api_key(key_name)? {
            continue;
        }

        // 从 SQLite 获取
        if let Ok(Some(api_key)) = get_setting_fn(key_name) {
            if !api_key.is_empty() {
                // 保存到密钥链
                set_api_key(key_name, &api_key)?;

                // 从 SQLite 删除（可选，保留作为备份也可以）
                // 为安全起见，迁移成功后清除 SQLite 中的明文
                let _ = delete_setting_fn(key_name);

                migrated.push(key_name.to_string());
            }
        }
    }

    Ok(migrated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_operations() {
        let test_key = "__test_key_for_unit_test";
        let test_value = "test_api_key_value_12345";

        // 清理可能存在的旧数据
        let _ = delete_api_key(test_key);

        // 测试不存在
        assert!(!has_api_key(test_key).unwrap());
        assert!(get_api_key(test_key).unwrap().is_none());

        // 测试设置
        set_api_key(test_key, test_value).unwrap();
        assert!(has_api_key(test_key).unwrap());
        assert_eq!(get_api_key(test_key).unwrap().unwrap(), test_value);

        // 测试删除
        delete_api_key(test_key).unwrap();
        assert!(!has_api_key(test_key).unwrap());
    }
}
