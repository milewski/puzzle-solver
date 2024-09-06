use std::cell::LazyCell;
use std::ops::{AddAssign, Rem};

use bitcoin::hex::{Case, DisplayHex};
use k256::{ProjectivePoint, SecretKey};
use k256::elliptic_curve::group::GroupEncoding;
use k256::elliptic_curve::point::AffineCoordinates;
use k256::elliptic_curve::sec1::ToEncodedPoint;
use num_bigint::BigInt;
use num_traits::{Euclid, Num, One, Pow, ToBytes, Zero};

const P: LazyCell<BigInt> = LazyCell::new(|| BigInt::from_str_radix("115792089237316195423570985008687907853269984665640564039457584007908834671663", 10).unwrap());

pub const G: LazyCell<Point> = LazyCell::new(|| {
    Point::new(
        "55066263022277343669578718895168534326250603453777594175500187360389116729240",
        "32670510020758816978083085130507043184471273380659243275938904335757337482424",
        10
    )
});

#[inline]
fn egcd(a: BigInt, b: BigInt) -> (BigInt, BigInt, BigInt) {
    if a.is_zero() {
        return (b, BigInt::zero(), BigInt::one());
    }

    let (gcd, x1, y1) = egcd(b.clone().rem(&a), a.clone());

    let x = y1 - (b / a) * x1.clone();
    let y = x1;

    (gcd, x, y)
}

fn inverse2(a: BigInt, m: &BigInt) -> BigInt {
    let (gcd, x, _) = egcd(a.clone(), m.clone());

    if gcd.is_one() == false {
        return inverse2(a, &m);
    }

    (x % m + m) % m
}

fn inverse(a: BigInt, m: &BigInt) -> BigInt {
    let mut m_mutable = m.clone();
    // let m_orig = m_mutable.clone();
    let mut a = a.rem_euclid(&m_mutable); // make sure a is positive
    let mut y_prev = BigInt::from(0);
    let mut y = BigInt::from(1);
    while a > BigInt::from(1) {
        let q = &m_mutable / &a;

        let y_before = y.clone(); // store current value of y
        y = &y_prev - q * &y; // calculate new value of y
        y_prev = y_before; // set previous y value to the old y value

        let a_before = a.clone(); // store current value of a
        a = &m_mutable % &a; // calculate new value of a
        m_mutable = a_before; // set m to the old a value
    }
    y.rem_euclid(&m)
}

#[derive(Debug, PartialOrd, PartialEq, Clone)]
pub struct Point {
    x: BigInt,
    y: BigInt,
}

impl Point {
    pub fn from_key(key: BigInt) -> Self {
        G.multiply(key)
    }

    #[inline]
    pub fn to_hex_string(&self) -> String {
        if self.y.rem_euclid(&BigInt::from(2u8)).is_zero() {
            format!("04{:064x}", self.x)
        } else {
            format!("03{:064x}", self.x)
        }
    }

    pub fn add_assign(&mut self) {
        self.add(G.clone());
    }

    #[inline]
    fn new(x: &str, y: &str, base: u32) -> Self {
        Point {
            x: BigInt::from_str_radix(x, base).unwrap(),
            y: BigInt::from_str_radix(y, base).unwrap(),
        }
    }

    #[inline]
    fn multiply(&self, k: BigInt) -> Point {
        let mut current = self.clone();

        let binary = k.to_str_radix(2);

        for char in binary.chars().skip(1) {
            current.double();

            if char == '1' {
                current.add(self.clone());
            }
        }

        current
    }

    #[inline]
    fn double(&mut self) {
        let a = BigInt::zero();
        // let p = BigInt::from_str_radix("115792089237316195423570985008687907853269984665640564039457584007908834671663", 10).unwrap();

        let p = P.clone();

        let x_point = &self.x;
        let y_point = &self.y;

        let numerator = ((3 * x_point.pow(2)) + a) % &p;
        let slope: BigInt = (numerator * inverse((2 * y_point) % &p, &p)) % &p;

        // x = slope² - 2 * x₁
        let mut x = ((&slope).pow(2) - (2 * x_point)) % &p;

        if x < BigInt::zero() {
            x += &p;
        }

        // y = slope * (x₁ - x) - y₁
        let mut y = ((&slope * (x_point - &x)) - y_point) % &p;

        if y < BigInt::zero() {
            y += &p;
        }

        self.x = x;
        self.y = y;
    }

