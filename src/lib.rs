use engine::{
    component::State, graphics::lowlevel::WgpuRenderer, input::keyboard::Keyboard, window,
};

pub mod block;
pub mod chunk;
pub mod coords;
pub mod mesh;
pub mod world;

/// A position in the world, in chunk coordinates.
pub type BlockPosition = coords::BlockPosition;
/// A position in the world, in chunk coordinates.
pub type ChunkPosition = coords::BlockPosition;

pub const FACE_TABLE: [[([f32; 3], [f32; 2]); 4]; 6] = [
    // +X
    [
        ([1.0, 0.0, 1.0], [0.0, 0.0]),
        ([1.0, 1.0, 1.0], [0.0, 1.0]),
        ([1.0, 1.0, 0.0], [1.0, 1.0]),
        ([1.0, 0.0, 0.0], [1.0, 0.0]),
    ],
    // -X
    [
        ([0.0, 0.0, 1.0], [0.0, 0.0]),
        ([0.0, 1.0, 1.0], [0.0, 1.0]),
        ([0.0, 1.0, 0.0], [1.0, 1.0]),
        ([0.0, 0.0, 0.0], [1.0, 0.0]),
    ],
    // +Y
    [
        ([1.0, 1.0, 0.0], [1.0, 0.0]),
        ([0.0, 1.0, 0.0], [0.0, 0.0]),
        ([0.0, 1.0, 1.0], [0.0, 1.0]),
        ([1.0, 1.0, 1.0], [1.0, 1.0]),
    ],
    // -Y
    [
        ([0.0, 0.0, 1.0], [0.0, 1.0]),
        ([1.0, 0.0, 1.0], [1.0, 1.0]),
        ([1.0, 0.0, 0.0], [1.0, 0.0]),
        ([0.0, 0.0, 0.0], [0.0, 0.0]),
    ],
    // +Z
    [
        ([0.0, 1.0, 1.0], [0.0, 1.0]),
        ([0.0, 0.0, 1.0], [0.0, 0.0]),
        ([1.0, 0.0, 1.0], [1.0, 0.0]),
        ([1.0, 1.0, 1.0], [1.0, 1.0]),
    ],
    // -Z
    [
        ([1.0, 0.0, 0.0], [1.0, 0.0]),
        ([1.0, 1.0, 0.0], [1.0, 1.0]),
        ([0.0, 1.0, 0.0], [0.0, 1.0]),
        ([0.0, 0.0, 0.0], [0.0, 0.0]),
    ],
];

pub const FACE_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

pub struct Game {
    component_db: State,
}

impl Game {
    pub fn new() -> anyhow::Result<Self> {
        let mut state = State::new();
        state.insert(Keyboard::new());
        let window = window::GlfwWindow::new(800, 600, "Minecraft Clone")
            .expect("Failed to create GLFW window");

        smol::block_on(WgpuRenderer::attach_to(&mut state, &window))?;

        state.insert(window);

        state.finish_initialization();

        Ok(Self {
            component_db: state,
        })
    }

    pub fn update(&mut self) {}
}

pub fn run_game() -> anyhow::Result<()> {
    let game = Game::new()?;

    println!("Game initialized: {:?}", game.component_db);
    Ok(())
}

/// Includes a Minecraft resource file at the given path.
/// The path should be relative to the `res/assets/minecraft/textures` directory.
#[macro_export]
macro_rules! include_minecraft_texture {
    ($res: literal) => {
        include_bytes!(concat!("../res/assets/minecraft/textures/", $res, ".png"))
    };
}
