use indexmap::IndexMap;
use serde::Serialize;
use serde_json::{to_value, Value};
use std::ops::Deref;
use std::sync::Arc;

use crate::field::Field;
use crate::prelude::AssociatedQuery;
use crate::query::{Query, QueryType};
use crate::table::Table;
use crate::traits::any::RelatedTable;
use crate::traits::column::Column;
use crate::traits::datasource::DataSource;
use crate::traits::entity::Entity;

impl<T: DataSource, E: Entity> Table<T, E> {
    pub fn get_empty_query(&self) -> Query {
        let mut query = Query::new().with_table(&self.table_name, self.table_alias.clone());
        for condition in self.conditions.iter() {
            query = query.with_condition(condition.clone());
        }
        for (alias, join) in &self.joins {
            query = query.with_join(join.join_query().clone());
        }
        query
    }

    pub fn get_select_query(&self) -> Query {
        let mut query = self.get_empty_query();
        query = self.add_fields_into_query(query, None);
        query
    }

    pub fn get_select_query_for_fields(
        &self,
        fields: IndexMap<String, Arc<Box<dyn Column>>>,
    ) -> Query {
        let mut query = Query::new().with_table(&self.table_name, self.table_alias.clone());
        for (field_alias, field_val) in fields {
            let field_val = field_val.clone();
            query = query.with_column_arc(field_alias, field_val);
        }
        query
    }

    pub fn get_select_query_for_field_names(&self, field_names: &[&str]) -> Query {
        let mut index_map = IndexMap::new();
        for field_name in field_names {
            let field = self.search_for_field(field_name).unwrap();
            index_map.insert(field_name.to_string(), Arc::new(field));
        }
        self.get_select_query_for_fields(index_map)
    }

    pub fn get_select_query_for_struct<R: Serialize>(&self, default: R) -> Query {
        let json_value = to_value(default).unwrap();

        let field_names = match json_value {
            Value::Object(map) => {
                let field_names = map.keys().cloned().collect::<Vec<String>>();
                field_names
            }
            _ => panic!("Expected argument to be a struct"),
        };

        let i = field_names
            .into_iter()
            .filter_map(|f| self.search_for_field(&f).map(|c| (f, Arc::new(c))));

        let i = i.collect::<IndexMap<_, _>>();

        self.get_select_query_for_fields(i)
    }

    pub fn get_insert_query(&self, values: E) -> Query {
        let mut query = Query::new()
            .with_table(&self.table_name, None)
            .with_type(QueryType::Insert);

        let serde_json::Value::Object(value_map) = serde_json::to_value(values).unwrap() else {
            panic!("Values must be a struct");
        };

        for (field, _) in &self.fields {
            let field_object = Arc::new(Field::new(field.clone(), self.table_alias.clone()));

            if field_object.calculated() {
                continue;
            };

            let Some(value) = value_map.get(field) else {
                continue;
            };

            query = query.with_set_field(field, value.clone());
        }
        query
    }

    pub fn field_query(&self, field: Arc<Field>) -> AssociatedQuery<T> {
        let query = self.get_empty_query().with_column(field.name(), field);
        AssociatedQuery::new(query, self.data_source.clone())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::prelude::*;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    use crate::{expr_arc, mocks::datasource::MockDataSource, prelude::SqlChunk};

    #[derive(Serialize, Deserialize, Clone)]
    struct User {
        name: String,
        surname: String,
    }

    impl Entity for User {}

    #[test]
    fn test_insert_query() {
        let data =
            json!([{ "name": "John", "surname": "Doe"}, { "name": "Jane", "surname": "Doe"}]);
        let db = MockDataSource::new(&data);

        let table = Table::new_with_entity("users", db)
            .with_field("name")
            .with_field("surname");

        let query = table
            .get_insert_query(User {
                name: "John".to_string(),
                surname: "Doe".to_string(),
            })
            .render_chunk()
            .split();

        assert_eq!(
            query.0,
            "INSERT INTO users (name, surname) VALUES ({}, {}) returning id"
        );
        assert_eq!(query.1[0], json!("John"));
        assert_eq!(query.1[1], json!("Doe"));
    }

    #[test]
    fn test_expression_query() {
        let data = json!([]);
        let db = MockDataSource::new(&data);

        let mut orders = Table::new("orders", db.clone())
            .with_field("price")
            .with_field("qty");

        orders.add_expression("total", |t| {
            expr_arc!(
                "{}*{}",
                t.get_field("price").unwrap().clone(),
                t.get_field("qty").unwrap().clone()
            )
            .render_chunk()
        });

        let query = orders.get_select_query().render_chunk().split();

        assert_eq!(query.0, "SELECT price, qty FROM orders");

        #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
        struct ItemLine {
            price: f64,
            qty: i32,
            total: f64,
        }

        let query = orders
            .get_select_query_for_struct(ItemLine::default())
            .render_chunk()
            .split();
        assert_eq!(
            query.0,
            "SELECT price, qty, (price*qty) AS total FROM orders"
        );
    }
}
