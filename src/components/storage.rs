use std::{
    cell::OnceCell,
    collections::HashMap,
    fmt::Error,
    fs::{self, File},
    path::Path,
    sync::{Arc, LazyLock, Mutex, OnceLock},
};

use anyhow::Result;
use dioxus::{logger::tracing, prelude::*};
use dioxus_leaflet::{Map, MapMarker, MapPosition};
use flarch::{nodeids::U256, tasks::now};
use flmacro::VersionedSerde;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Storage {
    pub messages: HashMap<usize, Vec<Message>>,
    pub private_users: HashMap<U256, UserPrivate>,
    #[serde(skip)]
    path: String,
}

static LOCK: LazyLock<Arc<Mutex<usize>>> = LazyLock::new(|| Arc::new(Mutex::new(0)));

impl Storage {
    pub async fn new(path: &str) -> Result<Self> {
        let a = LOCK.lock().map_err(|_| anyhow::anyhow!("mutex error"))?;
        if let Ok(file) = fs::read_to_string(path) {
            if let Ok(mut s) = serde_json::from_str::<Storage>(&file) {
                s.path = path.to_string();
                return Ok(s);
            }
        }

        Ok(Self {
            messages: HashMap::new(),
            private_users: HashMap::new(),
            path: path.to_string(),
        })
    }

    pub fn add_message(&mut self, user: U256, poi: usize, message: String) -> Result<()> {
        tracing::info!("Adding message {poi}/{message}");
        let msg = Message {
            sender: user,
            poi,
            time: now(),
            message,
        };
        self.messages
            .entry(poi)
            .and_modify(|m| m.push(msg.clone()))
            .or_insert(vec![msg]);
        Ok(())
    }

    pub fn add_user(&mut self, id: U256, name: String) -> Result<()> {
        self.private_users
            .entry(id)
            .and_modify(|u| u.name = name.clone())
            .or_insert_with(|| {
                tracing::info!("Adding user {name}/{id}");
                UserPrivate {
                    name,
                    points: 0,
                    id_private: id,
                }
            });
        Ok(())
    }

    pub fn users(&self) -> Vec<User> {
        self.private_users.iter().map(|(_, u)| u.into()).collect()
    }

    pub fn get_messages(&self, poi: usize) -> Vec<MessageString> {
        self.messages
            .get(&poi)
            .unwrap_or(&vec![])
            .iter()
            .map(|msg| MessageString {
                sender: self
                    .private_users
                    .get(&msg.sender)
                    .map(|user| user.name.clone())
                    .unwrap_or("Unknown".to_string()),
                time: msg.time,
                message: msg.message.clone(),
            })
            .collect()
    }

    async fn save(&mut self) -> Result<()> {
        let a = LOCK.lock().map_err(|_| anyhow::anyhow!("mutex error"))?;
        let path = Path::new(&self.path);
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }

        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }
}

#[derive(VersionedSerde, Debug, Clone)]
pub struct Message {
    pub sender: U256,
    pub poi: usize,
    pub time: i64,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageString {
    pub sender: String,
    pub time: i64,
    pub message: String,
}

#[derive(VersionedSerde, Debug, Clone)]
pub struct UserPrivate {
    name: String,
    points: usize,
    id_private: U256,
}

impl UserPrivate {
    pub fn public(&self) -> U256 {
        U256::hash_domain_parts("user", &[self.id_private.as_ref()])
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub name: String,
    pub points: usize,
    pub id_public: U256,
}

impl From<&UserPrivate> for User {
    fn from(value: &UserPrivate) -> Self {
        User {
            id_public: value.public(),
            name: value.name.clone(),
            points: value.points,
        }
    }
}

async fn with_storage<T, U: FnMut(&Storage) -> T>(mut f: U) -> T {
    let s = Storage::new("./data/storage.json")
        .await
        .expect("Should get storage");
    let t = f(&s);
    t
}

async fn with_storage_mut<T, U: FnMut(&mut Storage) -> T>(mut f: U) -> T {
    let mut s = Storage::new("./data/storage.json")
        .await
        .expect("Should get storage");
    let t = f(&mut s);
    s.save().await;
    t
}

#[server]
pub async fn get_messages(poi: usize) -> Result<Vec<MessageString>, ServerFnError> {
    Ok(with_storage(|s| s.get_messages(poi)).await)
}

#[server]
pub async fn get_users() -> Result<Vec<User>, ServerFnError> {
    Ok(with_storage(|s| s.users()).await)
}

#[server]
pub async fn add_message(user_private: U256, poi: usize, msg: String) -> Result<(), ServerFnError> {
    with_storage_mut(|s| s.add_message(user_private, poi, msg.clone())).await;
    Ok(())
}

#[server]
pub async fn store_user(user_private: U256, name: String) -> Result<(), ServerFnError> {
    with_storage_mut(|s| s.add_user(user_private, name.clone())).await;
    Ok(())
}
