use mongodm::{CollectionConfig, Indexes, Model};

use crate::utils::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RawImage {
    pub data: Vec<u8>,
    #[serde(rename = "type")]
    pub type_: String,
}

pub struct RawImageCfg {}

impl CollectionConfig for RawImageCfg {
    fn collection_name() -> &'static str {
        "RawImage"
    }

    fn indexes() -> Indexes {
        Indexes::new()
    }
}

impl Model for RawImage {
    type CollConf = TempImgCfg;
}
