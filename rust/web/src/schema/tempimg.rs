use juniper::{GraphQLInputObject, GraphQLObject};
use mongodm::bson::oid::ObjectId;

#[derive(GraphQLInputObject)]
pub struct TempImgInput {
    pub type_: String,
    pub data: String,
}

#[derive(GraphQLObject)]
pub struct TempImg {
    pub id: ObjectId,
    pub url: String,
    pub type_: String,
}
