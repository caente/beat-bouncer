use crate::{
    audio::MusicFile, beats, systems::ScoreText, Ball, Paddle, Side, ARENA_HEIGHT, ARENA_WIDTH,
};
use amethyst::{
    assets::{AssetStorage, Handle, Loader},
    core::{timing::Time, transform::Transform},
    ecs::prelude::World,
    prelude::*,
    renderer::{Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture},
};
#[derive(Default)]
pub struct Pong {
    ball_spawn_timer: Option<f32>,
    sprite_sheet_handle: Option<Handle<SpriteSheet>>,
}

impl SimpleState for Pong {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;
        use crate::audio::initialise_audio;

        // Wait one second before spawning the ball.
        self.ball_spawn_timer.replace(1.0);

        // Load the spritesheet necessary to render the graphics.
        // `spritesheet` is the layout of the sprites on the image;
        // `texture` is the pixel data.
        self.sprite_sheet_handle.replace(load_sprite_sheet(world));
        initialise_paddles(world, self.sprite_sheet_handle.clone().unwrap());
        initialise_camera(world);
        world.insert({
            MusicFile {
                audio_file: "audio/Computer_Music_All-Stars_-_Wheres_My_Jetpack.ogg",
            }
        });
        initialise_audio(world);
        let beats =
            beats::find_beats("assets/audio/Computer_Music_All-Stars_-_Wheres_My_Jetpack.ogg")
                .unwrap();
        world.insert(beats);
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if let Some(mut timer) = self.ball_spawn_timer.take() {
            // If the timer isn't expired yet, substract the time that passed since last update.
            {
                let time = data.world.fetch::<Time>();
                timer -= time.delta_seconds();
            }
            if timer <= 0.0 {
                // When timer expire, spawn the ball
                initialise_ball(data.world, self.sprite_sheet_handle.clone().unwrap());
            } else {
                // If timer is not expired yet, put it back onto the state.
                self.ball_spawn_timer.replace(timer);
            }
        }
        Trans::None
    }
}

fn load_sprite_sheet(world: &mut World) -> Handle<SpriteSheet> {
    // Load the sprite sheet necessary to render the graphics.
    // The texture is the pixel data
    // `sprite_sheet` is the layout of the sprites on the image
    // `texture_handle` is a cloneable reference to the texture

    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            "texture/pong_spritesheet_all.png",
            ImageFormat::default(),
            (),
            &texture_storage,
        )
    };

    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        "texture/pong_spritesheet.ron", // Here we load the associated ron file
        SpriteSheetFormat(texture_handle), // We pass it the texture we want it to use
        (),
        &sprite_sheet_store,
    )
}

/// Initialise the camera.
fn initialise_camera(world: &mut World) {
    // Setup camera in a way that our screen covers whole arena and (0, 0) is in the bottom left.
    let mut transform = Transform::default();
    transform.set_translation_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 1.0);

    world
        .create_entity()
        .with(Camera::standard_2d(ARENA_WIDTH, ARENA_HEIGHT))
        .with(transform)
        .build();
}

/// Initialises one paddle on the left, and one paddle on the right.
fn initialise_paddles(world: &mut World, sprite_sheet_handle: Handle<SpriteSheet>) {
    use crate::{PADDLE_HEIGHT, PADDLE_VELOCITY, PADDLE_WIDTH};

    let mut left_transform = Transform::default();
    let mut right_transform = Transform::default();
    let mut top_transform = Transform::default();
    let mut bottom_transform = Transform::default();

    // Correctly position the paddles.
    let y = ARENA_HEIGHT / 2.0;
    left_transform.set_translation_xyz(PADDLE_WIDTH * 0.5, y, 0.0);
    right_transform.set_translation_xyz(ARENA_WIDTH - PADDLE_WIDTH * 0.5, y, 0.0);
    bottom_transform.set_translation_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT - PADDLE_WIDTH * 0.5, 0.0);
    top_transform.set_translation_xyz(ARENA_WIDTH * 0.5, PADDLE_WIDTH * 0.5, 0.0);

    // Assign the sprites for the paddles
    let sprite_render_vertical = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: 0,
    };

    let sprite_render_horizontal = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 2,
    };

    // Create a left plank entity.
    world
        .create_entity()
        .with(sprite_render_vertical.clone())
        .with(Paddle {
            velocity: PADDLE_VELOCITY,
            side: Side::Left,
            width: PADDLE_WIDTH,
            height: PADDLE_HEIGHT,
        })
        .with(left_transform)
        .build();

    // Create right plank entity.
    world
        .create_entity()
        .with(sprite_render_vertical)
        .with(Paddle {
            velocity: 0.0,
            side: Side::Right,
            width: PADDLE_WIDTH,
            height: PADDLE_HEIGHT,
        })
        .with(right_transform)
        .build();
    // Create top plank entity
    world
        .create_entity()
        .with(sprite_render_horizontal.clone())
        .with(Paddle {
            velocity: 0.0,
            side: Side::Top,
            width: PADDLE_HEIGHT,
            height: PADDLE_WIDTH,
        })
        .with(top_transform)
        .build();
    // Create bottom plank entity
    world
        .create_entity()
        .with(sprite_render_horizontal)
        .with(Paddle {
            velocity: 0.0,
            side: Side::Bottom,
            width: PADDLE_HEIGHT,
            height: PADDLE_WIDTH,
        })
        .with(bottom_transform)
        .build();
}

/// Initialises one ball in the middle-ish of the arena.
fn initialise_ball(world: &mut World, sprite_sheet_handle: Handle<SpriteSheet>) {
    use crate::{BALL_RADIUS, BALL_VELOCITY_X, BALL_VELOCITY_Y};

    // Create the translation.
    let mut local_transform = Transform::default();
    local_transform.set_translation_xyz(ARENA_WIDTH / 2.0, ARENA_HEIGHT / 2.0, 0.0);

    // Assign the sprite for the ball
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 1, // ball is the second sprite on the sprite_sheet
    };

    world
        .create_entity()
        .with(sprite_render)
        .with(Ball {
            radius: BALL_RADIUS,
            velocity: [BALL_VELOCITY_X, BALL_VELOCITY_Y],
        })
        .with(local_transform)
        .build();
}
