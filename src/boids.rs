use crate::linalg::*;
use alloc::vec::Vec;
use core::f32;
use core::f32::consts::PI;
use wasm_bindgen::prelude::*;

// Inter-boid forces.
const ALIGN_FORCE: f32 = 0.2;
const COHESION_FORCE: f32 = 0.05;
const SEPARATION_FORCE: f32 = 2.0;

// Obstacle forces.
const ATTRACTION_FORCE: f32 = 0.05;
const AVOIDANCE_FORCE: f32 = 7.5;
const REPULSION_FORCE: f32 = 8.;

// Limits.
const MAX_SPEED: f32 = 3.;
const MAX_STEER_FORCE: f32 = 0.35;
const NEIGBOURHOOD_RADIUS: f32 = 50.0;
const DESIRED_SEPARATION: f32 = 15.;
const FIELD_OF_VIEW_LIMIT: f32 = 3.0 * PI / 4.0;

// Entity Component System (ECS) design --- struct of vectors as opposed to a vector of structs.
// Fits well here for performance, in theory.
// Watch: https://www.youtube.com/watch?v=aKLntZcp27M
#[wasm_bindgen]
pub struct BoidFlock {
    count: usize,
    positions: Vec<f32>,
    velocities: Vec<f32>,
    accelerations: Vec<f32>,
    width: usize,
    height: usize,
    attractor: Option<(f32, f32)>,
    repulsor: Option<(f32, f32)>,
}

#[wasm_bindgen]
impl BoidFlock {
    #[wasm_bindgen(constructor)]
    pub fn new(count: usize) -> BoidFlock {
        crate::set_panic_hook();

        BoidFlock {
            count,
            positions: vec![0.; 2 * count],
            velocities: vec![1.; 2 * count],
            accelerations: vec![0.; 2 * count],
            width: 0,
            height: 0,
            attractor: None,
            repulsor: None,
        }
    }

    pub fn set_width(&mut self, width: usize) {
        self.width = width;
    }

