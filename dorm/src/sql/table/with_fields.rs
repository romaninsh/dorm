use indexmap::IndexMap;
use serde_json::Value;
use std::sync::Arc;

use super::Field;
use crate::lazy_expression::LazyExpression;
use crate::prelude::Operations;
use crate::sql::table::Table;
use crate::traits::column::Column;
use crate::traits::datasource::DataSource;
use crate::traits::entity::Entity;

use super::AnyTable;

/// # Table Fields
///
/// Unlike a [`Query`], the [`Table`] will have a fixed set of fields, that you
/// should define and later reference. Typically field structure is defined in
/// your Entity model definition: <https://romaninsh.github.io/dorm/5-entity-model.html>,
/// however in the examples below, we will define the fields ad-hoc.
///
/// ```
/// use dorm::prelude::*;
///
/// let users = Table::new("users", postgres())
///     .with_id_field("id")
///     .with_title_field("name")
///     .with_field("role_name");
///
/// let some_users = users.clone()
///     .with_condition(users.get_field("role_name").unwrap().eq("admin"));
/// ```
///
/// ## Adding and referencing fields:
///
/// Any field you add to the table can be retrieved with [`Table::get_field()`] method.
/// Method actually returns [`Option`]<[`Arc`]<[`Field`]>>, but you should keep `Field`
/// wrapped in `Arc` to avoid cloning it.
///
/// It is a common practice to define a business entity model and use it along with table.
/// Here is a simplified example of defining `Table<Postgress, User>`.
///
/// ```
/// struct User {
///     id: i32,
///     name: String,
///     role_name: String,
/// }
/// impl User {
///     fn table() -> Table<Postgres, User> {
///         Table::new_with_entity("user", postgres())
///             .with_id_field("id")
///             .with_title_field("name")
///             .with_field("role_name")
///     }
/// }
/// pub trait UserTable: AnyTable {
///     fn role_name(&self) -> Arc<Field> {
///         self.get_field("role_name").unwrap()
///     }
/// }
/// impl UserTable for Table<Postgres, User> {}
/// ```
///
/// The above code defines a useful method: `User::table()` that returns a `Table<Postgres, User>`.
/// By implementing additional trait on `Table<Postgres, User>` you can effectively combine
/// existing methods from `Table` with your own methods:
///
/// ```
/// let users = User::table();
/// dbg!(users.get_field("role_name")); // Some(Arc<Field>)
/// dbg!(users.role_name());            // Arc<Field>
/// ```
///
/// A method `role_name()` can perform the unwrap, since we can be sure that such a field exists.
///
/// ```
/// let admin_users = User::table().
///     .with_condition(User::table().role_name().eq("admin"));
/// ```
///
/// ## `id` and `title` fields:
///
/// Methods [`with_id_field()`] and [`with_title_field()`] are identical to [`with_field()`]
/// but they will set internal reference to a field.
///
/// Using [`id()`] can be used to reference the `id` field more conveniently. Also [`with_id`]
/// can be used to set condition on the `id` field. You should always define some field as `id`
/// field, when creating a table.
///
/// ## Advanced field definition:
///
/// When you use [`with_field()`] method, it automatically creates a [`Field`] object
/// and adds it into an internal `fields` [`IndexMap`]. By specifying [`Field`] directly,
/// you can provide additional information about the field, such as alias.
///
/// Method [`add_field()`] accepts a [`Field`] as a second argument and you might want
/// to use it on some rare occasions.
///
/// You may access the internal `fields` [`IndexMap`] with [`fields()`] method.
///
///
/// [`Query`]: super::Query
/// [`with_id_field()`]: Table::with_id_field()
/// [`with_title_field()`]: Table::with_title_field()
/// [`with_field()`]: Table::with_field()
/// [`add_field()`]: Table::add_field()
/// [`id()`]: Table::id()
/// [`with_id`]: Table::with_id()
/// [`fields()`]: Table::fields()

pub trait TableWithFields: AnyTable {
    fn add_field(&mut self, field_name: String, field: Field);
    fn fields(&self) -> &IndexMap<String, Arc<Field>>;
    fn id(&self) -> Arc<Field>;
    fn search_for_field(&self, field_name: &str) -> Option<Box<dyn Column>>;
}

impl<T: DataSource, E: Entity> TableWithFields for Table<T, E> {
    /// **avoid using directly**.
    ///
    /// Adds a new field to the table. Note, that Field may use an alias. Additional
    /// features may be added into [`Field`] in the future, so better use [`with_field()`]
    /// to keep your code portable.
    fn add_field(&mut self, field_name: String, field: Field) {
        self.fields.insert(field_name, Arc::new(field));
    }

