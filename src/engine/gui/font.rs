use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetLoader, AsyncReadExt, LoadContext};
use bevy::reflect::TypePath;
use bevy_egui::egui::FontData;

#[derive(Asset, TypePath)]
pub struct EguiFontAsset {
    pub data: FontData,
}

#[derive(Default)]
pub struct EguiFontAssetLoader;

impl AssetLoader for EguiFontAssetLoader {
    type Asset = EguiFontAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = vec![];
        reader.read_to_end(&mut bytes).await?;
        let data = FontData::from_owned(bytes);
        Ok(EguiFontAsset { data })
    }
}