    #[inline]
    fn add(&mut self, point: Point) {
        let p = BigInt::from_str_radix("115792089237316195423570985008687907853269984665640564039457584007908834671663", 10).unwrap();

        if self == &point {
            return self.double();
        }

        let inverse = inverse(&self.x - &point.x, &P);
        let slope: BigInt = ((&self.y - &point.y) * inverse).rem_euclid(&P);

        let x = ((&slope).pow(2) - &self.x - &point.x).rem_euclid(&P);
        let y = ((slope * (&self.x - &x)) - &self.y).rem_euclid(&P);

        self.x = x;
        self.y = y;
    }
}

#[cfg(test)]
mod test {
    use num_bigint::BigInt;
    use num_traits::Num;

    use crate::secp256k1::{inverse, Point};

    #[test]
    fn test_add() {
        let test = [
            (
                ("103564036059960834835288257890889011375560051955804307675456865601075274494119", "37185091499315218754876212693101483136144979151655211171838067782794583373176"),
                ("43370625237413118313938591878007152532653693563196697827546440437974435933805", "29966507042732192471513017660016609642213677599774555345316407547155330737994"),
                ("82649014335650331581317251560086993097817778802246944200496497988191568377141", "18915745816807602105288678214496341685015967121116262908233175746282866595289"),
            ),
            (
                ("103564036059960834835288257890889011375560051955804307675456865601075274494119", "37185091499315218754876212693101483136144979151655211171838067782794583373176"),
                ("103564036059960834835288257890889011375560051955804307675456865601075274494119", "37185091499315218754876212693101483136144979151655211171838067782794583373176"),
                ("14750936993675825599388877536893198747942088452575310220022598531596157710646", "46876579962816719765042665428854459877898743433823224462433543136710091875640"),
            ),
            (
                ("3813482466601298440478130162507364671547133469830831004782044848339711022054", "35360900530515108470737097369724908075943993307289374659204941212018284340621"),
                ("55066263022277343669578718895168534326250603453777594175500187360389116729240", "32670510020758816978083085130507043184471273380659243275938904335757337482424"),
                ("90226212619380785921550130650961176954420125375412249131413171972699784160756", "21140031264753162582273490661381857551882386964655621422316808671948714081186"),
            )
        ];

        for ((a, b, expectation)) in test {
            let mut point1 = Point::new(a.0, a.1, 10);
            let point2 = Point::new(b.0, b.1, 10);

            point1.add(point2);

            assert_eq!(point1, Point::new(expectation.0, expectation.1, 10));
        }
    }

    #[test]
    fn test_multiply() {
        let test = [
            ("21", "24049875635381557237058143631624836741422505207761609709712554171343558302165", "22669890352939653242079781319904043788036611953081321775127194249638113810828"),
            ("50", "18752372355191540835222161239240920883340654532661984440989362140194381601434", "88478450163343634110113046083156231725329016889379853417393465962619872936244"),
            ("85686344596784521954765906439059362372334158125567466952138070287338244263046", "7734829935239794421881575032384692787053918982892539064738868453288849992386", "69798220913602732414539727403794149830463875302138082662269677346384798292718"),
        ];

        let g = Point::new(
            "55066263022277343669578718895168534326250603453777594175500187360389116729240",
            "32670510020758816978083085130507043184471273380659243275938904335757337482424",
            10
        );

        for ((key, x, y)) in test {
            assert_eq!(
                g.multiply(BigInt::from_str_radix(key, 10).unwrap()),
                Point::new(x, y, 10)
            );
        }
    }

    #[test]
    fn test_double() {
        let points = [
            (
                ("67021788774070519216873027028415755838031127946365785834333207609336897749720", "14759327695212435860865071988445239506497726870916335976224291980238136260511"),
                ("70832442759962948663411453488202085868848870296432219778612359739209574201761", "29603980038673079515543789152395845107725407169050731005481048229469173059741"),
            ),
            (
                ("55066263022277343669578718895168534326250603453777594175500187360389116729240", "32670510020758816978083085130507043184471273380659243275938904335757337482424"),
                ("89565891926547004231252920425935692360644145829622209833684329913297188986597", "12158399299693830322967808612713398636155367887041628176798871954788371653930"),
            )
        ];

        for ((x, y), expectation) in points {
            let mut point = Point::new(x, y, 10);
            point.double();

            assert_eq!(point, Point::new(expectation.0, expectation.1, 10));
        }
    }

