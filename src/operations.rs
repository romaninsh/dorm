use crate::{
    expr,
    traits::sql_chunk::{PreRender, SqlChunk},
    Expression,
};

pub trait Operations<'a>: SqlChunk<'a> {
    fn eq(&self, other: impl SqlChunk<'a>) -> PreRender {
        expr!("({}) = ({})", self.render_chunk(), other.render_chunk()).render_chunk()
    }

    fn add(&self, other: impl SqlChunk<'a>) -> PreRender {
        expr!("({}) + ({})", self.render_chunk(), other.render_chunk()).render_chunk()
    }

    fn sub(&self, other: impl SqlChunk<'a>) -> PreRender {
        expr!("({}) - ({})", self.render_chunk(), other.render_chunk()).render_chunk()
    }
}

pub fn concat(arg: Vec<&dyn SqlChunk>) -> PreRender {
    let arg = arg
        .iter()
        .map(|x| x.render_chunk())
        .collect::<Vec<PreRender>>();

    let arg_ref = arg
        .iter()
        .map(|x| x as &dyn SqlChunk)
        .collect::<Vec<&dyn SqlChunk>>();

    let format = arg.iter().map(|_| "{}").collect::<Vec<&str>>().join(", ");
    let expr = Expression::new(format, arg_ref);
    expr.render_chunk()
}
