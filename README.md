# adrodb

**Don't use this for anything serious**

---

adrodb is a really simple structured key-value storage using sqlite because I asked ChatGPT for Rust ideas.

You'll be able to create tables of data to structure your data and store simple values. Keys are treated as the primary key and must be unique.

This package wraps [rusqlite](https://github.com/rusqlite/rusqlite) to do all of its operations. Since it wraps this package you are free to create a in memory or database connection. Allowing you to persist data if required, or simply use it to move data between different processes.

The table is configured to allow any valid value to be `set` and `get`. See the example below for more information.

## Example

```rust
use adrodb::Table;
use rusqlite::{Connection, Result, Error};

fn main() -> Result<(), Error> {
  let conn = Connection::open_in_memory()?;
  let table = Table::new("users");

  // Creates a new Table in the database if it doesn't exist
  let db = table.create(&conn)?;
  
  db.set("beans", "on toast")?;

  let beans = db.get::<String>("beans")?;

  assert_eq!("on toast", beans);
}
```

The value passed to `.set` must impl the [`ToSql`](https://docs.rs/rusqlite/latest/rusqlite/trait.ToSql.html) trait from rusqlite.

The generic used the `.get` method must impl to [`FromSql`](https://docs.rs/rusqlite/latest/rusqlite/types/trait.FromSql.html) type from rusqlite.

You can skip the `Table::new()` and `.create()` method if you know the table has already been created. That will allow you to do:

```rust
use adrodb::Table;
use rusqlite::{Connection, Result, Error};

fn main() -> Result<(), Error> {
  let conn = Connection::open("persistent.db")?;
  let db = Table::existing("users", &conn);
  
  db.set("beans", "on toast")?;

  let beans = db.get::<String>("beans")?;

  assert_eq!("on toast", beans);
}
```

The `Table::existing` method will not do a check if the table does exist however, the `insert` and `get` methods will fail as a result.
