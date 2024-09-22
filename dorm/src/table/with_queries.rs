use indexmap::IndexMap;
use serde::Serialize;
use serde_json::{to_value, Value};
use std::ops::Deref;
use std::sync::Arc;

use crate::field::Field;
use crate::prelude::AssociatedQuery;
use crate::query::{Query, QueryType};
use crate::table::Table;
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

    // TODO: debug why this overwrites the previous fields
    fn add_fields_into_query(&self, mut query: Query, alias_prefix: Option<&str>) -> Query {
        for (field_key, field_val) in &self.fields {
            let field_val = if let Some(alias_prefix) = &alias_prefix {
                let alias = format!("{}_{}", alias_prefix, field_key);
                let mut field_val = field_val.deref().clone();
                field_val.set_field_alias(alias);
                Arc::new(field_val)
            } else {
                field_val.clone()
            };
            query = query.with_column(
                field_val
                    .deref()
                    .get_field_alias()
                    .unwrap_or_else(|| field_key.clone()),
                field_val,
            );
        }

        for (alias, join) in &self.joins {
            query = join.add_fields_into_query(query, Some(alias));
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

    pub fn get_insert_query(&self) -> Query {
        let mut query = Query::new()
            .with_table(&self.table_name, None)
            .with_type(QueryType::Insert);
        for (field, _) in &self.fields {
            let field_object = Arc::new(Field::new(field.clone(), self.table_alias.clone()));
            query = query.with_column(field.clone(), field_object);
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

    use crate::prelude::ExpressionArc;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    use super::*;
    use crate::{expr_arc, mocks::datasource::MockDataSource, prelude::SqlChunk};
    #[test]
    fn test_insert_query() {
        let data =
            json!([{ "name": "John", "surname": "Doe"}, { "name": "Jane", "surname": "Doe"}]);
        let db = MockDataSource::new(&data);

        let table = Table::new("users", db)
            .with_field("name")
            .with_field("surname");

        let query = table.get_insert_query().render_chunk().split();

        assert_eq!(
            query.0,
            "INSERT INTO users (name, surname) VALUES ({}, {}) returning id"
        );
        assert_eq!(query.1[0], Value::Null);
        assert_eq!(query.1[1], Value::Null);
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
