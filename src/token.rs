use chrono::{DateTime, Local};
use jsonwebtoken::{decode, DecodingKey, EncodingKey, TokenData, Validation};
use serde::{Deserialize, Serialize};
use std::ops::Add;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

// 存储当前用户信息
pub(crate) static CURRENT_USER: LazyLock<Arc<Mutex<CurrentUser>>, fn() -> Arc<Mutex<CurrentUser>>> = LazyLock::new(|| {
    return Arc::new(Mutex::new(CurrentUser { user: User::default(), token: "".to_string() }));
});

pub(crate) struct CurrentUser {
    pub(crate) user: User,
    pub(crate) token: String,
}

const KEYS: LazyLock<Keys, fn() -> Keys> = LazyLock::new(|| {
    let secret = std::env::var("JWT_SECRET").unwrap_or("abc".to_string());
    Keys::new(secret.as_bytes())
});

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub dgraph_uid: String,
    pub role: Role,
    // 失效时间，timestamp
    exp: i64,
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: 0,
            name: "".to_string(),
            email: None,
            phone: None,
            dgraph_uid: "".to_string(),
            role: Role::User,
            exp: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    User,
    Admin,
}

const SECOND_TO_EXPIRED: u64 = 60;
fn expire_timestamp() -> i64 {
    Local::now()
        .add(Duration::from_secs(SECOND_TO_EXPIRED))
        .timestamp()
}

async fn expire() -> DateTime<Local> {
    Local::now().add(Duration::from_secs(SECOND_TO_EXPIRED))
}


pub(crate) async fn parse_token(token: &str) -> Result<TokenData<User>, String> {
    let mut validation = Validation::default();
    // 修改leeway=0，让exp校验使用绝对时间，参考Validation.leeway的使用
    validation.leeway = 0;
    decode(token, &KEYS.decoding, &validation).map_err(|_| "token invalid".to_string())
}

struct Keys {
    pub(crate) encoding: EncodingKey,
    pub(crate) decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}



