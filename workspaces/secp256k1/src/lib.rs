extern crate lazy_static;
extern crate num_bigint;
extern crate num_traits;

use std::ops::{Add, Mul};

use lazy_static::lazy_static;
use num_bigint::BigInt;
use num_traits::{Num, One, Signed, ToPrimitive, Zero};

lazy_static! {
    static ref GX: BigInt = BigInt::from_str_radix("79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798", 16).unwrap();
    static ref GY: BigInt = BigInt::from_str_radix("483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8", 16).unwrap();
    static ref N: BigInt = BigInt::from_str_radix("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141", 16).unwrap();
    static ref P: BigInt = BigInt::from_str_radix("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F", 16).unwrap();

    pub static ref G: Point = Point { x: GX.clone(), y: GY.clone(), z: BigInt::one() };
    static ref I: Point = Point { x: BigInt::zero(), y: BigInt::one(), z: BigInt::zero() };

    static ref COMPUTED: Vec<Point> = precompute();
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
pub struct Point {
    x: BigInt,
    y: BigInt,
    z: BigInt,
}

impl Point {
    pub fn add(&self, point: &Point) -> Point {
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

    pub fn mul(&self, mut n: BigInt) -> Point {
        if n == BigInt::zero() {
            return I.clone();
        }

        if self.equals(&G) {
            return wNAF(n).0;
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

    pub fn from_private_key(private_key: BigInt) -> Point {
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

    fn negate(&self) -> Point {
        Self {
            x: self.x.clone(),
            y: modx(&-self.y.clone(), None),
            z: self.z.clone(),
        }
    }

    pub fn to_hex(&self) -> String {
        let (x, y) = self.to_affine();
        let head = if (y & BigInt::one()) == BigInt::zero() { "02" } else { "03" };

        format!("{}{}", head, x.to_str_radix(16))
    }

    pub fn to_bytes(&self) -> [u8; 33] {
        let (x, y) = self.to_affine();
        let head = if (&y & &BigInt::one()) == BigInt::zero() { 0x02 } else { 0x03 };

        let mut bytes = [0; 33];

        bytes[0] = head;

        let (_, x_bytes) = x.to_bytes_be();

        for (index, x_byte) in x_bytes.into_iter().enumerate() {
            bytes[index + 1] = x_byte;
        }

        bytes
    }

    pub fn to_affine(&self) -> (BigInt, BigInt) {
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

fn precompute() -> Vec<Point> {
    let w = 8;
    let mut points: Vec<Point> = vec![];
    let windows = 256 / w + 1;

    let mut p = G.clone();
    let mut b = p.clone();

    for _ in 0..windows {
        b = p.clone();                                              // any time Gx multiplication is done.
        points.push(b.clone());

        for _ in 1..(1 << (w - 1)) {
            b = b.add(&p);
            points.push(b.clone());
        }

        p = b.double();
    }

    points
}

fn wNAF(mut n: BigInt) -> (Point, Point) {
    let comp = &COMPUTED;

    let W: u32 = 8;
    let mut p = I.clone();
    let mut f = G.clone();

    let windows = 1 + 256 / W;
    let wsize = 2u32.pow(W - 1);
    let mask = BigInt::from(2u32.pow(W) - 1);
    let maxNum = 2u32.pow(W);
    let shiftBy = W;

    for w in 0..windows {
        let off = w * wsize;
        let mut wbits = &n & &mask;

        n >>= shiftBy;

        if wbits > BigInt::from(wsize) {
            wbits -= maxNum;
            n += BigInt::one();
        }

        let off1 = off;
        let off2: BigInt = off + wbits.abs() - 1;

        let cnd1 = w % 2 != 0;
        let cnd2 = wbits < BigInt::zero();

        if (wbits == BigInt::zero()) {
            f = f.add(&neg(cnd1, comp.get(off1 as usize).unwrap()));                 // bits are 0: add garbage to fake point
        } else {
            let usize_off2: usize = off2.to_usize().unwrap();
            p = p.add(&neg(cnd2, comp.get(usize_off2).unwrap()));                 // bits are 1: add to result point
        }
    }

    (p, f)
}

fn neg(cnd: bool, p: &Point) -> Point {
    if cnd {
        p.negate()
    } else {
        p.clone()
    }
}

fn main() {

    // println!("{:#?}", a.mul(BigInt::from(2000000000000u64)))

    // println!("{:#?}", Point::from_private_key(BigInt::one()).to_hex());
}
