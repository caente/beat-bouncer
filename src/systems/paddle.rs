use crate::systems::{Bottom, Left, Right, Top};
use crate::Side;
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
        for (ball, ball_transform) in (&ball, &transforms).join() {
            let ball_x = ball_transform.translation().x;
            let ball_y = ball_transform.translation().y;
            for (paddle, paddle_transform) in (&mut paddle, &transforms).join() {
                let paddle_x = paddle_transform.translation().x;
                let paddle_y = paddle_transform.translation().y;
                let collision = CollisionPrediction::new(
                    paddle_x,
                    paddle_y,
                    ball_x,
                    ball_y,
                    ball,
                    &paddle.side,
                );
                let movement = collision.intersection_velocity;
                paddle.velocity = movement;
            }
        }
    }
}

pub struct CollisionPrediction {
    pub x: f32,
    pub y: f32,
    pub time_until_collision: f32,
    pub intersection_velocity: f32,
}

impl CollisionPrediction {
    fn new(
        paddle_x: f32,
        paddle_y: f32,
        ball_x: f32,
        ball_y: f32,
        ball: &Ball,
        destination: &Side,
    ) -> CollisionPrediction {
        let [velocity_x, velocity_y] = ball.velocity;

        let Left(left) = Left::new(paddle_x, ball.radius);
        let Right(right) = Right::new(paddle_x, ball.radius, destination);
        let Top(top) = Top::new(paddle_y, ball.radius, destination);
        let Bottom(bottom) = Bottom::new(paddle_y, ball.radius);

        let end_x = if velocity_x < 0.0 { left } else { right };
        let end_y = if velocity_y < 0.0 { top } else { bottom };

        let time_until_collision_x = (end_y - ball_y) / velocity_y;
        let time_until_collision_y = (end_x - ball_x) / velocity_x;

        let collision_x = velocity_x * time_until_collision_x + ball_x + ball.radius;
        let collision_y = velocity_y * time_until_collision_y + ball_y;

        let velocity_paddle_x = (collision_x - paddle_x) / time_until_collision_x;
        let velocity_paddle_y = (collision_y - paddle_y) / time_until_collision_y;

        match destination {
            Side::Bottom | Side::Top => CollisionPrediction {
                x: collision_x,
                y: end_y,
                time_until_collision: time_until_collision_x,
                intersection_velocity: velocity_paddle_x,
            },
            Side::Left | Side::Right => CollisionPrediction {
                x: end_x,
                y: collision_y,
                time_until_collision: time_until_collision_y,
                intersection_velocity: velocity_paddle_y,
            },
        }
    }
}
