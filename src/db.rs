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
}