    pub fn set_height(&mut self, height: usize) {
        self.height = height;
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn positions(&self) -> js_sys::Float32Array {
        // UNSAFETY: views into WebAssembly memory are only valid so long as the backing buffer isn’t resized in JS.
        // The returned array may be mutated, carefully.
        unsafe { js_sys::Float32Array::view(&self.positions) }
    }

    pub fn velocities(&self) -> js_sys::Float32Array {
        // UNSAFETY: views into WebAssembly memory are only valid so long as the backing buffer isn’t resized in JS.
        // The returned array may be mutated, carefully.
        unsafe { js_sys::Float32Array::view(&self.velocities) }
    }

    pub fn update(&mut self) {
        (0..self.count).for_each(|i| self.flock(i));
        (0..self.count).for_each(|i| self.update_boid(i));
    }

    fn get_attractor(&self) -> Option<(f32, f32)> {
        if self.attractor.is_some() {
            self.attractor
        } else {
            Some((self.width as f32 / 2., self.height as f32 / 2.))
        }
    }

    pub fn set_attractor(&mut self, a_x: f32, a_y: f32) {
        self.attractor = Some((a_x, a_y));
    }

    pub fn unset_attractor(&mut self) {
        self.attractor = None;
    }

    fn get_repulsor(&self) -> Option<(f32, f32)> {
        self.repulsor
    }

    pub fn set_repulsor(&mut self, r_x: f32, r_y: f32) {
        self.repulsor = Some((r_x, r_y));
    }

    pub fn unset_repulsor(&mut self) {
        self.repulsor = None;
    }

    fn flock(&mut self, idx: usize) {
        // TODO: look into SIMD instructions in wasm...
        let neighbours = self.get_neighbours(idx);
        let alignment = mul_scalar(&self.align(idx, &neighbours), ALIGN_FORCE);
        let cohesion = mul_scalar(&self.cohede(idx, &neighbours), COHESION_FORCE);
        let separation = mul_scalar(&self.separate(idx, &neighbours), SEPARATION_FORCE);
        let attraction = if let Some((a_x, a_y)) = self.get_attractor() {
            mul_scalar(&self.attract(idx, a_x, a_y), ATTRACTION_FORCE)
        } else {
            (0., 0.)
        };
        let repulsion = if let Some((r_x, r_y)) = self.get_repulsor() {
            mul_scalar(&self.repel(idx, r_x, r_y), REPULSION_FORCE)
        } else {
            (0., 0.)
        };
        let avoidance = mul_scalar(
            &self.avoid_walls(idx, self.width, self.height),
            AVOIDANCE_FORCE,
        );

        let mut acceleration = (0f32, 0f32);
        add2_mut(&mut acceleration, &alignment);
        add2_mut(&mut acceleration, &cohesion);
        add2_mut(&mut acceleration, &separation);
        add2_mut(&mut acceleration, &attraction);
        add2_mut(&mut acceleration, &avoidance);
        add2_mut(&mut acceleration, &repulsion);

        self.accelerations[2 * idx] = acceleration.0;
        self.accelerations[2 * idx + 1] = acceleration.1;
    }

    /// Update the position and velocity of a boid based on its acceleration.
    /// Note: `flock()` should always precede this call.
    fn update_boid(&mut self, idx: usize) {
        let (vx, vy) = limit(
            self.velocities[2 * idx] + self.accelerations[2 * idx],
            self.velocities[2 * idx + 1] + self.accelerations[2 * idx + 1],
            MAX_SPEED,
        );
        let mut x = self.positions[2 * idx] + vx;
        let mut y = self.positions[2 * idx + 1] + vy;
        self.velocities[2 * idx] = vx;
        self.velocities[2 * idx + 1] = vy;

        // Warp the boid to the opposite side of the screen when it goes out of bounds.
        if x < 0. {
            x = self.width as f32 - 1.;
        } else if x > self.width as f32 {
            x = 1.;
        }
        if y < 0. {
            y = self.height as f32 - 1.;
        } else if y > self.height as f32 {
            y = 1.;
        }

        self.positions[2 * idx] = x;
        self.positions[2 * idx + 1] = y;
    }

    fn get_neighbours(&self, idx: usize) -> Vec<usize> {
        // Slow loop through all the boids to find neighbours.
        // TODO: segment the map or use a quad tree structure to speed this up.
        // Reusing a shared boolean Vec for each boid is slower.
        let mut neighbours = Vec::new();
        for i in 0..self.count {
            if i == idx {
                continue;
            }
            let dist = euclid_dist(
                self.positions[2 * idx],
                self.positions[2 * idx + 1],
                self.positions[2 * i],
                self.positions[2 * i + 1],
            );
            if dist < NEIGBOURHOOD_RADIUS {
                // Potential neighbour.
                // Project a line from this boid to the potential neighbour to calculate the angle
                // between it and the velocity vector of this boid, to see if it's in the field of vision.
                let (linex, liney) = (
                    self.positions[2 * idx] - self.positions[2 * i],
                    self.positions[2 * idx + 1] - self.positions[2 * i + 1],
                );
                if linex + liney > 0.01
                    && self.velocities[2 * idx] + self.velocities[2 * idx + 1] > 0.01
                    && angle_between(
                        self.velocities[2 * idx],
                        self.velocities[2 * idx + 1],
                        linex,
                        liney,
                    )
                    .abs()
                        > FIELD_OF_VIEW_LIMIT
                {
                    // Neighbour is behind the boid, out of its field of vision.
                    continue;
                }
                neighbours.push(i)
            }
        }

        neighbours
    }

    /// Steer towards the average heading of the neighbours.
    fn align(&self, _idx: usize, neighbours: &[usize]) -> (f32, f32) {
        let mut steer = neighbours
            .iter()
            .fold((0f32, 0f32), |steer: (f32, f32), &oidx| {
                add(
                    steer.0,
                    steer.1,
                    self.velocities[2 * oidx],
                    self.velocities[2 * oidx + 1],
                )
            });

        let len = neighbours.len();
        if len > 0 {
            let div = len as f32 / MAX_STEER_FORCE;
            steer.0 /= div;
            steer.1 /= div;
        }

        steer
    }

    /// Steer towards the centre of the neighbourhood.
    fn cohede(&self, idx: usize, neighbours: &[usize]) -> (f32, f32) {
        let mut steer = neighbours
            .iter()
            .fold((0f32, 0f32), |steer: (f32, f32), &oidx| {
                add(
                    steer.0,
                    self.positions[2 * oidx],
                    steer.1,
                    self.positions[2 * oidx + 1],
                )
            });
        let len = neighbours.len();
        if len > 0 {
            steer.0 = steer.0 / len as f32 - self.positions[2 * idx];
            steer.1 = steer.1 / len as f32 - self.positions[2 * idx + 1];
        }

        limit(steer.0, steer.1, MAX_STEER_FORCE)
    }

    /// Steer away from enchroaching neighbour boids, depending on how close they are.
    fn separate(&self, idx: usize, neighbours: &[usize]) -> (f32, f32) {
        let x = self.positions[2 * idx];
        let y = self.positions[2 * idx + 1];
        neighbours
            .iter()
            .fold((0f32, 0f32), |mut steer: (f32, f32), &oidx| {
                let ox = self.positions[2 * oidx];
                let oy = self.positions[2 * oidx + 1];
                let dist = euclid_dist(x, y, ox, oy);

                if dist > 0. && dist < DESIRED_SEPARATION {
                    let dir = normalise(x - ox, y - oy);
                    steer.0 += dir.0 / dist;
                    steer.1 += dir.1 / dist;
                }
                steer
            })
    }

    /// Steer towards a target location.
    fn attract(&self, idx: usize, targetx: f32, targety: f32) -> (f32, f32) {
        let x = self.positions[2 * idx];
        let y = self.positions[2 * idx + 1];
        self.steer_towards(x, y, targetx, targety)
    }

    fn steer_away(&self, x: f32, y: f32, ox: f32, oy: f32) -> (f32, f32) {
        let dist = euclid_dist(x, y, ox, oy);
        if dist > 0. {
            ((x - ox) / (dist * dist), (y - oy) / (dist * dist))
        } else {
            (0., 0.)
        }
    }

    fn steer_towards(&self, x: f32, y: f32, ox: f32, oy: f32) -> (f32, f32) {
        let dist = euclid_dist(x, y, ox, oy);
        if dist > 0. {
            ((ox - x) / dist, (oy - y) / dist)
        } else {
            (0., 0.)
        }
    }

    fn repel(&self, idx: usize, r_x: f32, r_y: f32) -> (f32, f32) {
        let x = self.positions[2 * idx];
        let y = self.positions[2 * idx + 1];
        self.steer_away(x, y, r_x, r_y)
    }

    fn avoid_walls(&self, idx: usize, width: usize, height: usize) -> (f32, f32) {
        let x = self.positions[2 * idx];
        let y = self.positions[2 * idx + 1];

        let mut steer = (0f32, 0f32);

        // Avoid walls.
        add2_mut(&mut steer, &self.steer_away(x, y, 0., y));
        add2_mut(&mut steer, &self.steer_away(x, y, width as f32, y));
        add2_mut(&mut steer, &self.steer_away(x, y, x, 0.));
        add2_mut(&mut steer, &self.steer_away(x, y, x, height as f32));

        // Avoid corners.
        // steer = add2(steer, self.steer_away(x, y, 0., 0.));
        // steer = add2(steer, self.steer_away(x, y, width as f32, 0.));
        // steer = add2(steer, self.steer_away(x, y, 0., height as f32));
        // steer = add2(steer, self.steer_away(x, y, width as f32, height as f32));

        steer
    }
}