    #[test]
    fn test_inverse() {
        let modulus = BigInt::from_str_radix("115792089237316195423570985008687907853269984665640564039457584007908834671663", 10).unwrap();

        let tests = [
            ("1", "1"),
            ("11111111111111111111111111111111111111111111111111111111111111111111111111111", "90130748869828070733377743534399355837839194050565991451424226268483214143944"),
            ("86844066927987146567678238756515930889628173209306178286953872356138621120753", "56698580177502738523426123429425159596775130962962197021687897647452353804423"),
            ("99999999999999999999999999999999999999999999999999999999999999999999999999999", "35746103038273384620057745950197241282708795931316346614482154920477875943030"),
        ];

        for (number, expected) in tests {
            let number = BigInt::from_str_radix(number, 10).unwrap();

            assert_eq!(inverse(number, &modulus), BigInt::from_str_radix(expected, 10).unwrap());
        }
    }
}

#[test]
fn example() {
    let private_key_bytes = BigInt::from_str_radix("bd70c09c15495906d394c1b7f9e80c4511777732a67a0000718f56847d29cc86", 16).unwrap();
    let mut point = Point::from_key(private_key_bytes);

    for _ in 1..1000 {
        point.add_assign();
    }

    assert_eq!("034768f74ca212178cadaf8eebcfdb70251f41669cb4cd898b1b78c8026b15fdb8", point.to_hex_string());
}

fn do_secp256k1() {
    // let x = BigInt::from_str_radix("55066263022277343669578718895168534326250603453777594175500187360389116729240", 10).unwrap();
    // let y = BigInt::from_str_radix("32670510020758816978083085130507043184471273380659243275938904335757337482424", 10).unwrap();

    let a = BigInt::zero();
    let b = 7;
    let p = BigInt::from_str_radix("115792089237316195423570985008687907853269984665640564039457584007908834671663", 10).unwrap();
    let n = BigInt::from_str_radix("115792089237316195423570985008687907852837564279074904382605163141518161494337", 10).unwrap();
    let g = (
        BigInt::from_str_radix("55066263022277343669578718895168534326250603453777594175500187360389116729240", 10).unwrap(),
        BigInt::from_str_radix("32670510020758816978083085130507043184471273380659243275938904335757337482424", 10).unwrap(),
    );

    let point1 = (
        BigInt::from_str_radix("7734829935239794421881575032384692787053918982892539064738868453288849992386", 10).unwrap(),
        BigInt::from_str_radix("69798220913602732414539727403794149830463875302138082662269677346384798292718", 10).unwrap(),
    );

    // let point2 = (
    //     BigInt::from_str_radix("79128294283794967005165487246559269088207262841953746818168185216874675049964", 10).unwrap(),
    //     BigInt::from_str_radix("24667349536032243904442110979746520585288268553755982637938373372899272149933", 10).unwrap(),
    // );

    // let result = add(point1, g.clone(), a.clone(), p.clone()).unwrap();

    // println!("{} {}", result.0.to_be_bytes().to_hex_string(Case::Lower), result.1.to_be_bytes().to_hex_string(Case::Lower));

    // let private_key = SecretKey::random(&mut rand::thread_rng());

    let private_key_bytes = BigInt::from_str_radix("bd70c09c15495906d394c1b7f9e80c4511777732a67a0000718f56847d29cc86", 16).unwrap();

    println!("PrivateKey: {:02x?}", private_key_bytes.to_be_bytes().to_hex_string(Case::Lower));

    // println!("KEY: {:02x?}", multiply(private_key_bytes.clone(), &g.clone(), &a.clone(), &p.clone()).unwrap().0.to_be_bytes().to_hex_string(Case::Lower));

    let mut point = SecretKey::from_slice(&private_key_bytes.into_parts().1.to_be_bytes())
        .unwrap()
        .public_key()
        .to_projective();

    println!("ORIGINAL X: {:02x?}", point.to_encoded_point(false).x().unwrap().to_hex_string(Case::Lower));
    println!("ORIGINAL Y: {:02x?}", point.to_encoded_point(false).y().unwrap().to_hex_string(Case::Lower));

    point.add_assign(ProjectivePoint::GENERATOR);

    println!("ADDED:    {:02x?}", point.to_bytes().to_hex_string(Case::Lower));
}