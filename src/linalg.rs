// Basic vector2 operations to avoid dependencies bloating the wasm target...
use core::f32;
use core::f32::consts::PI;

pub fn add(ax: f32, ay: f32, bx: f32, by: f32) -> (f32, f32) {
    (ax + bx, ay + by)
}

pub fn add2_mut((ax, ay): &mut (f32, f32), (bx, by): &(f32, f32)) {
    *ax += bx;
    *ay += by;
}

pub fn mul_scalar((x, y): &(f32, f32), scalar: f32) -> (f32, f32) {
    (scalar * x, scalar * y)
}

pub fn euclid_dist(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    ((ax - bx) * (ax - bx) + (ay - by) * (ay - by)).sqrt()
}

pub fn angle_between(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let mut angle = ay.atan2(ax) - by.atan2(bx);
    // normalize to (-pi, pi]
    if angle > PI {
        angle -= 2.0 * PI;
    } else if angle <= -PI {
        angle += 2.0 * PI;
    }

    angle
}

pub fn norm(x: f32, y: f32) -> f32 {
    (x * x + y * y).sqrt()
}

pub fn normalise(mut x: f32, mut y: f32) -> (f32, f32) {
    let nm = norm(x, y);
    if nm > 0. {
        x /= nm;
        y /= nm;
    }
    (x, y)
}

