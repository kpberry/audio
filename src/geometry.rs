use std::{fmt, ops};
use std::fmt::{Display, Formatter};

pub trait Visible {
    fn ray_intersection(&self, r: &L) -> Option<P>;
    fn ray_reflection(&self, r: &L) -> Option<L>;
}

#[derive(Clone)]
pub struct P {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl P {
    pub fn dist(&self, p: &P) -> f64 {
        (self - p).n()
    }

    pub fn lproj(&self, l: &L) -> P {
        &l.a + &(self - &l.a).proj(&l.v())
    }

    pub fn tproj(&self, t: &T) -> P {
        let ap = self - &t.a;
        &t.a + &(&ap - &ap.proj(&t.normal()))
    }

    pub fn pproj(&self, t: &T) -> P {
        // projection onto plane defined by triangle t
        self.tproj(t)
    }

    pub fn ray_dist(&self, r: &L) -> f64 {
        (&(self - &r.a) - &(&self.lproj(r) - &r.a)).n()
    }

    pub fn line_dist(&self, l: &L) -> f64 {
        let lv = l.v();

        let ac = self - &l.a;
        if ac.dot(&lv) > 0. {
            let bc = self - &l.b;
            if bc.dot(&lv) > 0. {
                self.dist(&l.b)
            } else {
                self.ray_dist(&l)
            }
        } else {
            self.dist(&l.a)
        }
    }

    pub fn plane_dist(&self, t: &T) -> f64 {
        // dist to plane defined by triangle t
        (&(self - &t.a) - &(&self.pproj(t) - &t.a)).n()
    }

    pub fn triangle_dist(&self, t: &T) -> f64 {
        let u = &t.b - &t.a;
        let v = &t.c - &t.a;
        let w = self - &t.a;
        let n = u.cross(&v);

        let gamma = u.cross(&w).dot(&n) / n.n2();
        if 0. <= gamma && gamma <= 1. {
            let beta = w.cross(&v).dot(&n) / n.n2();
            if 0. <= beta && beta <= 1. {
                let alpha = 1. - gamma - beta;
                if 0. <= alpha && alpha <= 1. {
                    return self.plane_dist(&t);
                }
            }
        }
        self.dist(&t.a).min(self.dist(&t.b).min(self.dist(&t.c)))
    }
}

impl Display for P {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "P({}, {}, {})", self.x, self.y, self.z)
    }
}

impl ops::Add<&V> for &P {
    type Output = P;
    fn add(self, v: &V) -> P {
        P {
            x: self.x + v.x,
            y: self.y + v.y,
            z: self.z + v.z,
        }
    }
}

impl ops::Add<&P> for &P {
    type Output = P;
    fn add(self, p: &P) -> P {
        P {
            x: self.x + p.x,
            y: self.y + p.y,
            z: self.z + p.z,
        }
    }
}

impl ops::Sub<&P> for &P {
    type Output = V;
    fn sub(self, p: &P) -> V {
        V {
            x: self.x - p.x,
            y: self.y - p.y,
            z: self.z - p.z,
        }
    }
}

pub struct V {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl V {
    fn new(x: f64, y: f64, z: f64) -> V {
        V { x, y, z }
    }

    fn n(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    fn n2(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    fn dot(&self, v: &V) -> f64 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }

    fn cross(&self, v: &V) -> V {
        V {
            x: self.y * v.z - self.z * v.y,
            y: self.z * v.x - self.x * v.z,
            z: self.x * v.y - self.y * v.x,
        }
    }

    fn proj(&self, v: &V) -> V {
        v * (self.dot(v) / v.n2())
    }

    fn pnormal(&self, pv: &V) -> V {
        let c = self.cross(&pv);
        self.cross(&c)
    }

    fn u(&self) -> V {
        self / self.n()
    }
}

impl ops::Add<&V> for &V {
    type Output = V;
    fn add(self, v: &V) -> V {
        V {
            x: self.x + v.x,
            y: self.y + v.y,
            z: self.z + v.z,
        }
    }
}

impl ops::Sub<&V> for &V {
    type Output = V;
    fn sub(self, v: &V) -> V {
        V {
            x: self.x - v.x,
            y: self.y - v.y,
            z: self.z - v.z,
        }
    }
}

impl ops::Mul<f64> for &V {
    type Output = V;
    fn mul(self, f: f64) -> V {
        V {
            x: self.x * f,
            y: self.y * f,
            z: self.z * f,
        }
    }
}

impl ops::Div<f64> for &V {
    type Output = V;
    fn div(self, f: f64) -> V {
        V {
            x: self.x / f,
            y: self.y / f,
            z: self.z / f,
        }
    }
}

impl ops::Neg for &V {
    type Output = V;

    fn neg(self) -> Self::Output {
        V {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Display for V {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "V({}, {}, {})", self.x, self.y, self.z)
    }
}

pub struct L {
    pub a: P,
    pub b: P,
}

impl L {
    pub fn v(&self) -> V {
        &self.b - &self.a
    }

    pub fn contains(&self, p: &P) -> bool {
        p.line_dist(self) < 0.00001
    }

    pub fn len(&self) -> f64 {
        self.v().n()
    }
}

impl Visible for L {
    fn ray_intersection(&self, r: &L) -> Option<P> {
        let n = self.v().pnormal(&r.v());
        let rv = r.v();
        let num = (&self.a - &r.a).dot(&n);
        let den = rv.dot(&n);
        let mut d = 0.;
        if den == 0. {
            if num != 0. {
                return None;
            }
        } else {
            d = num / den;
        }
        let i = &r.a + &(&rv * d);
        if rv.dot(&(&i - &r.a)) > 0. && self.contains(&i) {
            Some(i)
        } else {
            None
        }
    }

