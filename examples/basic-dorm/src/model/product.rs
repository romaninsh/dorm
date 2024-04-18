// CREATE TABLE product (
//     id SERIAL PRIMARY KEY,
//     name VARCHAR(255) NOT NULL,
//     description TEXT,
//     price DECIMAL(10, 2) NOT NULL
// );

use rust_decimal::Decimal;

use dorm::prelude::*;

struct Product {
    id: i32,
    name: String,
    description: Option<String>,
    price: Decimal,
}

pub trait FromRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error>
    where
        Self: Sized;
}

pub trait ToRow {
    fn to_row(&self) -> Result<tokio_postgres::Row, tokio_postgres::Error>;
}

impl FromRow for Product {
    fn from_row(row: &tokio_postgres::Row) -> Self {
        Product {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            price: row.get("price"),
        }
    }
}

impl ToRow for Product {
    fn to_row(&self) -> Result<tokio_postgres::Row, tokio_postgres::Error> {
        let mut row = tokio_postgres::Row::default();
        row.insert("id", &self.id);
        row.insert("name", &self.name);
        row.insert("description", &self.description);
        row.insert("price", &self.price);
        Ok(row)
    }
}

struct ProductSet<'a> {
    ds: tokio_postgres::Client,
    table: Table<'a>,
}

impl<'a> ProductSet<'a> {
    pub fn new(ds: tokio_postgres::Client) -> Self {
        let table = Table::new("product", ds)
            .add_field(Field::new("id", Type::Serial).primary())
            .add_field(Field::new("name", Type::Varchar(255)).not_null())
            .add_field(Field::new("description", Type::Text))
            .add_field(Field::new("price", Type::Decimal(10, 2)).not_null());

        Self { ds, table }
    }

    pub fn name(&self) -> &Field {
        self.table.fields().get("name").unwrap()
    }

    pub fn description(&self) -> &Field {
        self.table.fields().get("description").unwrap()
    }

    pub fn price(&self) -> &Field {
        self.table.fields().get("price").unwrap()
    }
}

impl PostgresTableDataSet<Product> for ProductSet {
    fn table(&self) -> &Table {
        &self.table
    }

    fn map<F>(self, f: F) -> self
    where
        F: Fn(&Product) -> Product,
    {
        let query = self.table.get_select_query();
        let new_data = self.ds.query_fetch(&query).map(|rows| {
            rows.iter()
                .map(|row| Product::from_row(row))
                .map(f)
                .map(|rec| rec.to_row())
        });

        // let replace_query = self.table.get_replace_query(new_data);
        // for row in new_data {
        //    self.ds.query_execute(&replace_query, row);
        // }

        // Discard data as we haven't implement REPLACE query yet
        let _ = new_data;
        self
    }
}
