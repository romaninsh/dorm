# DORM - the Dry Object-Relational Mapper

[![Book](https://github.com/romaninsh/dorm/actions/workflows/book.yaml/badge.svg)](https://romaninsh.github.io/dorm/)

DORM is a busines entity abstraction framework for Rust.

In Enterprise environment, software applications must be easy to maintain and change.
Typical Rust applications require significant effort to maintain and change the logic,
which makes Rust difficult to compete with languages such as Java, C# and Typescript.
Additionally, existing ORM libraries are rigid and do not allow you to decouple your
business logic from your database implementation detail.

DORM offers opinionated abstraction over SQL for a separation between your
physical database and business logic. Such decoupling allows you to change
either without affecting the other.

DORM also introduces great syntax sugar making your Rust code readable and
easy to understand. To achieve this, DORM comes with the following features:

1. [DataSet abstraction](https://romaninsh.github.io/dorm/1-table-and-fields.html) - like a Map, but Rows are stored remotely and only fetched when needed.
2. [Expressions](https://romaninsh.github.io/dorm/2-expressions-and-queries.html) - use a power of SQL without writing SQL.
3. [Query](https://romaninsh.github.io/dorm/2-expressions-and-queries.html#how-query-uses-expressions-) - a structured query-language aware object for any SQL statement.
4. [DataSources](https://romaninsh.github.io/dorm/1-table-and-fields.html#datasource) - a way to abstract away the database implementation.
5. [Table](https://romaninsh.github.io/dorm/1-table-and-fields.html) - your in-rust version of SQL table or a view
6. [Field](https://romaninsh.github.io/dorm/3-fields-and-operations.html) - representing columns or arbitrary expressions in a data set.
7. WIP: Entity modeling - a pattern for you to create your onw business entities.
8. TODO: CRUD operations - serde-compatible insert, update and delete operations.
9. [Joins](https://romaninsh.github.io/dorm/6-joins.html) - combining tables into a single table.
10. WIP: Reference traversal - convert a set of records into a set of related records.
11. WIP: Subqueries - augment a table with expressions based on related tables.

## Current status

DORM currently is in development. See [TODO](TODO.md) for the current status.

## Inspiration

DORM is inspired by Agile Data (from Agile Toolkit):

- `https://www.agiletoolkit.org/data`
- `https://agile-data.readthedocs.io/en/develop/quickstart.html#core-concepts`
