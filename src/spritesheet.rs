use anyhow::Result;
use bevy::{
    asset::{AssetLoader, LoadedAsset},
    prelude::*,
};
use bevy_type_registry::TypeUuid;
use std::{collections::HashMap, path::Path};
#[derive(Debug, TypeUuid)]
#[uuid = "ab3a0ad8-6fbc-4528-a4a5-90e7bf3fa9e1"]
pub struct Spritesheet {
    image: String,
    ranges: HashMap<String, std::ops::Range<u32>>,
}

impl Spritesheet {
    fn try_from_bytes(asset_path: &Path, bytes: Vec<u8>) -> Result<Spritesheet> {
        let spritesheet = Spritesheet {
            image: "".into(),
            ranges: HashMap::new(),
        };

        Ok(spritesheet)
    }
}

#[derive(Default)]
struct SpritesheetLoader {}

#[derive(Default)]
pub struct SpritesheetPlugin;

impl Plugin for SpritesheetPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<Spritesheet>()
            .init_asset_loader::<SpritesheetLoader>();
    }
}

impl AssetLoader for SpritesheetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let path = load_context.path();
            let map = Spritesheet::try_from_bytes(path, bytes.into())?;
            load_context.set_default_asset(LoadedAsset::new(map));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["json"];
        EXTENSIONS
    }
}
