use tokio_postgres::types::ToSql;

/// A `Renderable` trait for generating SQL queries and their associated parameters
///
/// This trait is designed to allow various types of SQL statements or sub-queries
/// to be dynamically generated, including the capability to handle parameters
/// that need to be passed to the query executor.
///
/// # Examples
///
/// A simplest implementation of `Renderable` is an `Expression` and it should allow
/// you to do this:
///
/// ```rust
///   let a = 3; let b = 2; let c = 3
///
///   let expr2 = expr!("{} + {}", &b, &c)
///   let expr1 = expr!("{} + {}", &a, &expr2);
///
///   c = 10;
///
///   let result = client.query_one(&expr1).await?;  // 3 + (2 + 10) = 15
/// ```
///
/// In this scenario, expressions remain borrowing values of parameters until it is the time
/// to execute the query. `Renderable` trait makes sure that the query and all the nested
/// queries are properly nested.
///
/// The `Renderable` can be optionally associated with a `DataSet`. When durign nesting
/// expressions will cross the boundaries of the `DataSet` - that query will be executed
/// preemptively and will be replaced with the result of the query.
///
/// Next example will execute sqlite query first, insert results into postgres query and
/// query again.
///
/// ```rust
///   let psql_client = get_psql_client();
///   let sqlite_client = get_sqlite_client();
///
///   let cached_users = expr_ds!(sqlite_client, "select id from cached_users");
///   let users = expr_ds!(psql_client, "select * from orders where user_id in ({})", &cached_users);
///
///   let result = users.fetch_all().await?;
/// ```
///
/// While `Expression` is one of the simplest implementations of `Renderable`, there are others:
///
/// ```rust
///   let query = table.get_select_query();
///
///   let result = query = query.fetch_all().await?;
/// ```
///
/// A `Query` can be constructed for an arbitrary query, but it makes more sense to rely on a `Table`,
/// `Union` or another implementation of `ReadableDataSet` to construct a query.
///
/// # NoSQL implementation of Renderable
///
/// Standard types such as String, Vec<String>, or ToSQL are implementing Renderable and can be used
/// as a part of a query, typically resulting in
pub trait Renderable<'a> {
    /// Generates an SQL statement.
    ///
    /// The method should return a complete SQL statement as a `String`. An `offset`
    /// parameter is provided to allow pagination or other offset-based query features.
    ///
    /// # Parameters
    /// - `offset`: The offset value used in the SQL query, typically for pagination.
    ///
    /// # Returns
    /// - Returns a `String` that contains the SQL statement.
    fn render(&self) -> String;

    /// Collects parameters that are used in the SQL query.
    ///
    /// This method should return a `Vec<Box<dyn ToSql + Sync>>`, where each element
    /// is a boxed object that implements both `ToSql` and `Sync`. These are intended
    /// to be used as bind parameters in the query.
    ///
    /// # Returns
    /// - Returns a vector of boxed objects that can be bound to SQL query parameters.
    fn params(&self) -> Vec<Box<dyn ToSql + Sync>>;
}

impl Renderable<'_> for String {
    fn render(&self) -> String {
        format!("'{}'", self.clone().replace("'", "''"))
    }
    fn params(&self) -> Vec<Box<dyn ToSql + Sync>> {
        vec![]
    }
}

// Does not work because sizing
// impl Renderable<'_> for str {
//     fn render(&self) -> String {
//         self.to_string()
//     }
// }
