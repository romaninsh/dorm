use crate::condition::Condition;
use crate::traits::dataset::{ReadableDataSet, WritableDataSet};
use crate::traits::datasource::DataSource;
use crate::{Expression, Field};
use indexmap::IndexMap;

// Generic implementation of SQL table. We don't really want to use this extensively,
// instead we want to use 3rd party SQL builders, that cary table schema information.
pub struct Table<'a> {
    data_source: Box<dyn DataSource>,
    table_name: String,
    fields: IndexMap<String, Field>,
    title_field: Option<String>,
    conditions: Vec<Condition>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> Table<'a> {
    pub fn new(table_name: &str, data_source: Box<dyn DataSource>) -> Table {
        Table {
            table_name: table_name.to_string(),
            data_source,
            title_field: None,
            conditions: Vec::new(),
            fields: IndexMap::new(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn add_field(mut self, field: &str) -> Self {
        self.fields.insert(field.to_string(), Field::new(field));
        self
    }

    pub fn add_title_field(mut self, field: &str) -> Self {
        self.title_field = Some(field.to_string());
        self.add_field(field)
    }

    pub fn add_condition(mut self, field: &'static str, op: &'static str, value: String) -> Self {
        self.conditions.push(Condition::new(field, op, value));
        self
    }

    pub fn get_select_query(&'a self) -> crate::Query<'a> {
        let mut query = crate::Query::new(&self.table_name);
        for (field, _) in &self.fields {
            let field = Field::new(field);
            query = query.add_column(Box::new(field));
        }
        for condition in &self.conditions {
            query = query.add_condition(condition);
        }
        query
    }

    pub fn get_all_data(&self) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
        self.data_source.query_fetch(&self.get_select_query())
    }

    pub fn sum(&'a self, expr: &str) -> Expression<'a> {
        let field = self.fields.get(expr).unwrap();
        Expression::new("SUM({})", vec![field])
    }

    pub fn iter(&self) -> impl Iterator<Item = Vec<String>> {
        self.get_all_data().unwrap().into_iter()
    }
}

impl<'a> IntoIterator for Table<'a> {
    type Item = Vec<String>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.get_all_data().unwrap().into_iter()
    }
}

impl<'a> ReadableDataSet for Table<'a> {
    fn select_query(&self) -> crate::Query {
        self.get_select_query()
    }
}

impl<'a> WritableDataSet for Table<'a> {
    fn insert_query(&self) -> crate::Query {
        todo!()
    }

    fn update_query(&self) -> crate::Query {
        todo!()
    }

    fn delete_query(&self) -> crate::Query {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{mocks::datasource::MockDataSource, Renderable};

    #[test]
    fn test_table() {
        let data = vec![vec!["1", "2"]];
        let data_source = MockDataSource::new(data.clone());

        let table = Table::new("users", Box::new(data_source))
            .add_field("name")
            .add_field("surname");

        let result = table.get_all_data();

        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn test_conditions() {
        let data = vec![vec!["1", "2"]];
        let data_source = MockDataSource::new(data.clone());

        let table = Table::new("users", Box::new(data_source))
            .add_field("name")
            .add_field("surname")
            .add_condition("name", "=", "John".to_owned());

        assert_eq!(
            table.select_query().render(),
            "SELECT name, surname FROM users WHERE name = 'John'".to_owned()
        );

        let result = table.get_all_data();

        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn test_vip_client() {
        let data = vec![vec!["1", "2"]];
        let db = MockDataSource::new(data.clone());

        let vip_client = Table::new("client", Box::new(db))
            .add_title_field("name")
            .add_field("is_vip")
            .add_field("total_spent")
            .add_condition("is_vip", "is", "true".to_owned());

        let sum = vip_client.sum("total_spent");
        assert_eq!(sum.render(), "(SUM(total_spent))".to_owned());
    }
}
