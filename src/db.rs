use rusqlite::{params, types::FromSql, Connection, Error, Result, ToSql};

type RusqilteResponse = Result<usize, Error>;

const KEY_COLUMN: &str = "k";
const VALUE_COLUMN: &str = "v";

#[derive(Debug)]
pub struct Table<'a> {
    name: &'a str,
}

#[derive(Debug)]
pub struct DatabaseTable<'a> {
    name: &'a str,
    connection: &'a Connection,
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
    /// use adrodb::Table;
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
    /// This returns a instance of `DatabaseTable` which is used to
    /// perform operations on your newly created table
    ///
    /// Both the `Connection::open` and `create` method return a
    /// [rusqlite](https://docs.rs/rusqlite/latest/rusqlite/)
    /// [`Result`](https://docs.rs/rusqlite/latest/rusqlite/type.Result.html) type.
    ///
    /// ### Example
    ///
    /// ```
    /// use adrodb::Table;
    /// use rusqlite::Connection;
    ///
    /// let connection = Connection::open("./test.sqlite")?;
    /// let users_table = Table::new("users");
    ///
    /// users_table.create(&connection)?;
    /// ```
    pub fn create(&'a self, connection: &'a Connection) -> Result<DatabaseTable, Error> {
        connection.execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS {} (
                {} varchar(255) PRIMARY KEY UNIQUE NOT NULL,
                {}
            )",
                self.name, KEY_COLUMN, VALUE_COLUMN
            ),
            (),
        )?;

        Ok(DatabaseTable::new(&self.name, &connection))
    }

    /// Returns a instance of `DatabaseTable` without running a `CREATE` command
    ///
    /// If you are sure a table has been previously created you can skip
    /// the `create` method to get your `DatabaseTable` instance.
    ///
    /// **DANGER:** if your table doesn't exist, every action performed on the
    /// `DatabaseTable` will fail.
    ///
    /// ### Example
    ///
    /// ```
    /// use adrodb::Table;
    /// use rusqlite::Connection;
    ///
    /// let connection = Connection::open("./test.sqlite")?;
    /// let table = Table::as_existing("users", &connection);
    ///
    /// table.insert(...)
    /// ```
    pub fn existing(name: &'a str, connection: &'a Connection) -> DatabaseTable<'a> {
        DatabaseTable::new(&name, &connection)
    }
}

