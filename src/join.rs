use std::ops::{Deref, DerefMut};

use crate::{
    prelude::{JoinQuery, Table},
    traits::datasource::DataSource,
};

#[derive(Clone, Debug)]
enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

#[derive(Clone, Debug)]
pub struct Join<T: DataSource> {
    table: Table<T>,
    join_query: JoinQuery,
}

impl<T: DataSource> Join<T> {
    pub fn new(table: Table<T>, join_query: JoinQuery) -> Join<T> {
        Join { table, join_query }
    }
    pub fn alias(&self) -> &str {
        self.table.get_alias().unwrap()
    }
    pub fn join_query(&self) -> &JoinQuery {
        &self.join_query
    }
    pub fn table(&self) -> &Table<T> {
        &self.table
    }
    pub fn table_mut(&mut self) -> &mut Table<T> {
        &mut self.table
    }
}

impl<T: DataSource> Deref for Join<T> {
    type Target = Table<T>;

    fn deref(&self) -> &Self::Target {
        &self.table
    }
}

impl<T: DataSource> DerefMut for Join<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.table
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_join() {
//         let data =
//             json!([{ "name": "John", "surname": "Doe"}, { "name": "Jane", "surname": "Doe"}]);
//         let db = MockDataSource::new(&data);

//         let vip_client = Table::new("client", db)
//             .add_title_field("name")
//             .add_field("is_vip")
//             .add_field("total_spent")
//             .add_condition_on_field("is_vip", "is", "true".to_owned())
//             .unwrap();

//         let vip_details = vip_client.add_join_table("id", "vip_details", "client_id");
//         vip_details.add_field("discount");

//         let select = vip_client.get_select_query();
//         assert_eq!(
//             select.render_chunk().sql().clone(),
//             "SELECT name, total_spent, vip_details.discount FROM client JOIN vip_details ON vip_detils.client_id = client.id WHERE is_vip is {}".to_owned()
//         );
//     }
// }
