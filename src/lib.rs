mod expression;
mod field;
mod mocks;
mod query;
mod table;
mod traits;

pub use expression::Expression;
pub use field::Field;
pub use query::Query;
pub use traits::renderable::Renderable;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression() {
        let query = Query::new("users")
            .add_column_field("id")
            .add_column_field("name")
            .add_column_expr(expr!("1 + 1"))
            .render();

        assert_eq!(query, "SELECT id, name, (1 + 1) FROM users");
    }
}
