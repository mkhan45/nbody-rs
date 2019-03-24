use ggez::nalgebra as na;
use std::f32::consts::PI;
use crate::body::Body;
use std::collections::HashSet;

use rayon::prelude::*;

const G: f32 = 6.674;

type Point2 = na::Point2<f32>;
type Vector2 = na::Vector2<f32>;

pub fn collide(body1: &Body, body2: &Body) -> Body{
    let body1_momentum = Point2::new(body1.velocity.x, body1.velocity.y);
    let body2_momentum = Point2::new(body2.velocity.x, body2.velocity.y);

    let body1_momentum = Point2::new(body1_momentum.x * body1.mass, body1_momentum.y * body1.mass);
    let body2_momentum = Point2::new(body2_momentum.x * body2.mass, body2_momentum.y * body2.mass);

    let total_momentum = Vector2::new(body1_momentum.x + body2_momentum.x, body1_momentum.y + body2_momentum.y);

    
    let volume_1 = 4.0/3.0 * PI * body1.radius.powf(3.0);
    let volume_2 = 4.0/3.0 * PI * body2.radius.powf(3.0);

    let total_mass = body1.mass + body2.mass;

    let total_volume = volume_1 + volume_2;

    let new_rad = (((3.0/4.0)*total_volume)/PI).powf(1.0/3.0);

    Body::new(
        if body1.radius > body2.radius {Point2::new(body1.pos.x, body1.pos.y)} else {Point2::new(body2.pos.x, body2.pos.y)},
        total_mass,
        new_rad,
        Vector2::new(total_momentum.x/total_mass, total_momentum.y/total_mass),
    )
}

pub fn distance(a: &Point2, b: &Point2) -> f32{
    ((b.x - a.x).powf(2.0) + (b.y-a.y).powf(2.0)).sqrt()
}

pub fn angle(a: &Point2, b: &Point2) -> f32{
    let mut restricted_dom = ((b.y - a.y)/(b.x - a.x)).atan();
    if b.x >= a.x{
        restricted_dom += PI;
    }

    restricted_dom
}

pub fn update_velocities_and_collide(bodies: &Vec<Body>, method: &Integrator) -> Vec<Body>{
        let mut bodies = bodies.clone();
        let mut collision_blacklist = HashSet::new();
        let mut collision_bodies = Vec::new();
        microprofile::scope!("Update velocities/collide", "Calculations");

        for current_body_i in 0..bodies.len(){
            bodies[current_body_i].current_accel = Vector2::new(0.0, 0.0);
            for other_body_i in 0..bodies.len(){
                if other_body_i != current_body_i {
                    let other_body = &bodies[other_body_i].clone();
                    let current_body = &mut bodies[current_body_i];

                    let r = distance(&other_body.pos, &current_body.pos);
                    let a_mag = (G*other_body.mass)/(r.powi(2)); //acceleration = Gm_2/r^2
                    let angle = angle(&other_body.pos, &current_body.pos);
                    
                    if r <= other_body.radius + current_body.radius && !collision_blacklist.contains(&current_body_i){
                        collision_blacklist.insert(current_body_i);
                        collision_blacklist.insert(other_body_i);
                        collision_bodies.push(collide(&current_body, &other_body));
                    }

                    current_body.current_accel.x += angle.cos() * a_mag;
                    current_body.current_accel.y += angle.sin() * a_mag;
                }
            }

            match method {
                &Integrator::Euler => bodies[current_body_i].update_euler(),
                &Integrator::Verlet => bodies[current_body_i].update_verlet(),
            };
        }

        bodies = bodies.par_iter()
            .enumerate()
            .filter_map(|(index, body)| {
                if collision_blacklist.contains(&index) {
                    None
                } else {
                    Some(body.clone())
                }
            }).collect();
        
        bodies.append(&mut collision_bodies);
        return bodies;
}

#[derive(Debug)]
pub enum Integrator{
    Euler,
    Verlet,
}
