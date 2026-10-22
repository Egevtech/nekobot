use rusqlite::params;

pub struct Database {
    connection: rusqlite::Connection,
}

fn init_database(path: String) -> rusqlite::Connection {
    let connection = rusqlite::Connection::open(path).expect("Failed to load database");

    let query = include_str!("../extern/sql/create_query.sql");
    connection.execute(query, ()).expect("Failed to execute init query");

    connection
}

impl Database {
    pub fn new(path: String) -> Self {
        Self { 
            connection: init_database(path)
        }
    }

    /**
    * In telegram bot api (https://core.telegram.org/bots/api)
    * user_id described as 32-52 bits unsigned integer.
    *
    * But SQL does not support u64(which is recommended type) as BIGINT.
    * In SQL BIGINT is signed integer
    */
    pub fn add_user(&self, from: teloxide::types::User) -> rusqlite::Result<()> {
        let query = include_str!("../extern/sql/add_user.sql");

        self.connection.execute(query, 
            (
                from.id.0 as i64,
                0,
                super::username_to_string(&from.username),
                from.first_name,
            )
        )?;

        Ok(())
    }

    pub fn is_user_in_database(&self, user_id: u64) -> rusqlite::Result<bool> {

        let query = include_str!("../extern/sql/is_user_exists_by_userid.sql");

        let mut stmt = self.connection.prepare(query)?;

        Ok(
            stmt.query_row(params![user_id as i64], |row| row.get(0))?
        )

    }

    pub fn increase_user_uses(&self, user_id: u64) -> rusqlite::Result<u32> {
        // SELECT use_count FROM users WHERE userid = ?1
        let get_query = include_str!("../extern/sql/get_user_uses.sql");
        
        let mut stmt = self.connection.prepare(get_query)?;
        
        let use_count: u32 = stmt.query_row(params![user_id as i64], |row| row.get::<_, u32>(0))? + 1;

        // UPDATE users SET use_count = ?1 WHERE userid = ?2
        let set_query = include_str!("../extern/sql/set_user_uses.sql");

        self.connection.execute(set_query, params![use_count, user_id as i64])?;

        Ok(use_count)
    }
}
