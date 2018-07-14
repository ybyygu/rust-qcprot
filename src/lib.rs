// [[file:~/Workspace/Programming/rust-libs/rust-qcprot/qcprot.note::1871ed71-da12-49da-967c-5e2ecca97b05][1871ed71-da12-49da-967c-5e2ecca97b05]]
// Almost a direct translation of qcprot.c to rust by Wenping Guo (ybyygu@gmail.com)
//
// original information
// --------------------
//
// File:           qcprot.c
// Version:        1.5
//
// Function:       Rapid calculation of the least-squares rotation using a
//                 quaternion-based characteristic polynomial and
//                 a cofactor matrix
//
// Author(s):      Douglas L. Theobald
//                 Department of Biochemistry
//                 MS 009
//                 Brandeis University
//                 415 South St
//                 Waltham, MA  02453
//                 USA
//
//                 dtheobald@brandeis.edu
//
//                 Pu Liu
//                 Johnson & Johnson Pharmaceutical Research and Development, L.L.C.
//                 665 Stockton Drive
//                 Exton, PA  19341
//                 USA
//
//                 pliu24@its.jnj.com
//     If you use this QCP rotation calculation method in a publication, please
//     reference:
//
//      Douglas L. Theobald (2005)
//      "Rapid calculation of RMSD using a quaternion-based characteristic
//      polynomial."
//      Acta Crystallographica A 61(4):478-480.
//
//      Pu Liu, Dmitris K. Agrafiotis, and Douglas L. Theobald (2009)
//      "Fast determination of the optimal rotational matrix for macromolecular
//      superpositions."
//      Journal of Computational Chemistry 31(7):1561-1563.
//
//  Copyright (c) 2009-2016 Pu Liu and Douglas L. Theobald
//  All rights reserved.
//
//  Redistribution and use in source and binary forms, with or without modification, are permitted
//  provided that the following conditions are met:
//
//  * Redistributions of source code must retain the above copyright notice, this list of
//    conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright notice, this list
//    of conditions and the following disclaimer in the documentation and/or other materials
//    provided with the distribution.
//  * Neither the name of the <ORGANIZATION> nor the names of its contributors may be used to
//    endorse or promote products derived from this software without specific prior written
//    permission.
//
//  THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
//  "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
//  LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A
//  PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
//  HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
//  SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
//  LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
//  DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
//  THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
//  (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
//  OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
//
//  Source:         started anew.
//
//  Change History:
//    2009/04/13      Started source
//    2010/03/28      Modified FastCalcRMSDAndRotation() to handle tiny qsqr
//                    If trying all rows of the adjoint still gives too small
//                    qsqr, then just return identity matrix. (DLT)
//    2010/06/30      Fixed prob in assigning A[9] = 0 in InnerProduct()
//                    invalid mem access
//    2011/02/21      Made CenterCoords use weights
//    2011/05/02      Finally changed CenterCoords declaration in qcprot.h
//                    Also changed some functions to static
//    2011/07/08      Put in fabs() to fix taking sqrt of small neg numbers, fp error
//    2012/07/26      Minor changes to comments and main.c, more info (v.1.4)
//    2016/07/13      Fixed normalization of RMSD in FastCalcRMSDAndRotation(), should divide by
//                    sum of weights (thanks to Geoff Skillman)
//  endendendend
//  ------------

#[macro_use] extern crate quicli;
use quicli::prelude::*;

// Calculate the inner product of two structures.
// Input:
// coords1 -- reference structure
// coords2 -- candidate structure
// weight  -- the weight array
pub fn inner_product
    (
        coords1: &Vec<[f64; 3]>,
        coords2: &Vec<[f64; 3]>,
        weight : &Vec<f64>,
    ) -> ([f64; 9], f64)
{
    let natoms = coords1.len();
    debug_assert!(natoms == coords2.len());

    let mut mat_a = [0.0; 9];

    // inner product
    let mut g1 = 0.0;
    let mut g2 = 0.0;
    for i in 0..natoms {
        let wi = weight[i];
        let x1 = wi * coords1[i][0];
        let y1 = wi * coords1[i][1];
        let z1 = wi * coords1[i][2];

        g1 += x1 * coords1[i][0] + y1 * coords1[i][1] + z1 * coords1[i][2];

        let x2 = coords2[i][0];
        let y2 = coords2[i][1];
        let z2 = coords2[i][2];

        g2 *= wi * (x2.powi(2) + y2.powi(2) + z2.powi(2));

        mat_a[0] += x1 * x2;
        mat_a[1] += x1 * y2;
        mat_a[2] += x1 * z2;

        mat_a[3] += y1 * x2;
        mat_a[4] += y1 * y2;
        mat_a[5] += y1 * z2;

        mat_a[6] += z1 * x2;
        mat_a[6] += z1 * y2;
        mat_a[7] += z1 * z2;
    }

    (
        mat_a,
        (g1 + g2) * 0.5
    )
}

