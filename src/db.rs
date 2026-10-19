use rusqlite::params;

pub struct Database {
    connection: rusqlite::Connection,
}

fn init_database() -> rusqlite::Connection {
    let connection = rusqlite::Connection::open("bot.db").expect("Failed to load database");

    let query = include_str!("../extern/sql/create_query.sql");
    connection.execute(query, ()).expect("Failed to execute init query");

    connection
}

impl Database {
    pub fn new() -> Self {
        Self { 
            connection: init_database()
        }
    }

    /**
    * In telegram bot api (https://core.telegram.org/bots/api)
    * user_id described as 32-52 bits unsigned integer.
    *
    * But SQL does not support u64(which is recommended type) as BIGINT.
    * In SQL BIGINT is signed integer
    */
    pub fn add_user(&self, user_id: u64) -> rusqlite::Result<()> {
        let query = include_str!("../extern/sql/add_user.sql");

        self.connection.execute(query, (user_id as i64, 0))?;

        Ok(())
    }

    pub fn get_connection(&self) -> &rusqlite::Connection {
        &self.connection
    }
}