    fn ray_reflection(&self, r: &L) -> Option<L> {
        match self.ray_intersection(r) {
            None => None,
            Some(q) => {
                let v = r.v().u();
                let n = self.v().pnormal(&v).u();
                let direction = &v - &(&v.proj(&n) * 2.);
                let end = &q + &direction;
                Some(L { a: q, b: end })
            }
        }
    }
}

impl Display for L {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "L({}, {})", self.a, self.b)
    }
}

pub struct T {
    pub a: P,
    pub b: P,
    pub c: P,
}

impl T {
    fn normal(&self) -> V {
        (&self.b - &self.a).cross(&(&self.c - &self.a))
    }

    fn contains(&self, p: &P) -> bool {
        p.triangle_dist(&self) < 0.00001
    }
}

impl Visible for T {
    fn ray_intersection(&self, r: &L) -> Option<P> {
        let n = self.normal();
        let rv = r.v();
        let num = (&self.a - &r.a).dot(&n);
        let den = rv.dot(&n);
        let mut d = 0.;
        if den == 0. {
            if num != 0. {
                return None;
            }
        } else {
            d = num / den;
        }
        let i = &r.a + &(&rv * d);
        if rv.dot(&(&i - &r.a)) > 0. && self.contains(&i) {
            Some(i)
        } else {
            None
        }
    }

    fn ray_reflection(&self, r: &L) -> Option<L> {
        match self.ray_intersection(r) {
            None => None,
            Some(q) => {
                let v = r.v().u();
                let n = self.normal().u();
                let direction = &v - &(&v.proj(&n) * 2.);
                let end = &q + &direction;
                Some(L { a: q, b: end })
            }
        }
    }
}

impl Display for T {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "T({}, {}, {})", self.a, self.b, self.c)
    }
}

pub struct Q {
    pub a: P,
    pub b: P,
    pub c: P,
    pub d: P,
    ta: T,
    tb: T,
}

impl Q {
    fn new(a: P, b: P, c: P, d: P) -> Q {
        Q {
            a: a.clone(),
            b: b.clone(),
            c: c.clone(),
            d: d.clone(),
            // we can strategically skip some clones here
            // equivalent to T {a: a, b: b, c: c} and T {a: a, b: c, c: d}
            ta: T {
                a: a.clone(),
                b,
                c: c.clone(),
            },
            tb: T {
                a,
                b: c,
                c: d.clone(),
            },
        }
    }
}

impl Visible for Q {
    fn ray_intersection(&self, r: &L) -> Option<P> {
        self.ta.ray_intersection(r).or(self.tb.ray_intersection(r))
    }

    fn ray_reflection(&self, r: &L) -> Option<L> {
        self.ta.ray_reflection(r).or(self.tb.ray_reflection(r))
    }
}

impl Display for Q {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Q({}, {}, {}, {})", self.a, self.b, self.c, self.d)
    }
}

pub struct S {
    pub c: P,
    pub r: f64,
}

impl Visible for S {
    fn ray_intersection(&self, r: &L) -> Option<P> {
        let p = self.c.lproj(r);
        let d = p.dist(&self.c);
        if self.r < d {
            None
        } else {
            let offset = (self.r * self.r - d * d).sqrt();
            let u = (&p - &r.a).u();
            let (i1, i2) = (&p + &(&u * offset), &p + &(&(-&u) * offset));
            Some(if i1.dist(&r.a) < i2.dist(&r.a) {
                i1
            } else {
                i2
            })
        }
    }

    fn ray_reflection(&self, r: &L) -> Option<L> {
        match self.ray_intersection(r) {
            None => None,
            Some(q) => {
                let v = r.v().u();
                let n = (&q - &self.c).u();
                let direction = &v - &(&v.proj(&n) * 2.);
                let end = &q + &direction;
                Some(L { a: q, b: end })
            }
        }
    }
}

impl Display for S {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "S({}, {})", self.c, self.r)
    }
}

pub fn make_box(w: f64, h: f64, d: f64, c: P, f: bool) -> Vec<Q> {
    let woff = V::new(w, 0., 0.);
    let hoff = V::new(0., h, 0.);
    let doff = V::new(0., 0., d);
    let mut result = vec![
        Q::new(c.clone(), &c + &woff, &c + &(&woff + &hoff), &c + &hoff), // back
        Q::new(
            &c + &woff,
            &c + &(&woff + &hoff),
            &c + &(&woff + &(&hoff + &doff)),
            &c + &(&woff + &doff),
        ), // right
        Q::new(c.clone(), &c + &hoff, &c + &(&hoff + &doff), &c + &doff), // left
        Q::new(c.clone(), &c + &doff, &c + &(&woff + &doff), &c + &woff), // top,
        Q::new(
            &c + &hoff,
            &c + &(&hoff + &doff),
            &c + &(&woff + &(&hoff + &doff)),
            &c + &(&woff + &hoff),
        ), // bottom
    ];
    if f {
        result.push(Q::new(
            &c + &doff,
            &c + &(&woff + &doff),
            &c + &(&woff + &(&hoff + &doff)),
            &c + &(&hoff + &doff),
        ));
    }
    return result;
}