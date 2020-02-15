use crate::{audio::Sounds, Ball};
use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
    core::Transform,
    derive::SystemDesc,
    ecs::prelude::{Entity, Join, Read, ReadExpect, System, SystemData, WriteStorage},
};

/// This system is responsible for checking if a ball has moved into a left or
/// a right edge. Points are distributed to the player on the other side, and
/// the ball is reset.
#[derive(SystemDesc)]
pub struct WinnerSystem;

impl<'s> System<'s> for WinnerSystem {
    type SystemData = (
        WriteStorage<'s, Ball>,
        WriteStorage<'s, Transform>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        Option<Read<'s, Output>>,
    );

    fn run(
        &mut self,
        (mut balls, mut transforms, storage, sounds, audio_output): Self::SystemData,
    ) {
        for (ball, transform) in (&mut balls, &mut transforms).join() {
            use crate::ARENA_WIDTH;

            let ball_x = transform.translation().x;

            let did_hit = ball_x <= ball.radius || ball_x >= ARENA_WIDTH - ball.radius;

            if did_hit {
                // Reset the ball.
                ball.velocity[0] = -ball.velocity[0];
                transform.set_translation_x(ARENA_WIDTH / 2.0);

                // Play audio.
                if let Some(ref output) = audio_output {
                    if let Some(sound) = storage.get(&sounds.score_sfx) {
                        output.play_once(sound, 1.0);
                    }
                }
            }
        }
    }
}

/// Stores the entities that are displaying the player score with UiText.
pub struct ScoreText {
    pub p1_score: Entity,
    pub p2_score: Entity,
}
