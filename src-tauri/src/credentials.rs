use keyring::Entry;

const SERVICE: &str = "ai-usage-dock";

fn entry(account_id: &str) -> Result<Entry, String> {
    Entry::new(SERVICE, account_id).map_err(|error| format!("无法访问系统凭据库: {error}"))
}

pub fn set_secret(account_id: &str, secret: &str) -> Result<(), String> {
    entry(account_id)?
        .set_password(secret)
        .map_err(|error| format!("无法写入系统凭据库: {error}"))
}

pub fn get_secret(account_id: &str) -> Result<String, String> {
    entry(account_id)?
        .get_password()
        .map_err(|error| format!("无法读取系统凭据库: {error}"))
}

pub fn delete_secret(account_id: &str) -> Result<(), String> {
    match entry(account_id)?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(format!("无法删除系统凭据: {error}")),
    }
}

pub fn has_secret(account_id: &str) -> bool {
    get_secret(account_id).is_ok()
}