    /// Return all fields. See also: [`Table::get_field`].
    fn fields(&self) -> &IndexMap<String, Arc<Field>> {
        &self.fields
    }

    /// Returns the id field. If `with_id_field` was not called, will try to find
    /// field called `"id"`. If not found, will panic.
    fn id(&self) -> Arc<Field> {
        let id_field = if self.id_field.is_some() {
            let x = self.id_field.clone().unwrap();
            x.clone()
        } else {
            "id".to_string()
        };
        self.get_field(&id_field).unwrap()
    }

    /// In addition to `self.fields` the fields can also be defined for a joined
    /// table. (See [`Table::with_join()`]) or through a lazy expression (See
    /// [`Table::with_expression()`]).
    ///
    /// The more broad scope requires us to use a [`Column`] trait rather than
    /// a [`Field`].
    ///
    /// [`Column`]: dorm::sql::Column
    fn search_for_field(&self, field_name: &str) -> Option<Box<dyn Column>> {
        // perhaps we have a field like this?
        if let Some(field) = self.get_field(field_name) {
            return Some(Box::new(field));
        }

        // maybe joined table have a field we want
        for (_, join) in self.joins.iter() {
            if let Some(field) = join.table().get_field(field_name) {
                return Some(Box::new(field.clone()));
            }
        }

        // maybe we have a lazy expression
        if let Some(lazy_expression) = self.lazy_expressions.get(field_name) {
            return match lazy_expression {
                LazyExpression::AfterQuery(_) => None,
                LazyExpression::BeforeQuery(expr) => {
                    let x = (expr)(self);
                    Some(Box::new(x))
                }
            };
        }
        None
    }
}

impl<T: DataSource, E: Entity> Table<T, E> {
    /// When building a table - a way to chain field declarations.
    pub fn with_field(mut self, field: &str) -> Self {
        self.add_field(
            field.to_string(),
            Field::new(field.to_string(), self.table_alias.clone()),
        );
        self
    }

    /// Adds a field that is also a title field. Title field will be
    /// used in the UI to represent the record.
    pub fn with_title_field(mut self, field: &str) -> Self {
        self.title_field = Some(field.to_string());
        self.with_field(field)
    }

    /// Adds a field that is also an id field. Id field is used
    /// by [`Table::id()`] and [`Table::with_id()`].
    pub fn with_id_field(mut self, field: &str) -> Self {
        self.id_field = Some(field.to_string());
        self.with_field(field)
    }

    /// Will add a condition for the `id` field. This is a syntactic sugar for
    /// `with_condition(id().eq(&id))`.
    pub fn with_id(self, id: Value) -> Self {
        let f = self.id().eq(&id);
        self.with_condition(f)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{mocks::datasource::MockDataSource, prelude::*, sql::table::Table};

    #[test]
    fn test_get_field() {
        let data = json!([]);
        let db = MockDataSource::new(&data);

        let roles = Table::new("roles", db.clone())
            .with_field("id")
            .with_field("name");

        assert!(roles.get_field("qq").is_none());
        assert!(roles.get_field("name").is_some());
    }

    #[test]
    fn test_search_for_field() {
        let data = json!([]);
        let db = MockDataSource::new(&data);

        let roles = Table::new("roles", db.clone())
            .with_field("id")
            .with_field("name")
            .with_expression("surname", |_| expr!("foo"));

        assert!(roles.search_for_field("qq").is_none());
        assert!(roles.search_for_field("name").is_some());
        assert!(roles.search_for_field("surname").is_some());
        assert!(roles.get_field("surname").is_none())
    }

    #[test]
    fn test_field_query() {
        let data = json!([]);
        let db = MockDataSource::new(&data);

        let mut roles = Table::new("roles", db.clone())
            .with_field("id")
            .with_field("name");

        roles.add_condition(roles.get_field("name").unwrap().eq(&"admin".to_string()));
        let query = roles.field_query(roles.get_field("id").unwrap());

        assert_eq!(
            query.render_chunk().sql().clone(),
            "SELECT id FROM roles WHERE (name = {})".to_owned()
        );

        let mut users = Table::new("users", db.clone())
            .with_field("id")
            .with_field("role_id");

        users.add_condition(users.get_field("role_id").unwrap().in_expr(&query));
        let query = users.get_select_query().render_chunk().split();

        assert_eq!(
            query.0,
            "SELECT id, role_id FROM users WHERE (role_id IN (SELECT id FROM roles WHERE (name = {})))"
        );
        assert_eq!(query.1[0], json!("admin"));
    }
}
