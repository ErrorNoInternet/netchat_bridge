#[derive(Clone)]
pub struct Database {
    database: sled::Db,
}

impl Database {
    pub fn new(database_path: &str) -> Result<Self, String> {
        let sled_database = match sled::open(database_path) {
            Ok(database) => database,
            Err(error) => return Err(format!("unable to open sled database: {error}")),
        };
        Ok(Self {
            database: sled_database,
        })
    }

    pub fn set(&self, key: &str, value: &str) -> Result<(), String> {
        match self.database.insert(key, value) {
            Ok(_) => Ok(()),
            Err(error) => Err(error.to_string()),
        }
    }

    pub fn get(&self, key: &str) -> Result<Option<String>, String> {
        match self.database.get(key) {
            Ok(value) => match value {
                Some(value) => Ok(Some(std::str::from_utf8(&value).unwrap().to_string())),
                None => Ok(None),
            },
            Err(error) => Err(error.to_string()),
        }
    }

    pub fn remove(&self, key: &str) -> Result<(), String> {
        match self.database.remove(key) {
            Ok(_) => Ok(()),
            Err(error) => Err(error.to_string()),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (String, String)> {
        self.database
            .iter()
            .filter(|result| result.is_ok())
            .map(|result| {
                (
                    std::str::from_utf8(&result.clone().unwrap().0)
                        .unwrap()
                        .to_string(),
                    std::str::from_utf8(&result.unwrap().1).unwrap().to_string(),
                )
            })
    }
}
