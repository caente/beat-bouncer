use crate::Paddle;
use amethyst::{
    core::{timing::Time, transform::Transform},
    derive::SystemDesc,
    ecs::prelude::{Join, Read, System, SystemData, WriteStorage},
};

/// This system is responsible for moving all the paddles according to the user
/// provided input.
#[derive(SystemDesc)]
pub struct MovePaddleSystem;

impl<'s> System<'s> for MovePaddleSystem {
    type SystemData = (
        WriteStorage<'s, Paddle>, 
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
        );

    fn run(&mut self, (mut paddle, mut transforms, time): Self::SystemData) {
        use crate::ARENA_HEIGHT;
        for (paddle, paddle_transform) in (&mut paddle, &mut transforms).join() {
            paddle_transform.prepend_translation_y(paddle.velocity * time.delta_seconds());
            // We make sure the paddle remains in the arena.
            let paddle_y = paddle_transform.translation().y;
            paddle_transform.set_translation_y(
                paddle_y
                    .max(paddle.height * 0.5)
                    .min(ARENA_HEIGHT - paddle.height * 0.5),
            );
        }
    }
}
