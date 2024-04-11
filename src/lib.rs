#[allow(dead_code)]
pub enum QueryType {
    Select,
    Insert,
    Update,
    Delete,
}

pub struct Query {
    table: String,
    query_type: QueryType,
    fields: Vec<String>,
}

impl Query {
    pub fn new(table: &str) -> Query {
        Query {
            table: table.to_string(),
            query_type: QueryType::Select,
            fields: Vec::new(),
        }
    }

    pub fn set_type(&mut self, query_type: QueryType) -> &mut Query {
        self.query_type = query_type;
        self
    }

    pub fn fields(&mut self, fields: Vec<&str>) -> &mut Query {
        self.fields = fields.iter().map(|f| f.to_string()).collect();
        self
    }

    pub fn field(&mut self, field: &str) -> &mut Query {
        self.fields.push(field.to_string());
        self
    }

    pub fn build(&self) -> String {
        format!("SELECT {} FROM {}", self.fields.join(", "), self.table)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query() {
        let query = Query::new("users").fields(vec!["id", "name"]).build();

        assert_eq!(query, "SELECT id, name FROM users");
    }
}
