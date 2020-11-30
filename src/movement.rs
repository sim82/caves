use bevy::math;
use bevy::{prelude::*, render::camera::Camera};
// use bevy_tiled_prototype::level;
use super::level;
use bevy_tiled_prototype::TiledMapCenter;

pub(crate) fn intersect_dist(shape: &level::CollisionShape, rect: &math::Rect<f32>) -> Vec2 {
    match shape {
        level::CollisionShape::Rect(shape) => {
            let x = 0f32
                .max(shape.right - rect.left)
                .min(0f32.max(rect.right - shape.left));

            let y = 0f32
                .max(shape.top - rect.bottom)
                .min(0f32.max(rect.top - shape.bottom));
            Vec2::new(x, y)
            // rect.left <= shape.right
            //     && rect.right >= shape.left
            //     && rect.top >= shape.bottom
            //     && rect.bottom <= shape.top
        }
    }
}

#[test]
fn test_intersect() {
    let testcases = [
        ((0., 2., 1., 3.), (1.5, 3.5, 2., 4.), (0.5, 1.)),
        ((0., 2., 1., 3.), (-1.5, 0.5, 2., 4.), (-0.5, 1.)),
        ((0., 2., 1., 3.), (-1.5, 0.5, -0.5, 1.5), (-0.5, -0.5)),
        ((0., 2., 1., 3.), (1.5, 3.5, -0.5, 1.5), (0.5, -0.5)),
        ((0., 2., 1., 3.), (2.5, 4.5, -0.5, 1.5), (-0.5, -0.5)),
    ];

    for (r1, r2, dr) in testcases.iter() {
        let d = intersect_dist2(
            &level::CollisionShape::Rect(Rect {
                left: r1.0,
                right: r1.1,
                bottom: r1.2,
                top: r1.3,
            }),
            &Rect {
                left: r2.0,
                right: r2.1,
                bottom: r2.2,
                top: r2.3,
            },
        );
        // d.x()
        // let eps = 1e-6;
        assert_relative_eq!(d.x(), dr.0);
        assert_relative_eq!(d.y(), dr.1);
    }

    //intersect_dist()
}

pub fn range_non_overlap(a1: f32, a2: f32, b1: f32, b2: f32) -> bool {
    b1 >= a2 || b2 <= a1
}

pub fn intersect_dist2(shape: &level::CollisionShape, rect: &math::Rect<f32>) -> Vec2 {
    match shape {
        level::CollisionShape::Rect(shape) => {
            let dx1 = shape.right - rect.left;
            let dx2 = shape.left - rect.right;

            let dy1 = shape.top - rect.bottom;
            let dy2 = shape.bottom - rect.top;

            let x = if dx1.abs() < dx2.abs() { dx1 } else { dx2 };
            let y = if dy1.abs() < dy2.abs() { dy1 } else { dy2 };

            Vec2::new(x, y)
        }
    }
}

#[derive(Debug, Clone)]
pub enum MoveRes {
    Complete(Vec2),
    Collision(Vec2, f32, [bool; 4]),
    Stuck,
}

pub fn try_move(s1: &level::CollisionShape, r2: &Rect<f32>, d_target: &Vec2) -> MoveRes {
    let level::CollisionShape::Rect(r1) = s1;

    let x_pos = d_target.x() > 0.0;
    let x_neg = d_target.x() < 0.0;

    let y_pos = d_target.y() > 0.0;
    let y_neg = d_target.y() < 0.0;

    let xfree_start = range_non_overlap(r1.left, r1.right, r2.left, r2.right);
    let yfree_start = range_non_overlap(r1.bottom, r1.top, r2.bottom, r2.top);

    if !(xfree_start || yfree_start) {
        return MoveRes::Stuck;
    }

    let mut r2_target = r2.clone();
    r2_target.left += d_target.x();
    r2_target.right += d_target.x();
    r2_target.top += d_target.y();
    r2_target.bottom += d_target.y();

    let xfree_end = range_non_overlap(r1.left, r1.right, r2_target.left, r2_target.right);
    let yfree_end = range_non_overlap(r1.bottom, r1.top, r2_target.bottom, r2_target.top);

    if xfree_start && xfree_end || yfree_start && yfree_end {
        return MoveRes::Complete(*d_target);
    }
    // println!(
    //     "try_move: {} {} {:?} -> {:?} {:?}",
    //     xfree_start, yfree_start, r2, r2_target, r1
    // );

    let intx = match (xfree_start, xfree_end) {
        (true, true) => 1.0,
        (false, _) => 1.0,
        (true, false) => {
            if x_pos {
                (r2_target.right - r1.left) / d_target.x()
            } else if x_neg {
                (r2_target.left - r1.right) / d_target.x()
            } else {
                1.0
            }
        }
    };

    let inty = match (yfree_start, yfree_end) {
        (true, true) => 1.0,
        (false, _) => 1.0,
        (true, false) => {
            if y_pos {
                (r2_target.top - r1.bottom) / d_target.y()
            } else if y_neg {
                (r2_target.bottom - r1.top) / d_target.y()
            } else {
                1.0
            }
        }
    };

    // println!("int: {} {}", intx, inty);

    let dx = 1.0 - intx;
    let dy = 1.0 - inty;

    let mut d = dx.max(dy);

    if d < f32::EPSILON * 100.0 {
        d = 0.0;
    }

    // println!("d: {} {} {}", dx, dy, d);

    MoveRes::Collision(
        *d_target * d,
        d,
        [
            x_neg && !xfree_end,
            x_pos && !xfree_end,
            y_neg && !yfree_end,
            y_pos && !yfree_end,
        ],
    )
}

#[test]
fn test_move() {
    let r1 = Rect {
        left: 0.0,
        right: 2.0,
        top: 3.0,
        bottom: 1.0,
    };

    let r2 = Rect {
        left: 3.0,
        right: 5.0,
        top: 2.0,
        bottom: 0.0,
    };

    let d = try_move(&level::CollisionShape::Rect(r1), &r2, &Vec2::new(-1.5, 0.5));
    println!("d: {:?}", d);
}
