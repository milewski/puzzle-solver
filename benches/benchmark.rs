use std::ops::AddAssign;

use bitcoin::hashes::Hash;
use bitcoin::hex::{Case, DisplayHex};
use criterion::{Criterion, criterion_group, criterion_main};
use k256::{ProjectivePoint, SecretKey};
use k256::elliptic_curve::group::GroupEncoding;
use k256::elliptic_curve::sec1::ToEncodedPoint;
use num_bigint::BigInt;
use num_traits::{Num, ToBytes};
use puzzle_solver::secp256k1::{G, Point};

fn my_benchmark_function(c: &mut Criterion) {
    // c.bench_function("sha256", |b| {
    //     b.iter(|| {
    //         do_sha256(&[
    //             0x02,
    //             0xc9, 0x49, 0x57, 0xfe, 0xef, 0xc3, 0xc4, 0x15, 0xef, 0xc3, 0x71, 0x71, 0x83, 0x36, 0x68, 0xa9,
    //             0xfe, 0xb3, 0x94, 0xa3, 0x09, 0x74, 0x01, 0xc0, 0x10, 0xc0, 0xed, 0x8b, 0xcd, 0x98, 0xaa, 0xb5
    //         ]);
    //
    //         // bitcoin::hashes::sha256::Hash::hash(&[
    //         //     0x02,
    //         //     0xc9, 0x49, 0x57, 0xfe, 0xef, 0xc3, 0xc4, 0x15, 0xef, 0xc3, 0x71, 0x71, 0x83, 0x36, 0x68, 0xa9,
    //         //     0xfe, 0xb3, 0x94, 0xa3, 0x09, 0x74, 0x01, 0xc0, 0x10, 0xc0, 0xed, 0x8b, 0xcd, 0x98, 0xaa, 0xb5
    //         // ]);
    //     });
    // });

    c.bench_function("sepc256k1", |b| {
        b.iter(|| {
            let private_key_bytes = BigInt::from_str_radix("bd70c09c15495906d394c1b7f9e80c4511777732a67a0000718f56847d29cc86", 16).unwrap();

            let mut point = Point::from_key(private_key_bytes);

            for _ in 1..1000 {
                point.add_assign();
            }

            assert_eq!("034768f74ca212178cadaf8eebcfdb70251f41669cb4cd898b1b78c8026b15fdb8", point.to_hex_string());

            // let mut point = SecretKey::from_slice(&private_key_bytes.into_parts().1.to_be_bytes())
            //     .unwrap()
            //     .public_key()
            //     .to_projective();

            // let mut point = Point::from_key(private_key_bytes);
            //
            // for _ in 1..1000 {
            //     point.add_assign(G);
            // }
            //
            // // assert_eq!("034768f74ca212178cadaf8eebcfdb70251f41669cb4cd898b1b78c8026b15fdb8", point.to_bytes().to_hex_string(Case::Lower));
        });
    });
}

// fn from_library(c: &mut Criterion) {
//     c.bench_function("bitcoin_sha256", |b| {
//         b.iter(|| {
//             bitcoin::hashes::sha256::Hash::hash(&[
//                 0x02,
//                 0xc9, 0x49, 0x57, 0xfe, 0xef, 0xc3, 0xc4, 0x15, 0xef, 0xc3, 0x71, 0x71, 0x83, 0x36, 0x68, 0xa9,
//                 0xfe, 0xb3, 0x94, 0xa3, 0x09, 0x74, 0x01, 0xc0, 0x10, 0xc0, 0xed, 0x8b, 0xcd, 0x98, 0xaa, 0xb5
//             ]);
//         });
//     });
// }

criterion_group!(benches, my_benchmark_function);
criterion_main!(benches);