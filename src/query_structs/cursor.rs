use graphql_client::GraphQLQuery;

pub trait Cursor<R> {
    fn cursor(response: &R::ResponseData) -> Option<String>
    where
        R: GraphQLQuery + Send + Sync;
}
