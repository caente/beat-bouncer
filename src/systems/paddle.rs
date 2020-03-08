use crate::systems::{Bottom, Left, Right, Top};
use crate::{Ball, Paddle};
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
        use crate::{Side, PADDLE_WIDTH, PADDLE_HEIGHT};
        for (ball, ball_transform) in (&ball, &transforms).join() {
            let ball_x = ball_transform.translation().x;
            let ball_y = ball_transform.translation().y;
            let [velocity_x, velocity_y] = ball.velocity;
            for (paddle, paddle_transform) in (&mut paddle, &transforms).join() {
                let paddle_x = paddle_transform.translation().x;
                let paddle_y = paddle_transform.translation().y;
                let Top(top) = Top::new(paddle_y, ball.radius, &paddle.side);
                let Bottom(bottom) = Bottom::new(paddle_y, ball.radius);
                let Left(left) = Left::new(paddle_x, ball.radius);
                let Right(right) = Right::new(paddle_x, ball.radius, &paddle.side);
                let end_x = if velocity_x < 0.0 { left } else { right };
                let end_y = if velocity_y < 0.0 { top } else { bottom };
                let time_until_collision_y = (end_x - ball_x) / velocity_x;
                let time_until_collision_x = (end_y - ball_y) / velocity_y;
                let collision_y = velocity_y * time_until_collision_y + ball_y;
                let collision_x = velocity_x * time_until_collision_x + ball_x;
                let velocity_paddle_y = (collision_y - paddle_y) / time_until_collision_y;
                let velocity_paddle_x = (collision_x - paddle_x) / time_until_collision_x;
                let movement = match paddle.side {
                    Side::Left | Side::Right => velocity_paddle_y,
                    Side::Top | Side::Bottom => velocity_paddle_x,
                };
                paddle.velocity = movement;
            }
        }
    }
}
