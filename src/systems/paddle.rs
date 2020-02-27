use crate::Ball;
use crate::Paddle;
use amethyst::{
    core::transform::Transform,
    derive::SystemDesc,
    ecs::prelude::{Join, ReadStorage, System, SystemData, WriteStorage},
};

/// This system is responsible for moving all the paddles according to the user
/// provided input.
#[derive(SystemDesc)]
pub struct PaddleSystem;

impl<'s> System<'s> for PaddleSystem {
    type SystemData = (
        WriteStorage<'s, Paddle>,
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Ball>,
    );

    fn run(&mut self, (mut paddle, transforms, ball): Self::SystemData) {
        use crate::Side;
        for (ball, ball_transform) in (&ball, &transforms).join() {
            let ball_x = ball_transform.translation().x;
            let ball_y = ball_transform.translation().y;
            let [velocity_x, velocity_y] = ball.velocity;
            use crate::{ARENA_HEIGHT, ARENA_WIDTH};
            let end_x = if velocity_x < 0.0 { 0.0 } else { ARENA_WIDTH };
            let end_y = if velocity_y < 0.0 { 0.0 } else { ARENA_HEIGHT };
            let time_until_collision_y = (end_x - ball_x) / velocity_x;
            let time_until_collision_x = (end_y - ball_y) / velocity_y;
            let collision_y = velocity_y * time_until_collision_y + ball_y;
            let collision_x = velocity_y * time_until_collision_x + ball_x;
            for (paddle, paddle_transform) in (&mut paddle, &transforms).join() {
                let paddle_y = paddle_transform.translation().y;
                let paddle_x = paddle_transform.translation().x;
                let velocity_paddle_y = (collision_y - paddle_y) / time_until_collision_y;
                let velocity_paddle_x = (collision_x - paddle_x) / time_until_collision_x;
                let movement = match paddle.side {
                    Side::Left if velocity_x < 0.0 => velocity_paddle_y,
                    Side::Left => 0.0,
                    Side::Right if velocity_x >= 0.0 => velocity_paddle_y,
                    Side::Right => 0.0,
                    Side::Top if velocity_y < 0.0 => velocity_paddle_x,
                    Side::Top => 0.0,
                    Side::Bottom if velocity_y > 0.0 => velocity_paddle_x,
                    Side::Bottom => 0.0,
                };
                paddle.velocity = movement;
            }
        }
    }
}
