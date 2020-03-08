use crate::pong::Intervals;
use crate::{
    audio::{play_bounce, Sounds},
    Ball, Paddle, Side,
};
use crate::{ARENA_HEIGHT, ARENA_WIDTH, BALL_RADIUS, PADDLE_HEIGHT, PADDLE_WIDTH};
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
        Write<'s, Intervals>,
    );

    fn run(
        &mut self,
        (mut balls, paddles, transforms, storage, sounds, audio_output, mut intervals): Self::SystemData,
    ) {
        // Check whether a ball collided, and bounce off accordingly.
        //
        // We also check for the velocity of the ball every time, to prevent multiple collisions
        // from occurring.
        for (ball, transform) in (&mut balls, &transforms).join() {
            let magic_time = 0.6; //intervals.next().unwrap_or(0.0);
                                  //println!("magic_time:{}", magic_time);
            let ball_x = transform.translation().x;
            let ball_y = transform.translation().y;
            // Bounce at the paddles.
            for (paddle, paddle_transform) in (&paddles, &transforms).join() {
                let paddle_x = paddle_transform.translation().x - (paddle.width * 0.5);
                let paddle_y = paddle_transform.translation().y - (paddle.height * 0.5);

                // To determine whether the ball has collided with a paddle, we create a larger
                // rectangle around the current one, by subtracting the ball radius from the
                // lowest coordinates, and adding the ball radius to the highest ones. The ball
                // is then within the paddle if its centre is within the larger wrapper
                // rectangle.
                let rectangle = HitRectangle::new(paddle_x, paddle_y, ball.radius, &paddle.side);
                if point_in_rect(ball_x, ball_y, &rectangle) {
                    if (paddle.side == Side::Left && ball.velocity[0] < 0.0)
                        || (paddle.side == Side::Right && ball.velocity[0] > 0.0)
                    {
                        ball.velocity[0] = -ball.velocity[0];
                    } else if (paddle.side == Side::Top && ball.velocity[1] < 0.0)
                        || (paddle.side == Side::Bottom && ball.velocity[1] > 0.0)
                    {
                        ball.velocity[1] = -ball.velocity[1];
                    }
                    ball.velocity = adjust_velocity(&ball_x, &ball_y, &ball.velocity, &magic_time);
                    play_bounce(&*sounds, &storage, audio_output.as_ref().map(|o| o.deref()));
                }
            }
        }
    }
}
const OFF_SET: f32 = 1.1;
fn radius_offset(ball_radius: f32) -> f32 {
    ball_radius * OFF_SET
}
pub struct Top(pub f32);
impl Top {
    pub fn new(paddle_y: f32, ball_radius: f32, side: &Side) -> Top {
        let ball_radius_offset = radius_offset(ball_radius);
        match side {
            Side::Top | Side::Bottom => Top(paddle_y + (PADDLE_WIDTH + ball_radius_offset)),
            Side::Left | Side::Right => Top(paddle_y + (PADDLE_HEIGHT + ball_radius_offset)),
        }
    }
}
pub struct Bottom(pub f32);
impl Bottom {
    pub fn new(paddle_y: f32, ball_radius: f32) -> Bottom {
        let ball_radius_offset = radius_offset(ball_radius);
        Bottom(paddle_y - ball_radius_offset)
    }
}
pub struct Left(pub f32);
impl Left {
    pub fn new(paddle_x: f32, ball_radius: f32) -> Left {
        let ball_radius_offset = radius_offset(ball_radius);
        Left(paddle_x - ball_radius_offset)
    }
}
pub struct Right(pub f32);
impl Right {
    pub fn new(paddle_x: f32, ball_radius: f32, side: &Side) -> Right {
        let ball_radius_offset = radius_offset(ball_radius);
        match side {
            Side::Top | Side::Bottom => Right(paddle_x + (PADDLE_HEIGHT + ball_radius_offset)),
            Side::Left | Side::Right => Right(paddle_x + (PADDLE_WIDTH + ball_radius_offset)),
        }
    }
}
pub struct HitRectangle {
    pub top: Top,
    pub bottom: Bottom,
    pub left: Left,
    pub right: Right,
}
impl HitRectangle {
    pub fn new(paddle_x: f32, paddle_y: f32, ball_radius: f32, side: &Side) -> HitRectangle {
        HitRectangle {
            top: Top::new(paddle_y, ball_radius, side),
            bottom: Bottom::new(paddle_y, ball_radius),
            left: Left::new(paddle_x, ball_radius),
            right: Right::new(paddle_x, ball_radius, side),
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
        PADDLE_WIDTH + BALL_RADIUS
    };
    let ym = if velocity[1] >= 0.0 {
        ARENA_HEIGHT - (BALL_RADIUS + PADDLE_WIDTH)
    } else {
        PADDLE_WIDTH + BALL_RADIUS
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
fn point_in_rect(x: f32, y: f32, rectangle: &HitRectangle) -> bool {
    match rectangle {
        HitRectangle {
            top: Top(top),
            bottom: Bottom(bottom),
            left: Left(left),
            right: Right(right),
        } => x >= *left && x <= *right && y >= *bottom && y <= *top,
    }
}
