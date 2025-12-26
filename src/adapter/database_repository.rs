use std::{
    collections::HashMap, env, error::Error, fs::File, io::{BufReader, BufWriter, Write}, path::Path
};

use serde::{Deserialize, Serialize};

const FILE_NAME: &str = "bot_db.json";
const FILE_DB_PATH_ENV: &str = "FILE_DB_PATH";

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct BotMemoryEntity {
    pub user_map: HashMap<String, UserEntity>,
    pub administrators: Vec<String>,
    pub managed_chat_id: Option<String>,
}

#[derive(Clone, Default)]
pub struct BotDatabase {
    pub bot_memory: BotMemoryEntity,
}

impl BotDatabase {
    pub fn try_init() -> Self {
        let file_path = env::var(FILE_DB_PATH_ENV).unwrap_or_else(|_| "./".to_string());

        let result: Result<BotMemoryEntity, Box<dyn Error>> = {
            let file_rs = File::open(format!("{}{}", &file_path, FILE_NAME));
            match file_rs {
                Ok(file) => {
                    let reader = BufReader::new(file);
                    let bot_memory = serde_json::from_reader(reader);
                    if let Err(e) = bot_memory {
                        panic!("Could not parse file cause: {}", e);
                    }
                    Ok(bot_memory.unwrap())
                }
                Err(e) => Err(format!("Could not open file cause: {}", e).into()),
            }
        };

        if let Ok(rs) = result {
            return Self { bot_memory: rs };
        }

        let bot_mem = BotMemoryEntity {
            user_map: HashMap::new(),
            administrators: Vec::new(),
            managed_chat_id: None,
        };

        if !Path::new(&format!("{}{}", file_path, FILE_NAME)).exists() {
            let file_rs = File::create(format!("{}{}", file_path, FILE_NAME));
            if let Err(e) = file_rs {
                panic!("Could not create file cause: {}", e);
            }
            let file: File = file_rs.unwrap();
            let mut writer = BufWriter::new(file);
            if let Err(e) = serde_json::to_writer(&mut writer, &bot_mem) {
                panic!("Could not create empty bot memory database cause: {}", e);
            }
            if let Err(e) = writer.flush() {
                panic!("Could not write file cause: {}", e);
            }
        }
        Self {
            bot_memory: bot_mem,
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let file_path = env::var(FILE_DB_PATH_ENV).unwrap_or_else(|_| "./".to_string());
        let file = File::create(format!("{}{}", file_path, FILE_NAME))?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &self.bot_memory)?;
        writer.flush()?;

        Ok(())
    }
}
