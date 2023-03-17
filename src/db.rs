use rusqlite::{params, Connection, Error, Result, ToSql};

type RusqilteResponse = Result<usize, Error>;

const KEY_COLUMN: &str = "k";
const VALUE_COLUMN: &str = "v";

#[derive(Debug)]
pub struct Table<'a> {
    name: &'a str,
}

impl<'a> Table<'a> {
    /// Creates a new Table Struct
    ///
    /// Creates a new instance of the table and allows you to set the name
    /// for the table. This name will be used as the database table name.
    ///
    /// ### Example
    ///
    /// ```
    /// use db::Table;
    ///
    /// let users_table = Table::new("users");
    /// ```
    pub fn new(name: &'a str) -> Self {
        Table { name }
    }

    /// Creates the table in your database
    ///
    /// Given a connection to a database, this function will create
    /// the table if it doesn't already exist with the appropriate
    /// key & value columns. The key column is treated as the primary
    /// key, must be unique and can't be null. The value column remains
    /// unknown for flexibility.
    ///
    /// Both the `Connection::open` and `create` method return a
    /// [rusqlite](https://docs.rs/rusqlite/latest/rusqlite/)
    /// [`Result`](https://docs.rs/rusqlite/latest/rusqlite/type.Result.html) type.
    ///
    /// ### Example
    ///
    /// ```
    /// use db::Table;
    /// use rusqlite::Connection;
    ///
    /// let connection = Connection::open("./test.sqlite")?;
    /// let users_table = Table::new("users");
    ///
    /// users_table.create(&connection)?;
    /// ```
    pub fn create(&self, connection: &Connection) -> RusqilteResponse {
        let result = connection.execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS {} (
                {} varchar(255) PRIMARY KEY UNIQUE NOT NULL,
                {}
            )",
                self.name, KEY_COLUMN, VALUE_COLUMN
            ),
            (),
        )?;

        Ok(result)
    }

    /// Inserts some data into the table
    ///
    /// This will insert a key & value into the given table. The key
    /// is a string slice while the value needs to adhere to the
    /// [ToSql](https://docs.rs/rusqlite/latest/rusqlite/trait.ToSql.html) trait
    /// provided by russqlite.
    ///
    /// ### Example
    ///
    /// ```
    /// use db::Table;
    /// use rusqlite::Connection;
    ///
    /// let connection = Connection::open("./test.sqlite")?;
    /// let users_table = Table::new("users");
    ///
    /// users_table.create(&connection)?;
    ///
    /// users_table.insert(&connection, "jimmy", "abc@abc.com")?;
    /// ```
    pub fn insert<T: ToSql + ?Sized>(
        &self,
        connection: &Connection,
        key: &str,
        value: &T,
    ) -> RusqilteResponse {
        let result = connection.execute(
            &format!(
                "INSERT INTO {} ({}, {}) VALUES(?1, ?2);",
                self.name, KEY_COLUMN, VALUE_COLUMN
            ),
            params![key, value],
        )?;

        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_table_new() {
        let table = Table::new("test");
        assert_eq!(table.name, "test");
    }

    #[test]
    fn test_table_create() -> Result<(), Error> {
        let connection = Connection::open_in_memory()?;
        let table = Table::new("test");

        // Create the table
        let result = table.create(&connection);

        assert_eq!(true, result.is_ok());

        // To test that, we attempt to drop the same table.
        // If it wasn't created, this will return a Err
        let result = connection.execute("DROP TABLE test", ());

        assert_eq!(true, result.is_ok());

        Ok(())
    }

    #[test]
    fn test_table_insert() -> Result<(), Error> {
        let conn = Connection::open_in_memory()?;
        let table = Table::new("test");
        table.create(&conn)?;

        let result = table.insert(&conn, "jimmy", "abc@abc.com");
        assert_eq!(true, result.is_ok());

        let mut stmt = conn.prepare(&format!("SELECT * from {}", table.name))?;
        let mut rows = stmt.query([])?;
        let first = rows.next()?;

        assert_eq!(true, first.is_some());
        let key: String = first.unwrap().get(0)?;
        assert_eq!("jimmy".to_owned(), key);

        Ok(())
    }
}