pub fn limit(mut x: f32, mut y: f32, limit: f32) -> (f32, f32) {
    let nm = norm(x, y);
    if nm > 0. && nm > limit {
        x *= limit / nm;
        y *= limit / nm;
    }
    (x, y)
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    macro_rules! assert_eqf32 {
        ($x:expr, $y:expr) => {
            assert_approx_eq!($x, $y, 1e-6f32)
        };
    }

    fn _norm_tuple((x, y): (f32, f32)) -> f32 {
        norm(x, y)
    }

    #[test]
    fn test_add() {
        assert_eq!(add(0., 0., 0., 0.), (0., 0.));
        assert_eq!(add(0., 2., 1., 0.), (1., 2.));
        assert_eq!(add(1., 0., 0., 2.), (1., 2.));
        assert_eq!(add(-1., 2., 1., -2.), (0., 0.));
        assert_eq!(add(1., 123., 9., 877.), (10., 1000.));
    }

    #[test]
    fn test_add2() {
        let add2_fn = |mut a: (f32, f32), b: (f32, f32)| -> (f32, f32) {
            add2_mut(&mut a, &b);
            a
        };
        assert_eq!(add2_fn((0., 0.), (0., 0.)), (0., 0.));
        assert_eq!(add2_fn((0., 2.), (1., 0.)), (1., 2.));
        assert_eq!(add2_fn((1., 0.), (0., 2.)), (1., 2.));
        assert_eq!(add2_fn((-1., 2.), (1., -2.)), (0., 0.));
        assert_eq!(add2_fn((1., 123.), (9., 877.)), (10., 1000.));
    }

    #[test]
    fn test_mul_scalar() {
        assert_eq!(mul_scalar(&(0., 0.), 3.), (0., 0.));
        assert_eq!(mul_scalar(&(1., 2.), 0.), (0., 0.));
        assert_eq!(mul_scalar(&(1., 2.), 1.), (1., 2.));
        assert_eq!(mul_scalar(&(1., 2.), 5.), (5., 10.));
        assert_eq!(mul_scalar(&(1., 2.), -5.), (-5., -10.));
    }

    #[test]
    fn test_norm() {
        assert_eqf32!(norm(0., 0.), 0.);
        assert_eqf32!(norm(0., 1.), 1.);
        assert_eqf32!(norm(1., 1.), (2. as f32).sqrt());
        assert_eqf32!(norm(1., 2.), (5. as f32).sqrt());
        assert_eqf32!(norm(3., 4.), 5.);
        assert_eqf32!(norm(-1., 1.), (2. as f32).sqrt());
        assert_eqf32!(norm(-1., -2.), (5. as f32).sqrt());
        assert_eqf32!(norm(-3., -4.), 5.);
    }

    #[test]
    fn test_normalise() {
        assert_eq!(normalise(0., 0.), (0., 0.));
        assert_eq!(normalise(0., 1.), (0., 1.));
        assert_eq!(
            normalise(1., 1.),
            (1. / (2. as f32).sqrt(), 1. / (2. as f32).sqrt())
        );
        assert_eq!(
            normalise(1., 2.),
            (1. / (5. as f32).sqrt(), (2. / (5. as f32).sqrt()))
        );
        assert_eq!(normalise(3., 4.), (3. / 5., 4. / 5.));
        assert_eq!(
            normalise(-1., 1.),
            (-1. / (2. as f32).sqrt(), 1. / (2. as f32).sqrt())
        );
        assert_eq!(
            normalise(-1., -2.),
            (-1. / (5. as f32).sqrt(), (-2. / (5. as f32).sqrt()))
        );
        assert_eq!(normalise(-3., -4.), (-3. / 5., -4. / 5.));
    }

    #[test]
    fn test_norm_after_normalise() {
        assert_eqf32!(_norm_tuple(normalise(0., 0.)), 0.);
        assert_eqf32!(_norm_tuple(normalise(0., 1.)), 1.);
        assert_eqf32!(_norm_tuple(normalise(1., 1.)), 1.);
        assert_eqf32!(_norm_tuple(normalise(1., 2.)), 1.);
        assert_eqf32!(_norm_tuple(normalise(3., 4.)), 1.);
        assert_eqf32!(_norm_tuple(normalise(-1., 1.)), 1.);
        assert_eqf32!(_norm_tuple(normalise(-1., -2.)), 1.);
        assert_eqf32!(_norm_tuple(normalise(-3., -4.)), 1.);
    }

    #[test]
    fn test_euclid_dist() {
        assert_eqf32!(euclid_dist(0., 0., 0., 0.), 0.);
        assert_eqf32!(euclid_dist(0., 1., 0., 1.), 0.);
        assert_eqf32!(euclid_dist(1., 1., 1., 1.), 0.);
        assert_eqf32!(euclid_dist(-1., 1., -1., 1.), 0.);
        assert_eqf32!(euclid_dist(1234., 5678., 1234., 5678.), 0.);
        assert_eqf32!(euclid_dist(0., 1., 1., 0.), (2. as f32).sqrt());
        assert_eqf32!(euclid_dist(1., 2., 3., 4.), (8. as f32).sqrt());
        assert_eqf32!(euclid_dist(3., 4., 1., 2.), (8. as f32).sqrt());
        assert_eqf32!(euclid_dist(-1., 1., 1., -1.), (8. as f32).sqrt());
    }

    #[test]
    fn test_angle_between() {
        assert_eqf32!(angle_between(0., 0., 0., 0.), 0.);
        assert_eqf32!(angle_between(0., 1., 0., 1.), 0.);
        assert_eqf32!(angle_between(1., 1., 1., 1.), 0.);
        assert_eqf32!(angle_between(-1., 1., -1., 1.), 0.);
        assert_eqf32!(angle_between(1234., -5678., 1234., -5678.), 0.);
        assert_eqf32!(angle_between(0., 1., 1., 1.), PI / 4.);
        assert_eqf32!(angle_between(0., 1., 1., 0.), PI / 2.);
        assert_eqf32!(angle_between(0., 1., 1., -1.), 3. * PI / 4.);
        assert_eqf32!(angle_between(0., 1., 0., -1.), PI);
        assert_eqf32!(angle_between(0., 1., -1., -1.), -3. * PI / 4.);
        assert_eqf32!(angle_between(0., 1., -1., 0.), -PI / 2.);
        assert_eqf32!(angle_between(0., 1., -1., 1.), -PI / 4.);
    }

    #[test]
    fn test_norm_limit() {
        assert_eqf32!(_norm_tuple(limit(0., 0., 2.)), 0.);
        assert_eqf32!(_norm_tuple(limit(0., 1., 2.)), 1.);
        assert_eqf32!(_norm_tuple(limit(0., 2., 2.)), 2.);
        assert_eqf32!(_norm_tuple(limit(0., 3., 2.)), 2.);
        assert_eqf32!(_norm_tuple(limit(0., -1., 2.)), 1.);
        assert_eqf32!(_norm_tuple(limit(0., -2., 2.)), 2.);
        assert_eqf32!(_norm_tuple(limit(0., -3., 2.)), 2.);
        assert_eqf32!(_norm_tuple(limit(1., 0., 2.)), 1.);
        assert_eqf32!(_norm_tuple(limit(2., 0., 2.)), 2.);
        assert_eqf32!(_norm_tuple(limit(3., 0., 2.)), 2.);
        assert_eqf32!(_norm_tuple(limit(-1., 0., 2.)), 1.);
        assert_eqf32!(_norm_tuple(limit(-2., 0., 2.)), 2.);
        assert_eqf32!(_norm_tuple(limit(-3., 0., 2.)), 2.);
        assert_eqf32!(_norm_tuple(limit(1., 1., 1.)), 1.);
        assert_eqf32!(_norm_tuple(limit(1., 1., 2.)), (2. as f32).sqrt());
        assert_eqf32!(_norm_tuple(limit(-1., -1., 1.)), 1.);
        assert_eqf32!(_norm_tuple(limit(-1., -1., 2.)), (2. as f32).sqrt());
        assert_eqf32!(_norm_tuple(limit(123., 456., 42.)), 42.);
        assert_eqf32!(
            _norm_tuple(limit(123., 456., 500.)),
            (123. * 123. + 456. * 456. as f32).sqrt()
        );
        assert_eqf32!(_norm_tuple(limit(-123., -456., 42.)), 42.);
        assert_eqf32!(
            _norm_tuple(limit(-123., -456., 500.)),
            (123. * 123. + 456. * 456. as f32).sqrt()
        );
    }
}
