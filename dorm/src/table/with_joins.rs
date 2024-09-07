use std::ptr::eq;
use std::sync::Arc;

use crate::join::Join;
use crate::prelude::{Operations, SqlChunk};
use crate::query::{JoinQuery, JoinType, QueryConditions};
use crate::table::Table;
use crate::traits::datasource::DataSource;
use crate::uniqid::UniqueIdVendor;

impl<T: DataSource> Table<T> {
    /// Left-Joins their_table table and return self. Assuming their_table has set id field,
    /// but we still have to specify foreign key in our own table. For more complex
    /// joins use `join_table` method.
    pub fn with_join(mut self, their_table: Table<T>, our_foreign_id: &str) -> Self {
        self.add_join(their_table, our_foreign_id);
        self
    }

    pub fn add_join(&mut self, mut their_table: Table<T>, our_foreign_id: &str) -> Arc<Join<T>> {
        // before joining, make sure there are no alias clashes
        if eq(&*self.table_aliases, &*their_table.table_aliases) {
            panic!(
                "Tables are already joined: {}, {}",
                self.table_name, their_table.table_name
            )
        }

        if their_table
            .table_aliases
            .lock()
            .unwrap()
            .has_conflict(&self.table_aliases.lock().unwrap())
        {
            panic!(
                "Table alias conflict while joining: {}, {}",
                self.table_name, their_table.table_name
            )
        }

        self.table_aliases
            .lock()
            .unwrap()
            .merge(their_table.table_aliases.lock().unwrap().to_owned());

        // Get information about their_table
        let their_table_name = their_table.table_name.clone();
        if their_table.table_alias.is_none() {
            let their_table_alias = self
                .table_aliases
                .lock()
                .unwrap()
                .get_one_of_uniq_id(UniqueIdVendor::all_prefixes(&their_table_name));
            their_table.set_alias(&their_table_alias);
        };
        let their_table_id = their_table.id();

        // Give alias to our table as well
        if self.table_alias.is_none() {
            let our_table_alias = self
                .table_aliases
                .lock()
                .unwrap()
                .get_one_of_uniq_id(UniqueIdVendor::all_prefixes(&self.table_name));
            self.set_alias(&our_table_alias);
        }
        let their_table_alias = their_table.table_alias.as_ref().unwrap().clone();

        let mut on_condition = QueryConditions::on().add_condition(
            self.get_field(our_foreign_id)
                .unwrap()
                .eq(&their_table_id)
                .render_chunk(),
        );

        // Any condition in their_table should be moved into ON condition
        for condition in their_table.conditions.iter() {
            on_condition = on_condition.add_condition(condition.render_chunk());
        }
        their_table.conditions = Vec::new();

        // Create a join
        let join = JoinQuery::new(
            JoinType::Left,
            crate::query::QuerySource::Table(their_table_name, Some(their_table_alias.clone())),
            on_condition,
        );
        self.joins.insert(
            their_table_alias.clone(),
            Arc::new(Join::new(their_table, join)),
        );

        self.get_join(&their_table_alias).unwrap()
    }

    pub fn get_join(&self, table_alias: &str) -> Option<Arc<Join<T>>> {
        self.joins.get(table_alias).map(|r| r.clone())
    }
}