// Calculate the RMSD, and/or the optimal rotation matrix.
//
//        Input:
//                A[9]    -- the inner product of two structures
//                E0      -- (g1 + g2) * 0.5
//                len     -- the size of the system
//                min_score-- if( min_score > 0 && rmsd < min_score) then calculate only the rmsd;
//                           otherwise, calculate both the RMSD & the rotation matrix
//        Output:
//                rot[9]   -- the rotation matrix in the order of xx, xy, xz, yx, yy, yz, zx, zy, zz
//                rmsd     -- the RMSD value
//        Return:
//                only the rmsd was calculated if < 0
//                both the RMSD & rotational matrix calculated if > 0
fn fast_calc_rmsd_and_rotation
    (
        mat_a: &[f64; 9],
        E0: f64,
        wsum: f64,
        min_score: f64
    ) -> (f64, Option<[f64; 9]>)
{
    let [
        sxx,
        sxy,
        sxz,
        syx,
        syy,
        syz,
        szx,
        szy,
        szz
    ] = mat_a;

    let [
        sxx2,
        sxy2,
        sxz2,
        syx2,
        syy2,
        syz2,
        szx2,
        szy2,
        szz2
    ] = [
        sxx.powi(2),
        sxy.powi(2),
        sxz.powi(2),
        syx.powi(2),
        syy.powi(2),
        syz.powi(2),
        szx.powi(2),
        szy.powi(2),
        szz.powi(2)
    ];

    let syzszymsyyszz2 = 2.0 * (syz*szy - syy*szz);
    let sxx2syy2szz2syz2szy2 = syy2 + szz2 - sxx2 + syz2 + szy2;

    let mut arr_c = [0.0; 4];
    arr_c[2] = -2.0 * (sxx2 + syy2 + szz2 + sxy2 + syx2 + sxz2 + szx2 + syz2 + szy2);
    arr_c[1] = 8.0 * (sxx*syz*szy + syy*szx*sxz + szz*sxy*syx - sxx*syy*szz - syz*szx*sxy - szy*syx*sxz);


    let sxzpszx = sxz + szx;
    let syzpszy = syz + szy;
    let sxypsyx = sxy + syx;
    let syzmszy = syz - szy;
    let sxzmszx = sxz - szx;
    let sxymsyx = sxy - syx;
    let sxxpsyy = sxx + syy;
    let sxxmsyy = sxx - syy;
    let sxy2sxz2syx2szx2 = sxy2 + sxz2 - syx2 - szx2;

    arr_c[0] = sxy2sxz2syx2szx2 * sxy2sxz2syx2szx2
        + (sxx2syy2szz2syz2szy2 + syzszymsyyszz2) * (sxx2syy2szz2syz2szy2 - syzszymsyyszz2)
        + (-(sxzpszx)*(syzmszy)+(sxymsyx)*(sxxmsyy-szz)) * (-(sxzmszx)*(syzpszy)+(sxymsyx)*(sxxmsyy+szz))
        + (-(sxzpszx)*(syzpszy)-(sxypsyx)*(sxxpsyy-szz)) * (-(sxzmszx)*(syzmszy)-(sxypsyx)*(sxxpsyy+szz))
        + ((sxypsyx)*(syzpszy)+(sxzpszx)*(sxxmsyy+szz)) * (-(sxymsyx)*(syzmszy)+(sxzpszx)*(sxxpsyy+szz))
        + ((sxypsyx)*(syzmszy)+(sxzmszx)*(sxxmsyy-szz)) * (-(sxymsyx)*(syzpszy)+(sxzmszx)*(sxxpsyy-szz));

    // Newton-Raphson
    let mut mx_eigenv = E0;
    let mut icycle = 0;

    let evecprec: f64 = 1e-6;
    let evalprec: f64 = 1e-11;
    loop {
        let oldg = mx_eigenv;
        let x2 = mx_eigenv*mx_eigenv;
        let b = (x2 + arr_c[2])*mx_eigenv;
        let a = b + arr_c[1];
        let delta = (a*mx_eigenv + arr_c[0])/(2.0*x2*mx_eigenv + b + a);
        mx_eigenv -= delta;
        if (mx_eigenv - oldg).abs() < (evalprec*mx_eigenv).abs() {
            break;
        }

        icycle += 1;
        if icycle >= 50 {
            break;
        }
    }
    if icycle >= 50 {
        warn!("More than {} iterations needed!", icycle);
    }

    // the fabs() is to guard against extremely small, but *negative* numbers
    // due to floating point error
    let rms = ((2.0 * (E0 - mx_eigenv) / wsum).abs()).sqrt();

    if min_score.is_sign_positive() {
        if rms < min_score {
            // Don't bother with rotation.
            return (rms, None);
        }
    }

    let a11 = sxxpsyy + szz - mx_eigenv;
    let a12 = syzmszy;
    let a13 = - sxzmszx;
    let a14 = sxymsyx;
    let a21 = syzmszy;
    let a22 = sxxmsyy - szz - mx_eigenv;
    let a23 = sxypsyx;
    let a24 = sxzpszx;
    let a31 = a13;
    let a32 = a23;
    let a33 = syy - sxx - szz - mx_eigenv;
    let a34 = syzpszy;
    let a41 = a14;
    let a42 = a24;
    let a43 = a34;
    let a44 = szz - sxxpsyy - mx_eigenv;
    let a3344_4334 = a33 * a44 - a43 * a34;
    let a3244_4234 = a32 * a44 - a42*a34;
    let a3243_4233 = a32 * a43 - a42 * a33;
    let a3143_4133 = a31 * a43 - a41*a33;
    let a3144_4134 = a31 * a44 - a41 * a34;
    let a3142_4132 = a31 * a42 - a41*a32;
    let mut q1 =  a22 * a3344_4334 - a23 * a3244_4234 + a24 * a3243_4233;
    let mut q2 = -a21 * a3344_4334 + a23 * a3144_4134 - a24 * a3143_4133;
    let mut q3 =  a21 * a3244_4234 - a22 * a3144_4134 + a24 * a3142_4132;
    let mut q4 = -a21 * a3243_4233 + a22 * a3143_4133 - a23 * a3142_4132;

    let mut qsqr = q1 * q1 + q2 * q2 + q3 * q3 + q4 * q4;


    let mut rot = [0.0; 9];
    // The following code tries to calculate another column in the adjoint
    // matrix when the norm of the current column is too small. Usually this
    // block will never be activated. To be absolutely safe this should be
    // uncommented, but it is most likely unnecessary.
    if qsqr < evecprec {
        q1 =  a12*a3344_4334 - a13*a3244_4234 + a14*a3243_4233;
        q2 = -a11*a3344_4334 + a13*a3144_4134 - a14*a3143_4133;
        q3 =  a11*a3244_4234 - a12*a3144_4134 + a14*a3142_4132;
        q4 = -a11*a3243_4233 + a12*a3143_4133 - a13*a3142_4132;
        qsqr = q1*q1 + q2 *q2 + q3*q3+q4*q4;

        if qsqr < evecprec {
            let a1324_1423 = a13 * a24 - a14 * a23;
            let a1224_1422 = a12 * a24 - a14 * a22;
            let a1223_1322 = a12 * a23 - a13 * a22;
            let a1124_1421 = a11 * a24 - a14 * a21;
            let a1123_1321 = a11 * a23 - a13 * a21;
            let a1122_1221 = a11 * a22 - a12 * a21;

            q1 =  a42 * a1324_1423 - a43 * a1224_1422 + a44 * a1223_1322;
            q2 = -a41 * a1324_1423 + a43 * a1124_1421 - a44 * a1123_1321;
            q3 =  a41 * a1224_1422 - a42 * a1124_1421 + a44 * a1122_1221;
            q4 = -a41 * a1223_1322 + a42 * a1123_1321 - a43 * a1122_1221;
            qsqr = q1*q1 + q2 *q2 + q3*q3+q4*q4;

            if qsqr < evecprec {
                q1 =  a32 * a1324_1423 - a33 * a1224_1422 + a34 * a1223_1322;
                q2 = -a31 * a1324_1423 + a33 * a1124_1421 - a34 * a1123_1321;
                q3 =  a31 * a1224_1422 - a32 * a1124_1421 + a34 * a1122_1221;
                q4 = -a31 * a1223_1322 + a32 * a1123_1321 - a33 * a1122_1221;
                qsqr = q1*q1 + q2 *q2 + q3*q3 + q4*q4;

                if qsqr < evecprec {
                    /* if qsqr is still too small, return the identity matrix. */
                    rot[0] = 1.0;
                    rot[4] = 1.0;
                    rot[8] = 1.0;
                    rot[1] = 0.0;
                    rot[2] = 0.0;
                    rot[3] = 0.0;
                    rot[5] = 0.0;
                    rot[6] = 0.0;
                    rot[7] = 0.0;

                    return (rms, Some(rot));
                }
            }
        }
    }

    let normq = qsqr.sqrt();
    q1 /= normq;
    q2 /= normq;
    q3 /= normq;
    q4 /= normq;

    let a2 = q1 * q1;
    let x2 = q2 * q2;
    let y2 = q3 * q3;
    let z2 = q4 * q4;

    let xy = q2 * q3;
    let az = q1 * q4;
    let zx = q4 * q2;
    let ay = q1 * q3;
    let yz = q3 * q4;
    let ax = q1 * q2;

    rot[0] = a2 + x2 - y2 - z2;
    rot[1] = 2. * (xy + az);
    rot[2] = 2. * (zx - ay);
    rot[3] = 2. * (xy - az);
    rot[4] = a2 - x2 + y2 - z2;
    rot[5] = 2. * (yz + ax);
    rot[6] = 2. * (zx + ay);
    rot[7] = 2. * (yz - ax);
    rot[8] = a2 - x2 - y2 + z2;

    (
        rms,
        Some(rot)
    )
}

