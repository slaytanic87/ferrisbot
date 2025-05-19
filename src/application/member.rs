use std::collections::HashMap;

use log::debug;

#[derive(Clone, Default)]
pub struct UserEntity {
    pub user_id: i64,
    pub username: String,
    pub firstname: String,
    pub last_activity_unix_time: u64,
}

impl UserEntity {
    pub fn new(
        user_id: i64,
        username: &str,
        firstname: &str,
        last_activity_unix_time: u64,
    ) -> Self {
        Self {
            user_id,
            username: username.to_string(),
            firstname: firstname.to_string(),
            last_activity_unix_time,
        }
    }
}

#[derive(Clone, Default)]
pub struct UserManagement {
    user_map: HashMap<String, UserEntity>,
    administrators: Vec<String>,
}

impl UserManagement {
    pub fn new() -> Self {
        Self {
            user_map: HashMap::new(),
            administrators: Vec::new(),
        }
    }

    pub fn add_user(
        &mut self,
        user_id: i64,
        username: &str,
        firstname: &str,
        last_activity_unix_time: u64,
    ) {
        let user_entity = UserEntity::new(user_id, username, firstname, last_activity_unix_time);
        self.user_map.insert(username.to_string(), user_entity);
    }

    pub fn remove_user(&mut self, username: &str) {
        self.user_map.remove(username);
    }

    pub fn update_user_activity(
        &mut self,
        username: &str,
        firstname: &str,
        user_id: i64,
        last_activity_unix_time: u64,
    ) {
        if let Some(user_entity) = self.user_map.get_mut(username) {
            user_entity.last_activity_unix_time = last_activity_unix_time;
            user_entity.firstname = firstname.to_string();
            user_entity.user_id = user_id;
            return;
        }
        self.add_user(user_id, username, firstname, last_activity_unix_time);
    }

    pub fn contains_user(&self, username: &str) -> bool {
        self.user_map.contains_key(username)
    }

    pub fn get_user(&self, username: &str) -> Option<&UserEntity> {
        self.user_map.get(username)
    }

    pub fn register_administrator(&mut self, username: String) {
        debug!("Registering administrator: {}", username);
        self.administrators.push(username);
    }

    pub fn is_administrator(&self, username: &str) -> bool {
        self.administrators.contains(&username.to_string())
    }
}
