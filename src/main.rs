// [[file:~/Workspace/Programming/rust-libs/rust-qcprot/qcprot.note::d74a3989-cd72-4a28-9633-ffeed6b2bf96][d74a3989-cd72-4a28-9633-ffeed6b2bf96]]
#[macro_use] extern crate quicli;
use quicli::prelude::*;

extern crate qcprot;
use qcprot::calc_rmsd_rotational_matrix;

fn main() -> Result<()> {
    let mut frag_a = vec![
        [ -2.803, -15.373, 24.556],
        [  0.893, -16.062, 25.147],
        [  1.368, -12.371, 25.885],
        [ -1.651, -12.153, 28.177],
        [ -0.440, -15.218, 30.068],
        [  2.551, -13.273, 31.372],
        [  0.105, -11.330, 33.567],
    ];


    let mut frag_b = vec![
        [-14.739, -18.673, 15.040],
        [-12.473, -15.810, 16.074],
        [-14.802, -13.307, 14.408],
        [-17.782, -14.852, 16.171],
        [-16.124, -14.617, 19.584],
        [-15.029, -11.037, 18.902],
        [-18.577, -10.001, 17.996],
    ];

    let mut weight = [0.0; 7];
    for i in 0..7 {
        weight[i] = i as f64 + 1.0;
    }

    println!("Coords before centering:");

    println!("{:#?}", frag_a);
    println!("{:#?}", frag_b);

    let x = calc_rmsd_rotational_matrix(&mut frag_a, &mut frag_b, Some(&weight.to_vec()))?;
    println!("{:#?}", x);

    Ok(())
}
// d74a3989-cd72-4a28-9633-ffeed6b2bf96 ends here
