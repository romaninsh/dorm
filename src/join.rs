use crate::{condition::Condition, traits::datasource::DataSource};

#[derive(Clone)]
enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

#[derive(Clone)]
pub struct Join<T: DataSource> {
    data_source: T,
    join_type: JoinType,
    table_name: String,
    table_alias: Option<String>,
    on_conditions: Vec<Condition>,
}

impl<T: DataSource> Join<T> {
    fn new(table_name: &str, data_source: T) -> Join<T> {
        Join {
            table_name: table_name.to_string(),
            table_alias: None,
            data_source,
            join_type: JoinType::Left,
            on_conditions: Vec::new(),
        }
    }

    fn new_field(&self, field: String) -> Field {
        Field::new(field, self.table_alias.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join() {
        let data =
            json!([{ "name": "John", "surname": "Doe"}, { "name": "Jane", "surname": "Doe"}]);
        let db = MockDataSource::new(&data);

        let vip_client = Table::new("client", db)
            .add_title_field("name")
            .add_field("is_vip")
            .add_field("total_spent")
            .add_condition_on_field("is_vip", "is", "true".to_owned())
            .unwrap();

        let vip_details = vip_client.add_join_table("id", "vip_details", "client_id");
        vip_details.add_field("discount");

        let select = vip_client.get_select_query();
        assert_eq!(
            select.render_chunk().sql().clone(),
            "SELECT name, total_spent, vip_details.discount FROM client JOIN vip_details ON vip_detils.client_id = client.id WHERE is_vip is {}".to_owned()
        );
    }
}
