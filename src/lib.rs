mod field;
mod query;
mod traits;

pub use field::Field;
pub use query::Query;
pub use traits::renderable::Renderable;

// Trait is implemented by a physical field or a expression inside a query
// which can be built as a part of a query and has a specific type
trait Column {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query() {
        let query = Query::new("users").fields(vec!["id", "name"]).render();

        assert_eq!(query, "SELECT id, name FROM users");
    }
}
