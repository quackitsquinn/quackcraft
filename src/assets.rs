use engine::{
    assets::AssetStore,
    component::ComponentStore,
    graphics::{lowlevel::WgpuRenderer, textures::TextureCollection},
};
use log::info;

use crate::{include_minecraft_texture, render::block_textures::BlockTextureAtlas, world::Block};

pub struct BlockTextureState {
    pub textures: TextureCollection,
    pub atlas: BlockTextureAtlas,
}

pub fn init_asset_store(
    components: &ComponentStore,
    wgpu: &WgpuRenderer,
) -> (TextureCollection, BlockTextureAtlas, AssetStore) {
    let mut a = AssetStore::new();

    a.add_image("dirt", include_minecraft_texture!("block/dirt"))
        .unwrap();
    a.add_image(
        "grass_block_side",
        include_minecraft_texture!("block/grass_block_side"),
    )
    .unwrap();
    a.add_image(
        "grass_block_top",
        include_minecraft_texture!("block/grass_block_top"),
    )
    .unwrap();
    a.add_image("stone", include_minecraft_texture!("block/stone"))
        .unwrap();
    a.add_image("oak_wood", include_minecraft_texture!("block/oak_log"))
        .unwrap();
    a.add_image(
        "oak_log_top",
        include_minecraft_texture!("block/oak_log_top"),
    )
    .unwrap();
    a.add_image("oak_leaves", include_minecraft_texture!("block/oak_leaves"))
        .unwrap();

    let (texture_collection, atlas) = init_texture_collection(components, wgpu, &a);
    (texture_collection, atlas, a)
}

fn init_texture_collection(
    components: &ComponentStore,
    wgpu: &WgpuRenderer,
    asset_store: &AssetStore,
) -> (TextureCollection, BlockTextureAtlas) {
    let mut texture_collection =
        TextureCollection::new(components, Some("Block Texture Atlas"), (16, 16));

    let dirt = asset_store.get_image("dirt").unwrap();

    let grass_block = [
        asset_store.get_image("grass_block_side").unwrap(),
        asset_store.get_image("grass_block_top").unwrap(),
        asset_store.get_image("dirt").unwrap(),
    ];

    let stone = asset_store.get_image("stone").unwrap();
    let oak_wood = [
        asset_store.get_image("oak_wood").unwrap(),
        asset_store.get_image("oak_log_top").unwrap(),
    ];

    let oak_leaves = asset_store.get_image("oak_leaves").unwrap();

    let dirt_handle = texture_collection.add_texture("dirt", &dirt);

    let grass_handle = texture_collection.add_textures("grass_block", &grass_block);

    let stone_handle = texture_collection.add_texture("stone", &stone);

    let oak_wood_handle = texture_collection.add_textures("oak_wood", &oak_wood);

    let oak_leaves_handle = texture_collection.add_texture("oak_leaves", &oak_leaves);

    info!(
        "Initialized texture collection with textures: dirt={:?}, grass_block={:?}, stone={:?}, oak_wood={:?}, oak_leaves={:?}",
        dirt_handle, grass_handle, stone_handle, oak_wood_handle, oak_leaves_handle
    );

    let mut atlas = BlockTextureAtlas::new();

    atlas.set_texture_handle(Block::Dirt, dirt_handle);
    atlas.set_texture_handle(Block::Grass, grass_handle);
    atlas.set_texture_handle(Block::Stone, stone_handle);
    atlas.set_texture_handle(Block::OakWood, oak_wood_handle);
    atlas.set_texture_handle(Block::OakLeaves, oak_leaves_handle);

    (texture_collection, atlas)
}
