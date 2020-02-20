use crate::{
    audio::{play_bounce, Sounds},
    beats::Beats,
    Ball, Paddle, Side,
};
use crate::{ARENA_HEIGHT, ARENA_WIDTH, BALL_RADIUS, PADDLE_WIDTH};
use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
    core::transform::Transform,
    derive::SystemDesc,
    ecs::prelude::{Join, Read, ReadExpect, ReadStorage, System, SystemData, Write, WriteStorage},
};
use std::ops::Deref;

/// This system is responsible for detecting collisions between balls and
/// paddles, as well as balls and the top and bottom edges of the arena.
#[derive(SystemDesc)]
pub struct BounceSystem;

impl<'s> System<'s> for BounceSystem {
    type SystemData = (
        WriteStorage<'s, Ball>,
        ReadStorage<'s, Paddle>,
        ReadStorage<'s, Transform>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        Option<Read<'s, Output>>,
        Write<'s, Beats>,
    );

    fn run(
        &mut self,
        (mut balls, paddles, transforms, storage, sounds, audio_output, mut beats): Self::SystemData,
    ) {
        // Check whether a ball collided, and bounce off accordingly.
        //
        // We also check for the velocity of the ball every time, to prevent multiple collisions
        // from occurring.
        for (ball, transform) in (&mut balls, &transforms).join() {
            let magic_time = beats.intervals.pop().unwrap_or(0.0);
            println!("magic_time:{}",magic_time);
            let ball_x = transform.translation().x;
            let ball_y = transform.translation().y;

            // Bounce at the top or the bottom of the arena.
            if (ball_y <= ball.radius && ball.velocity[1] < 0.0)
                || (ball_y >= ARENA_HEIGHT - ball.radius && ball.velocity[1] > 0.0)
            {
                ball.velocity[1] = -ball.velocity[1];
                ball.velocity = adjust_velocity(&ball_x, &ball_y, &ball.velocity, &magic_time);
                play_bounce(&*sounds, &storage, audio_output.as_ref().map(|o| o.deref()));
            }

            // Bounce at the paddles.
            for (paddle, paddle_transform) in (&paddles, &transforms).join() {
                let paddle_x = paddle_transform.translation().x - (paddle.width * 0.5);
                let paddle_y = paddle_transform.translation().y - (paddle.height * 0.5);

                // To determine whether the ball has collided with a paddle, we create a larger
                // rectangle around the current one, by subtracting the ball radius from the
                // lowest coordinates, and adding the ball radius to the highest ones. The ball
                // is then within the paddle if its centre is within the larger wrapper
                // rectangle.
                if point_in_rect(
                    ball_x,
                    ball_y,
                    paddle_x - ball.radius,
                    paddle_y - ball.radius,
                    paddle_x + (paddle.width + ball.radius),
                    paddle_y + (paddle.height + ball.radius),
                ) && ((paddle.side == Side::Left && ball.velocity[0] < 0.0)
                    || (paddle.side == Side::Right && ball.velocity[0] > 0.0))
                {
                    ball.velocity[0] = -ball.velocity[0];
                    ball.velocity = adjust_velocity(&ball_x, &ball_y, &ball.velocity, &magic_time);
                    play_bounce(&*sounds, &storage, audio_output.as_ref().map(|o| o.deref()));
                }
            }
        }
    }
}

fn adjust_velocity(x: &f32, y: &f32, velocity: &[f32; 2], magic_time: &f32) -> [f32; 2] {
    match fixed_coordinate(x, y, velocity) {
        (xm, ym) => [(xm - x) / magic_time, (ym - y) / magic_time],
    }
}

fn fixed_coordinate(x: &f32, y: &f32, velocity: &[f32; 2]) -> (f32, f32) {
    let xm = if velocity[0] >= 0.0 {
        ARENA_WIDTH - (PADDLE_WIDTH + BALL_RADIUS)
    } else {
        BALL_RADIUS
    };
    let ym = if velocity[1] >= 0.0 {
        ARENA_HEIGHT - BALL_RADIUS
    } else {
        BALL_RADIUS
    };
    let tx = (xm - *x) / velocity[0];
    let ty = (ym - *y) / velocity[1];
    if tx <= ty {
        let ym = velocity[1] * tx + y;
        (xm, ym)
    } else {
        let xm = velocity[0] * ty + x;
        (xm, ym)
    }
}

// A point is in a box when its coordinates are smaller or equal than the top
// right and larger or equal than the bottom left.
fn point_in_rect(x: f32, y: f32, left: f32, bottom: f32, right: f32, top: f32) -> bool {
    x >= left && x <= right && y >= bottom && y <= top
}
