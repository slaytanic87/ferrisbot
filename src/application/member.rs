use std::time::{Duration, SystemTime};

use log::debug;

use crate::adapter::{BotDatabase, UserEntity};

#[derive(Clone, Default)]
pub struct UserManagement {
    pub bot_db: BotDatabase,
}

impl UserManagement {
    pub fn new() -> Self {
        let bot_db = BotDatabase::try_init();
        Self { bot_db }
    }

    pub fn add_user(
        &mut self,
        user_id: i64,
        username: &str,
        firstname: &str,
        last_activity_unix_time: u64,
    ) {
        let user_entity = UserEntity::new(user_id, username, firstname, last_activity_unix_time);
        self.bot_db
            .bot_memory
            .user_map
            .insert(user_id.to_string(), user_entity);
    }

    pub fn get_inactive_users_since(&self, duration: Duration) -> Vec<UserEntity> {
        let current_time: u64 = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.bot_db
            .bot_memory
            .user_map
            .values()
            .filter(|user| current_time - user.last_activity_unix_time > duration.as_secs())
            .cloned()
            .collect()
    }

    pub fn remove_user(&mut self, user_id: i64) {
        self.bot_db.bot_memory.user_map.remove(&user_id.to_string());
    }

    pub fn update_user_activity(
        &mut self,
        username: &str,
        firstname: &str,
        user_id: i64,
        last_activity_unix_time: u64,
    ) {
        if let Some(user_entity) = self
            .bot_db
            .bot_memory
            .user_map
            .get_mut(&user_id.to_string())
        {
            user_entity.last_activity_unix_time = last_activity_unix_time;
            user_entity.username = username.to_string();
            user_entity.firstname = firstname.to_string();
            user_entity.user_id = user_id;
        } else if let Some((index, _)) = self.get_user_by_name(username) {
            self.remove_user(index.parse().unwrap());
            self.add_user(user_id, username, firstname, last_activity_unix_time);
        } else {
            self.add_user(user_id, username, firstname, last_activity_unix_time);
        }
        self.persist();
        debug!("Usermap {:?}", self.bot_db.bot_memory.user_map.values());
    }

    pub fn contains_username(&self, username: &str) -> bool {
        self.bot_db
            .bot_memory
            .user_map
            .iter()
            .any(|(_, user)| user.username == username)
    }

    pub fn get_user_by_name(&self, username: &str) -> Option<(&String, &UserEntity)> {
        self.bot_db
            .bot_memory
            .user_map
            .iter()
            .find_map(|(index, user)| {
                if user.username == username {
                    Some((index, user))
                } else {
                    None
                }
            })
    }

    pub fn register_administrator(&mut self, username: String) {
        debug!("Registering administrator: {}", username);
        self.bot_db.bot_memory.administrators.push(username);
    }

    pub fn determine_user_role(&self, username: &str) -> &str {
        if self
            .bot_db
            .bot_memory
            .administrators
            .contains(&username.to_string())
        {
            "Admin"
        } else {
            "Regular User"
        }
    }

    pub fn is_administrator(&self, username: &str) -> bool {
        self.bot_db
            .bot_memory
            .administrators
            .contains(&username.to_string())
    }

    pub fn clear_administrators(&mut self) {
        self.bot_db.bot_memory.administrators.clear();
    }

    pub fn set_managed_chat_id(&mut self, chat_id: Option<String>) {
        self.bot_db.bot_memory.managed_chat_id = chat_id;
    }

    pub fn persist(&mut self) {
        if let Err(e) = self.bot_db.save() {
            debug!("Could not save cause: {}", e);
        }
    }
}
