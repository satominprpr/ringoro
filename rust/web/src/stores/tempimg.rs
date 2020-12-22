use mongodm::{CollectionConfig, Indexes, Model};

use crate::utils::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TempImg {
    pub data: Vec<u8>,
    #[serde(rename = "type")]
    pub type_: String,
}

pub struct TempImgCfg {}

impl CollectionConfig for TempImgCfg {
    fn collection_name() -> &'static str {
        "TempImg"
    }

    fn indexes() -> Indexes {
        Indexes::new()
    }
}

impl Model for TempImg {
    type CollConf = TempImgCfg;
}
