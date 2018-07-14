// [[file:~/Workspace/Programming/rust-libs/rust-qcprot/qcprot.note::bb2a93d1-814d-480a-a777-78afca20044a][bb2a93d1-814d-480a-a777-78afca20044a]]
#[macro_use] extern crate approx;
extern crate qcprot;

use qcprot::*;

#[test]
fn test_center_coords() {
    let mut frag = vec![
        [ -2.803, -15.373, 24.556],
        [  0.893, -16.062, 25.147],
        [  1.368, -12.371, 25.885],
        [ -1.651, -12.153, 28.177],
        [ -0.440, -15.218, 30.068],
        [  2.551, -13.273, 31.372],
        [  0.105, -11.330, 33.567],
    ];

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
// bb2a93d1-814d-480a-a777-78afca20044a ends here
