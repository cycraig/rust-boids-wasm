use boids::BoidFlock;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::Rng;

fn init_boid_flock(n: isize) -> BoidFlock {
    let mut rng = rand::thread_rng();
    let mut flock = BoidFlock::new(n as usize);
    let width: usize = 600;
    let height: usize = 600;
    flock.set_width(width);
    flock.set_height(height);
    let positions = flock.positions_mut();
    unsafe {
        for i in 0..n {
            *positions.offset(2 * i) = rng.gen_range(0..width) as f32;
            *positions.offset(2 * i + 1) = rng.gen_range(0..height) as f32;
        }
    }
    flock
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("flock", |b| {
        b.iter(|| {
            let mut flock = init_boid_flock(black_box(100));
            for _ in 0..black_box(120) {
                flock.update();
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
