use fenris::element::{Tet20Element, Tet4Element, VolumetricFiniteElement};
use fenris::error::estimate_element_L2_error;
use fenris::nalgebra::coordinates::XYZ;
use fenris::nalgebra::{DVector, DVectorSlice, Point3, Vector1, Vector2, U3};
use fenris::quadrature;
use fenris::quadrature::{Quadrature, QuadraturePair3d};
use matrixcompare::assert_scalar_eq;
use std::ops::Deref;
use util::flatten_vertically;

// TODO: Port this to the library proper?
fn transform_quadrature_to_physical_domain<Element>(
    element: &Element,
    weights: &[f64],
    points: &[Point3<f64>],
) -> QuadraturePair3d<f64>
where
    Element: VolumetricFiniteElement<f64, GeometryDim = U3>,
{
    weights
        .iter()
        .zip(points)
        .map(|(w, xi)| {
            let j_det = element.reference_jacobian(xi).determinant().abs();
            (w * j_det, element.map_reference_coords(xi))
        })
        .unzip()
}

fn arbitrary_tet20_element() -> Tet20Element<f64> {
    let a = Point3::new(2.0, 0.0, 1.0);
    let b = Point3::new(3.0, 4.0, 1.0);
    let c = Point3::new(1.0, 1.0, 2.0);
    let d = Point3::new(3.0, 1.0, 4.0);
    let tet4_element = Tet4Element::from_vertices([a, b, c, d]);
    Tet20Element::from(&tet4_element)
}

#[test]
#[allow(non_snake_case)]
fn test_element_L2_error_scalar() {
    // We define two functions u1 and u2. u1 is a high order polynomial and u2 is a polynomial
    // that can be exactly represented in some element (Tet20 in this case).
    // Then, given a quadrature rule on the element, we can compute the integral of the function
    //  ||e||^2       e = u1 - u2
    // in two ways:
    //  1. directly with a high order quadrature
    //  2. by computing the L2 error with u = u1 and u_h = u2
    // This allows us to test the L2 error computation routine.

    let u1 = |x: &Point3<f64>| {
        let &XYZ { x, y, z } = x.deref();
        // A polynomial of total order 5
        6.0 * x.powi(5) + 2.0 * y.powi(5) - 2.0 * z.powi(5) + 3.0 * x * y.powi(3) * z
            - 2.0 * y.powi(3)
            + x.powi(2) * y.powi(2)
            + 3.0 * x
            + 2.0 * y
            - 3.0 * z
            - 6.0
    };
    let u2 = |x: &Point3<f64>| {
        let &XYZ { x, y, z } = x.deref();
        // A polynomial of total order 3
        6.0 * x.powi(3) - 2.0 * y.powi(3)
            + 4.0 * z.powi(3)
            + 2.0 * x.powi(2) * y
            + 4.0 * y.powi(2) * z
            + x.powi(2)
            - y.powi(3)
            + 5.0 * x * y * z
            + 2.0 * x
            + 3.0 * y
            - 5.0 * z
            + 2.0
    };
    let u = |x: &Point3<f64>| u1(x) - u2(x);

    // TODO: Use some arbitrary element rather than reference element
    let element = arbitrary_tet20_element();
    let u_h_element = DVector::from_vec(element.vertices().iter().map(u2).collect());

    // Use a quadrature rule with sufficient strength such that it can exactly capture the error
    // (since we compute a squared norm, we need double the polynomial degree)
    let (weights, points) = quadrature::total_order::tetrahedron(10).unwrap();
    let mut basis_buffer = vec![3.0; 20];
    let L2_error_computed = estimate_element_L2_error(
        &element,
        |x| Vector1::new(u1(x)),
        DVectorSlice::from(&u_h_element),
        &weights,
        &points,
        &mut basis_buffer,
    );

    let L2_error_expected = {
        let (weights, points) =
            transform_quadrature_to_physical_domain(&element, &weights, &points);
        let u_squared_norm = |x: &Point3<f64>| u(x).powi(2);
        (weights, points).integrate(u_squared_norm).sqrt()
    };

    assert_scalar_eq!(
        L2_error_computed,
        L2_error_expected,
        comp = abs,
        tol = 1e-12
    );
}

#[test]
#[allow(non_snake_case)]
fn test_element_L2_error_vector() {
    // This test is completely analogous to the scalar test, it just tests vector-valued
    // functions intead

    let u1 = |x: &Point3<f64>| {
        let &XYZ { x, y, z } = x.deref();
        // A polynomial of total order 5
        let u1_1 = 6.0 * x.powi(5) + 2.0 * y.powi(5) - 2.0 * z.powi(5) + 3.0 * x * y.powi(3) * z
            - 2.0 * y.powi(3)
            + x.powi(2) * y.powi(2)
            + 3.0 * x
            + 2.0 * y
            - 3.0 * z
            - 6.0;
        let u1_2 = 3.0 * x.powi(5) - 3.0 * y.powi(5)
            + 2.0 * z.powi(5)
            + 3.0 * x.powi(3) * y * z
            + 4.0 * x
            + 2.0 * y
            + 15.0;
        Vector2::new(u1_1, u1_2)
    };
    let u2 = |x: &Point3<f64>| {
        let &XYZ { x, y, z } = x.deref();
        // A polynomial of total order 3
        let u2_1 = 6.0 * x.powi(3) - 2.0 * y.powi(3)
            + 4.0 * z.powi(3)
            + 2.0 * x.powi(2) * y
            + 4.0 * y.powi(2) * z
            + x.powi(2)
            - y.powi(3)
            + 5.0 * x * y * z
            + 2.0 * x
            + 3.0 * y
            - 5.0 * z
            + 2.0;
        let u2_2 = 3.0 * x.powi(3) - 4.0 * y.powi(3)
            + 2.0 * z.powi(3)
            + 2.0 * x.powi(2) * z
            + 3.0 * y.powi(2)
            - 2.0 * x
            + 3.0 * y
            - 5.0 * z
            + 9.0;
        Vector2::new(u2_1, u2_2)
    };
    let u = |x: &Point3<f64>| u1(x) - u2(x);

    // TODO: Use some arbitrary element rather than reference element
    let element = arbitrary_tet20_element();
    let u_h_element =
        flatten_vertically(&element.vertices().iter().map(u2).collect::<Vec<_>>()).unwrap();

    // Use a quadrature rule with sufficient strength such that it can exactly capture the error
    // (since we compute a squared norm, we need double the polynomial degree)
    let (weights, points) = quadrature::total_order::tetrahedron(10).unwrap();
    let mut basis_buffer = vec![3.0; 20];
    let L2_error_computed = estimate_element_L2_error(
        &element,
        u1,
        DVectorSlice::from(&u_h_element),
        &weights,
        &points,
        &mut basis_buffer,
    );

    let L2_error_expected = {
        let (weights, points) =
            transform_quadrature_to_physical_domain(&element, &weights, &points);
        let u_squared_norm = |x: &Point3<f64>| u(x).norm_squared();
        (weights, points).integrate(u_squared_norm).sqrt()
    };

    assert_scalar_eq!(
        L2_error_computed,
        L2_error_expected,
        comp = abs,
        tol = 1e-12
    );
}