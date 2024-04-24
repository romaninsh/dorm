use serde_json::Value;

use crate::traits::column::Column;
use crate::traits::sql_chunk::{PreRender, SqlChunk};
use crate::{expr, Expression};

#[derive(Debug)]
pub struct Field {
    name: String,
}

impl Field {
    pub fn new(name: String) -> Field {
        Field { name }
    }
}

impl<'a> SqlChunk<'a> for Field {
    fn render_chunk(&self) -> PreRender {
        PreRender::new((format!("`{}`", self.name), vec![]))
    }
}

impl<'a> Column<'a> for Field {
    fn render_column(&self, alias: &str) -> PreRender {
        (if self.name == alias {
            expr!(format!("`{}`", self.name))
        } else {
            expr!(format!("`{}` AS `{}`", self.name, alias))
        })
        .render_chunk()
    }
    fn calculated(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field() {
        let field = Field::new("id".to_string());
        let (sql, params) = field.render_chunk().split();

        assert_eq!(sql, "`id`");
        assert_eq!(params.len(), 0);

        // let (sql, params) = field.render_column("id").render_chunk();
        // assert_eq!(sql, "`id`");
        // assert_eq!(params.len(), 0);

        // let (sql, params) = &field.render_column("id_alias").render_chunk();
        // assert_eq!(sql, "`id` AS `id_alias`");
        // assert_eq!(params.len(), 0);
    }
}
