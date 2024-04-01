use super::*;

/// Defines a curve of the type: `a * t^2 + b * t + c`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Parabola<T = f32> {
    pub a: vec2<T>,
    pub b: vec2<T>,
    pub c: vec2<T>,
}

impl<T: Float> Parabola<T> {
    pub fn new(points: [vec2<T>; 3]) -> Self {
        let two = T::ONE + T::ONE;
        let [p0, p1, p2] = points;
        // f(t)  = a * t^2 + b * t + c
        // f(-1) = p0    (start)     | a - b + c = p0
        // f(0)  = p1    (middle)    | c = p1
        // f(1)  = p2    (end)       | a + b + c = p2
        let c = p1;
        let a = (p2 + p0) / two - c;
        let b = (p2 - p0) / two;

        Self { a, b, c }
    }

    pub fn map<U: Float>(self, f: impl Fn(T) -> U) -> Parabola<U> {
        Parabola {
            a: self.a.map(&f),
            b: self.b.map(&f),
            c: self.c.map(&f),
        }
    }

    pub fn get(&self, t: T) -> vec2<T> {
        self.a * t * t + self.b * t + self.c
    }

    pub fn tangent(&self, t: T) -> vec2<T> {
        let two = T::ONE + T::ONE;
        self.a * t * two + self.b
    }

    /// Returns the `t` value of the closest point on the parabola to the given point.
    pub fn project(&self, point: vec2<T>) -> T {
        self.normals_from(point)
            .into_iter()
            .min_by_key(|&t| r32((self.get(t) - point).len_sqr().as_f32()))
            .expect("At least one root was expected")
    }

    /// Returns all `t`s where the vector from the point forms a perpendicular with the parabola tangent.
    pub fn normals_from(&self, point: vec2<T>) -> Vec<T> {
        let two = T::ONE + T::ONE;
        // let third = T::from_f32(1.0 / 3.0);

        // Solve a cubic equation to find candidate points
        // The equation describes the derivative being zero
        // or, equivalently, tangent being perpendicular to the delta
        let a = two * self.a.len_sqr();
        let b = T::from_f32(3.0) * vec2::dot(self.a, self.b);
        let c = two * vec2::dot(self.a, self.c) + self.b.len_sqr() - two * vec2::dot(self.a, point);
        let d = vec2::dot(self.b, self.c) - vec2::dot(self.b, point);

        // // Solve using <https://en.wikipedia.org/wiki/Cubic_equation#Cardano's_formula>
        // // Transform to depressed form
        // let p = (T::from_f32(3.0) * a * c - b * b) / (T::from_f32(3.0) * a * a);
        // let q = (two * b * b * b - T::from_f32(9.0) * a * b * c + T::from_f32(27.0) * a * a * d)
        //     / (T::from_f32(27.0) * a * a * a);

        // let big_q = {
        //     let a = p / T::from_f32(3.0);
        //     let b = q / two;
        //     a * a * a + b * b
        // };

        // let Some(cmp) = big_q.partial_cmp(&T::ZERO) else {
        //     return T::ZERO;
        // };
        // let xs = match cmp {
        //     std::cmp::Ordering::Greater => {
        //         // 1 real root, 2 complex root
        //         let q_root = big_q.sqrt();
        //         let alpha = (-q / two + q_root).powf(-third);
        //         let beta = (-q / two - q_root).powf(-third);
        //         vec![alpha + beta]
        //     }
        //     std::cmp::Ordering::Equal => {
        //         // All real roots
        //         let alpha = (-q / two).powf(-third); // beta = alpha
        //         vec![two * alpha, -alpha]
        //     }
        //     std::cmp::Ordering::Less => {
        //         // All real roots, but requires complex algebra
        //         let alpha_cube = vec2(-q / two, (-big_q).sqrt()); // Imaginary number

        //         // Polar form
        //         let r = alpha_cube.len();
        //         let theta = alpha_cube.arg().normalized_2pi();

        //         // Three cube roots

        //         vec![]
        //     }
        // };

        // let ts = xs.into_iter().map(|x| x - b / T::from_f32(3.0) * a);

        let ts = roots::find_roots_cubic(a.as_f32(), b.as_f32(), c.as_f32(), d.as_f32());
        let ts = ts.as_ref().iter().copied().map(T::from_f32);

        ts.collect()
    }

    pub fn chain(&self, resolution: usize) -> Chain<T> {
        let mut vertices = Vec::with_capacity(resolution);

        let start = -T::ONE;
        let end = T::ONE;
        let step = (end - start) / T::from_f32(resolution as f32);
        for i in 0..=resolution {
            let t = start + step * T::from_f32(i as f32);
            vertices.push(self.get(t));
        }

        Chain { vertices }
    }
}
