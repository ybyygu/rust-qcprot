// [[file:~/Workspace/Programming/rust-libs/rust-qcprot/qcprot.note::bb2a93d1-814d-480a-a777-78afca20044a][bb2a93d1-814d-480a-a777-78afca20044a]]
#[macro_use] extern crate approx;
extern crate qcprot;

use qcprot::*;

#[test]
fn test_center_coords() {
    let (mut frag, _, _) = prepare_test_data();

    let natoms = frag.len();
    let mut arr_w = Vec::with_capacity(natoms);
    for i in 0..natoms {
        arr_w.push(i as f64 + 1.0);
    };

    let frag_expected = vec![
        [-3.172,  -2.221,  -5.400],
        [ 0.524,  -2.910,  -4.809],
        [ 0.999,   0.781,  -4.071],
        [-2.020,   0.999,  -1.779],
        [-0.809,  -2.066,   0.112],
        [ 2.182,  -0.121,   1.416],
        [-0.264,   1.822,   3.611],
    ];
    center_coords(&mut frag, &arr_w);
    for i in 0..natoms {
        for j in 0..3 {
            assert_relative_eq!(frag_expected[i][j], frag[i][j], epsilon=1e-3);
        }
    }
}

#[test]
fn test_qcprot() {
    let (mut frag_a, mut frag_b, weight) = prepare_test_data();
    let (rmsd, rot) = calc_rmsd_rotational_matrix(&mut frag_a, &mut frag_b, Some(&weight)).expect("qcprot rot");
    assert_relative_eq!(0.745016, rmsd, epsilon=1e-3);

    let rot_expected = [
         0.77227551,    0.63510272,   -0.01533190,
        -0.44544846,    0.52413614,   -0.72584914,
        -0.45295276,    0.56738509,    0.68768304,
    ];

    let rot = rot.expect("rot matrix");
    println!("{:#?}", rot);
    for i in 0..9 {
        assert_relative_eq!(rot_expected[i], rot[i], epsilon=1e-4);
    }
}
// bb2a93d1-814d-480a-a777-78afca20044a ends here