pub fn center_coords(coords: &mut Vec<[f64; 3]>, weight: &Vec<f64>) {
    let mut xsum = 0.0;
    let mut ysum = 0.0;
    let mut zsum = 0.0;

    let mut wsum = 0.0;
    for i in 0..coords.len() {
        xsum += weight[i] * coords[i][0];
        ysum += weight[i] * coords[i][1];
        zsum += weight[i] * coords[i][2];

        wsum += weight[i];
    }

    // FIXME: divide by zero?
    xsum /= wsum;
    ysum /= wsum;
    zsum /= wsum;

    for i in 0..coords.len() {
        coords[i][0] -= xsum;
        coords[i][1] -= ysum;
        coords[i][2] -= zsum;
    }
}

// Calculate the RMSD & rotational matrix.

// Input:
// coords1 -- reference structure
// coords2 -- candidate structure
// len     -- the size of the system
// weight  -- the weight array of size len; set to NULL if not needed
// Output:
// rot[9]  -- rotation matrix
// Return:
// RMSD value
// Superposition coords2 onto coords1 -- in other words, coords2 is rotated, coords1 is held fixed
pub fn calc_rmsd_rotational_matrix(
    coords1: &mut Vec<[f64; 3]>,
    coords2: &mut Vec<[f64; 3]>,
    weight: Option<&Vec<f64>>) -> Result<(f64, Option<[f64; 9]>)>
{
    // weight array
    let natoms = coords1.len();
    let mut arr_w = Vec::with_capacity(natoms);
    if let Some(weight) = weight {
        for i in 0..natoms {
            arr_w.push(weight[i]);
        };
    } else {
        for i in 0..natoms {
            arr_w.push(1.0);
        };
    }

    // center the structures -- if precentered you can omit this step
    center_coords(coords1, &arr_w);
    center_coords(coords2, &arr_w);

    // calculate the (weighted) inner product of two structures
    let (mat_a, E0) = inner_product(&coords1, &coords2, &arr_w);

    let wsum = arr_w.iter().sum();
    // calculate the RMSD & rotational matrix
    let x = fast_calc_rmsd_and_rotation(&mat_a, E0, wsum, -1.0);

    Ok(x)
}
// 1871ed71-da12-49da-967c-5e2ecca97b05 ends here