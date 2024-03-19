extern crate lazy_static;
extern crate num_bigint;
extern crate num_traits;

use std::ops::{Add, BitAnd, BitOr, Mul, Sub};
use lazy_static::lazy_static;
use num_bigint::{BigInt, BigUint};
use num_traits::{Num, One, Zero};

lazy_static! {
    static ref GX: BigInt = BigInt::from_str_radix("79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798", 16).unwrap();
    static ref GY: BigInt = BigInt::from_str_radix("483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8", 16).unwrap();
    static ref N: BigInt = BigInt::from_str_radix("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141", 16).unwrap();
    static ref P: BigInt = BigInt::from_str_radix("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F", 16).unwrap();

    static ref G: Point = Point{ x: GX.clone(), y: GY.clone(), z: BigInt::one() };
    static ref I: Point = Point{ x: BigInt::zero(), y: BigInt::one(), z: BigInt::zero() };
}

struct Curve {
    p: BigInt,
    n: BigInt,
    a: BigInt,
    b: BigInt,
    gx: BigInt,
    gy: BigInt,
}

impl Curve {
    fn default() -> Curve {
        Self {
            p: P.clone(),
            n: N.clone(),
            a: BigInt::zero(),
            b: BigInt::from(7u8),
            gx: GX.clone(),
            gy: GY.clone(),
        }
    }
}

#[derive(Debug, Clone)]
struct Point {
    x: BigInt,
    y: BigInt,
    z: BigInt,
}

impl Point {
    fn add(&self, point: &Point) -> Point {
        let x1 = self.x.clone();
        let y1 = self.y.clone();
        let z1 = self.z.clone();

        let x2 = point.x.clone();
        let y2 = point.y.clone();
        let z2 = point.z.clone();

        let mut x3: BigInt = BigInt::zero();
        let mut y3: BigInt = BigInt::zero();
        let mut z3: BigInt = BigInt::zero();

        let curve = Curve::default();

        let a = &curve.a;
        let b = &curve.b;

        let b3 = modx(&(b * BigInt::from(3u8)), None);

        let mut t0 = modx(&(&x1 * &x2), None);
        let mut t1 = modx(&(&y1 * &y2), None);
        let mut t2 = modx(&(&z1 * &z2), None);
        let mut t3 = modx(&(&x1 + &y1), None);
        let mut t4 = modx(&(&x2 + &y2), None);

        t3 = modx(&(&t3 * &t4), None);
        t4 = modx(&(&t0 + &t1), None);
        t3 = modx(&(&t3 - &t4), None);
        t4 = modx(&(&x1 + &z1), None);

        let mut t5 = modx(&(&x2 + &z2), None);

        t4 = modx(&(&t4 * &t5), None);
        t5 = modx(&(&t0 + &t2), None);
        t4 = modx(&(&t4 - &t5), None);
        t5 = modx(&(&y1 + &z1), None);
        x3 = modx(&(&y2 + &z2), None);
        t5 = modx(&(&t5 * &x3), None);
        x3 = modx(&(&t1 + &t2), None);
        t5 = modx(&(&t5 - &x3), None);
        z3 = modx(&(a * &t4), None);
        x3 = modx(&(&b3 * &t2), None);
        z3 = modx(&(&x3 + &z3), None);
        x3 = modx(&(&t1 - &z3), None);
        z3 = modx(&(&t1 + &z3), None);
        y3 = modx(&(&x3 * &z3), None);
        t1 = modx(&(&t0 + &t0), None);
        t1 = modx(&(&t1 + &t0), None);
        t2 = modx(&(a * &t2), None);
        t4 = modx(&(&b3 * &t4), None);
        t1 = modx(&(&t1 + &t2), None);
        t2 = modx(&(&t0 - &t2), None);
        t2 = modx(&(a * &t2), None);
        t4 = modx(&(&t4 + &t2), None);
        t0 = modx(&(&t1 * &t4), None);
        y3 = modx(&(&y3 + &t0), None);
        t0 = modx(&(&t5 * &t4), None);
        x3 = modx(&(&t3 * &x3), None);
        x3 = modx(&(&x3 - &t0), None);
        t0 = modx(&(&t3 * &t1), None);
        z3 = modx(&(&t5 * &z3), None);
        z3 = modx(&(&z3 + &t0), None);

        Point {
            x: x3,
            y: y3,
            z: z3,
        }
    }

    fn double(&self) -> Point {
        self.add(&self)
    }

    fn mul(&self, mut n: BigInt) -> Point {
        if n == BigInt::zero() {
            return I.clone();
        }

        if self.equals(&G) {
            println!("cache...")
        }

        let mut p = I.clone();
        let mut d = self.clone();

        while n > BigInt::zero() {
            if (&n & BigInt::one()).bit(0) {
                p = p.add(&d)
            }

            n >>= 1;
            d = d.double();
        }

        p
    }

    fn from_private_key(private_key: BigInt) -> Point {
        G.mul(private_key)
    }

    fn equals(&self, other: &Point) -> bool {
        let x1 = &self.x;
        let y1 = &self.y;
        let z1 = &self.z;

        let x2 = &other.x;
        let y2 = &other.y;
        let z2 = &other.z;

        let x1z2 = modx(&(x1 * z2), None);
        let x2z1 = modx(&(x2 * z1), None);
        let y1z2 = modx(&(y1 * z2), None);
        let y2z1 = modx(&(y2 * z1), None);

        x1z2 == x2z1 && y1z2 == y2z1
    }

    fn to_hex(&self) -> String {
        let (x, y) = self.toAffine();
        let head = if (y & BigInt::one()) == BigInt::zero() { "02" } else { "03" };

        format!("{}{}", head, x.to_str_radix(16))
    }

    fn toAffine(&self) -> (BigInt, BigInt) {
        if self.equals(&I) {
            return (BigInt::zero(), BigInt::zero());
        }

        if self.z == BigInt::one() {
            return (self.x.clone(), self.y.clone());
        }

        let iz = inv(&self.z);

        (
            modx(&(&self.x * &iz), None),
            modx(&(&self.y * &iz), None)
        )
    }
}

fn inv(num: &BigInt) -> BigInt {
    let md = P.clone();

    let mut a = modx(num, Some(&md));
    let mut b = md.clone();
    let mut x = BigInt::zero();
    let mut y = BigInt::one();
    let mut u = BigInt::one();
    let mut v = BigInt::zero();

    while (a != BigInt::zero()) {
        // uses euclidean gcd algorithm
        let q = &b / &a;
        let r = &b % &a;

        let m = &x - &u * &q;
        let n = &y - &v * &q;

        b = a;
        a = r;
        x = u;
        y = v;
        u = m;
        v = n;
    }

    modx(&x, Some(&md))
}

fn modx(a: &BigInt, b: Option<&BigInt>) -> BigInt {
    let b = b.unwrap_or_else(|| &P);
    let r = a % b;

    if r >= BigInt::zero() {
        r
    } else {
        r.add(b)
    }
}


fn main() {
    let a = Point {
        y: BigInt::one(),
        x: BigInt::one(),
        z: BigInt::one(),
    };

    let b = Point {
        y: BigInt::one(),
        x: BigInt::one(),
        z: BigInt::one(),
    };

    // println!("{:#?}", a.mul(BigInt::from(2000000000000u64)))

    println!("{:#?}", Point::from_private_key(BigInt::one()).to_hex());
}
