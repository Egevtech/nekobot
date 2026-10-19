use sqlite;

pub fn init_database() -> sqlite::Connection {
    let connection = sqlite::open("bot.db").expect("Failed to load database");

    let query = include_str!("../extern/sql/create_query.sql");
    connection.execute(query).expect("Failed to execute init query");

    connection
}