impl<'a> DatabaseTable<'a> {
    /// Creates a new instance of the Database table with a reference to
    /// a connection
    ///
    /// Operations to interact with the data is tied to this struct to ensure
    /// the user of the API is sure that the table has been created
    /// previously.
    ///
    /// This is a private method with is called from `Table`.
    fn new(name: &'a str, connection: &'a Connection) -> Self {
        DatabaseTable { name, connection }
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
    /// use adrodb::Table;
    /// use rusqlite::Connection;
    ///
    /// let connection = Connection::open("./test.sqlite")?;
    /// let users_table = Table::new("users");
    ///
    /// users_table.create(&connection)?;
    ///
    /// users_table.insert(&connection, "jimmy", "abc@abc.com")?;
    /// ```
    pub fn set<T: ToSql + ?Sized>(&self, key: &str, value: &T) -> RusqilteResponse {
        let result = self.connection.execute(
            &format!(
                "INSERT INTO {} ({}, {}) VALUES(?1, ?2);",
                self.name, KEY_COLUMN, VALUE_COLUMN
            ),
            params![key, value],
        )?;

        Ok(result)
    }

    /// Get a value in the database by key
    ///
    /// Given a key a query will be made to attempt to get a
    /// value from the database of type `T`. Failure could mean
    /// the SQL query failed, casting value or no results were found
    ///
    /// ### Example
    ///
    /// ```
    /// use adrodb::Table;
    /// use rusqlite::Connection;
    ///
    /// let connection = Connection::open("./test.sqlite")?;
    /// let table = Table::existing("users");
    ///
    /// table.insert("jimmy", "abc")?;
    ///
    /// let result = table.get::<String>("jimmy");
    /// ```
    pub fn get<T: FromSql>(&self, key: &str) -> Result<T, Error> {
        let mut statement = self.connection.prepare(&format!(
            "SELECT {} FROM {} WHERE {} = ?1 LIMIT 1",
            VALUE_COLUMN, self.name, KEY_COLUMN
        ))?;
        let value = statement.query_row(params![key], |row| row.get::<usize, T>(0))?;

        Ok(value)
    }

    /// Removes data by the key
    ///
    /// Removes data from the table based on a given key. The request
    /// returns the number of affected rows. If the key is missing
    /// from the table the result will be `Ok(usize)`.
    ///
    /// ### Example
    /// ```
    /// use adrodb::Table;
    /// use rusqlite::Connection;
    ///
    /// let connection = Connection::open("./test.sqlite")?;
    /// let table = Table::existing("users");
    ///
    /// let result = table.remove("jimmy");
    /// ```
    pub fn remove(&self, key: &str) -> RusqilteResponse {
        let result = self.connection.execute(
            &format!("DELETE FROM {} WHERE {} = ?1", self.name, KEY_COLUMN),
            params![key],
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
        let table = table.create(&conn)?;

        let result = table.set("jimmy", "abc@abc.com");
        assert_eq!(true, result.is_ok());

        let mut stmt = conn.prepare(&format!("SELECT * from {}", table.name))?;
        let mut rows = stmt.query([])?;
        let first = rows.next()?;

        assert_eq!(true, first.is_some());
        let key: String = first.unwrap().get(0)?;
        assert_eq!("jimmy".to_owned(), key);

        Ok(())
    }

    #[test]
    fn test_existing_table() {
        let conn = Connection::open_in_memory().unwrap();
        let table = Table::existing("nope", &conn);
        let result = table.set("key", "value");

        assert_eq!(true, result.is_err());

        let table = Table::new("users");
        table.create(&conn).unwrap();
        let table = Table::existing("users", &conn).set("key1", "value");

        assert_eq!(true, table.is_ok());
    }

    #[test]
    fn test_getting_missing_key() {
        let conn = Connection::open_in_memory().unwrap();
        let table = Table::new("users");
        let table = table.create(&conn).unwrap();

        let result = table.get::<String>("jimmy");

        assert_eq!(true, result.is_err());
    }

    #[test]
    fn test_getting_valid_key() {
        let conn = Connection::open_in_memory().unwrap();
        let table = Table::new("users");
        let table = table.create(&conn).unwrap();

        table.set("jimmy", "abc").unwrap();

        let result = table.get::<String>("jimmy");

        assert_eq!(true, result.is_ok());
        assert_eq!("abc", result.unwrap());
    }

    #[test]
    fn test_casting_results() {
        let conn = Connection::open_in_memory().unwrap();
        let table = Table::new("users");
        let table = table.create(&conn).unwrap();

        table.set("jim", "123").unwrap();
        table.set("jimmy", &123).unwrap();
        table.set("bob", &true).unwrap();

        let result = table.get::<i32>("jimmy");
        assert_eq!(123, result.unwrap());

        let result = table.get::<String>("jim");
        assert_eq!("123", result.unwrap());

        let result = table.get::<bool>("bob");
        assert_eq!(true, result.unwrap());
    }

    #[test]
    fn test_removing_by_key() {
        let conn = Connection::open_in_memory().unwrap();
        let table = Table::new("users");
        let db = table.create(&conn).unwrap();
        db.set("jimmy", "bean").unwrap();

        let jimmy = db.get::<String>("jimmy").unwrap();
        assert_eq!(jimmy, "bean");

        db.remove("jimmy").unwrap();

        let jimmy = db.get::<String>("jimmy");
        assert_eq!(true, jimmy.is_err());
    }

    #[test]
    fn test_removing_by_missing_key() {
        let conn = Connection::open_in_memory().unwrap();
        let table = Table::new("users");
        let db = table.create(&conn).unwrap();

        let result = db.remove("unknown");

        assert_eq!(true, result.is_ok());
        assert_eq!(0, result.unwrap());
    }
}
