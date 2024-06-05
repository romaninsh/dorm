// CREATE TABLE product (
//     id SERIAL PRIMARY KEY,
//     name VARCHAR(255) NOT NULL,
//     description TEXT,
//     price DECIMAL(10, 2) NOT NULL
// );

use std::sync::Arc;

use rust_decimal::Decimal;

use anyhow::Result;
use dorm::prelude::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct Product {
    id: i32,
    name: String,
    description: Option<String>,
    price: Decimal,
}

pub struct ProductSet {
    table: Table<Postgres>,
}

impl TableDelegate<Postgres> for ProductSet {
    fn table(&self) -> &Table<Postgres> {
        &self.table
    }
}

impl ProductSet {
    pub fn new(ds: Postgres) -> Self {
        let table = Table::new("product", ds)
            .with_field("name")
            .with_field("description")
            .with_field("default_price");
        // .add_field(Field::new("id", Type::Serial).primary())
        // .add_field(Field::new("name", Type::Varchar(255)).not_null())
        // .add_field(Field::new("description", Type::Text))
        // .add_field(Field::new("price", Type::Decimal(10, 2)).not_null());

        Self { table }
    }

    pub fn name(&self) -> Arc<Field> {
        self.table.get_field("name").unwrap()
    }

    pub fn description(&self) -> Arc<Field> {
        self.table.get_field("description").unwrap()
    }

    pub fn price(&self) -> Arc<Field> {
        self.table.get_field("default_price").unwrap()
    }

    async fn map<T, F>(self, mut callback: F) -> Result<Self>
    where
        F: FnMut(T) -> T,
        T: Serialize + DeserializeOwned,
    {
        let data = self.table.get_all_data().await?;
        let new_data = data.into_iter().map(|row| {
            let rec: T = serde_json::from_value(Value::Object(row)).unwrap();
            let modified = callback(rec);
            serde_json::to_value(modified)
                .unwrap()
                .as_object()
                .unwrap()
                .clone()
        });

        // for row in new_data.into_iter() {
        //     let insert_query = self.table.update_query(row);
        //     self.ds.query_execute(&insert_query, row);
        // }

        Ok(self)
    }
}
