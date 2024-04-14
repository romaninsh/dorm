// We mostly work with dry expressions. Then we can render the expression, which
// converts our structure into SQL or a similar query language. However rendering
// does not give us any data. In order to get data, we need to create a stream

struct Stream {
    data_source: Box<dyn DataSource>,
    query: Query,
}
