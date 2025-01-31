use fenris::element::{
    map_physical_coordinates, project_physical_coordinates, ClosestPoint, ClosestPointInElement, ElementConnectivity,
    FiniteElement, FixedNodesReferenceFiniteElement, Hex20Element, Hex27Element, Hex8Element, Quad4d2Element,
    Quad9d2Element, Segment2d2Element, Tet10Element, Tet20Element, Tet4Element, Tri3d2Element, Tri6d2Element,
};
use fenris::error::estimate_element_L2_error;
use fenris::geometry::proptest::{clockwise_triangle2d_strategy_f64, nondegenerate_convex_quad2d_strategy_f64};
use fenris::geometry::{LineSegment2d, Quad2d, Triangle, Triangle2d};
use fenris::integrate::IntegrationWorkspace;
use fenris::mesh::procedural::create_unit_square_uniform_tri_mesh_2d;
use fenris::mesh::TriangleMesh2d;
use fenris::nalgebra::DVector;
use fenris::quadrature;
use fenris::util::proptest::point2_f64_strategy;
use fenris_optimize::calculus::{approximate_jacobian, VectorFunctionBuilder};
use matrixcompare::{assert_matrix_eq, assert_scalar_eq, prop_assert_matrix_eq};
use nalgebra::{
    point, DVectorView, DimName, Dyn, MatrixView, OMatrix, OPoint, Point1, Point2, Point3, Vector1, Vector2, Vector3,
    U1, U10, U2, U20, U27, U3, U4, U6, U8, U9,
};
use proptest::prelude::*;
use util::assert_approx_matrix_eq;

#[test]
fn map_reference_coords_quad2d() {
    let vertices = [
        Point2::new(5.0, 3.0),
        Point2::new(10.0, 4.0),
        Point2::new(11.0, 6.0),
        Point2::new(6.0, 4.0),
    ];
    let quad = Quad2d(vertices);
    let quad = Quad4d2Element::from(quad);

    let x0 = quad.map_reference_coords(&Point2::new(-1.0, -1.0));
    assert!(x0.coords.relative_eq(&vertices[0].coords, 1e-10, 1e-10));

    let x1 = quad.map_reference_coords(&Point2::new(1.0, -1.0));
    assert!(x1.coords.relative_eq(&vertices[1].coords, 1e-10, 1e-10));

    let x2 = quad.map_reference_coords(&Point2::new(1.0, 1.0));
    assert!(x2.coords.relative_eq(&vertices[2].coords, 1e-10, 1e-10));

    let x3 = quad.map_reference_coords(&Point2::new(-1.0, 1.0));
    assert!(x3.coords.relative_eq(&vertices[3].coords, 1e-10, 1e-10));
}

#[test]
fn map_reference_coords_edge2d() {
    let a = Point2::new(5.0, 3.0);
    let b = Point2::new(10.0, 4.0);
    let edge = Segment2d2Element::from(LineSegment2d::from_end_points(a, b));

    let x0 = edge.map_reference_coords(&Point1::new(-1.0));
    assert!(x0.coords.relative_eq(&a.coords, 1e-10, 1e-10));

    let x1 = edge.map_reference_coords(&Point1::new(1.0));
    assert!(x1.coords.relative_eq(&b.coords, 1e-10, 1e-10));

    let x2 = edge.map_reference_coords(&Point1::new(0.0));
    assert!(x2
        .coords
        .relative_eq(&((a.coords + b.coords) / 2.0), 1e-10, 1e-10));

    let x3 = edge.map_reference_coords(&Point1::new(0.5));
    assert!(x3
        .coords
        .relative_eq(&(0.25 * a.coords + 0.75 * b.coords), 1e-10, 1e-10));
}

#[test]
fn map_physical_coords_quad2d() {
    let vertices = [
        Point2::new(5.0, 3.0),
        Point2::new(10.0, 4.0),
        Point2::new(11.0, 6.0),
        Point2::new(6.0, 4.0),
    ];
    let quad = Quad2d(vertices);
    let quad = Quad4d2Element::from(quad);

    let xi0 = map_physical_coordinates(&quad, &vertices[0]).unwrap();
    assert!(xi0
        .coords
        .relative_eq(&Vector2::new(-1.0, -1.0), 1e-10, 1e-10));

    let xi1 = map_physical_coordinates(&quad, &vertices[1]).unwrap();
    assert!(xi1
        .coords
        .relative_eq(&Vector2::new(1.0, -1.0), 1e-10, 1e-10));

    let xi2 = map_physical_coordinates(&quad, &vertices[2]).unwrap();
    assert!(xi2
        .coords
        .relative_eq(&Vector2::new(1.0, 1.0), 1e-10, 1e-10));

    let xi3 = map_physical_coordinates(&quad, &vertices[3]).unwrap();
    assert!(xi3
        .coords
        .relative_eq(&Vector2::new(-1.0, 1.0), 1e-10, 1e-10));
}

#[test]
fn tri3d2_lagrange_property() {
    // We expect that N_i(x_j) = delta_ij
    // where N_i is the ith basis function, j is the vertex associated with the ith node,
    // and delta_ij is the Kronecker delta.
    let element = Tri3d2Element::reference();

    for (i, xi) in element.vertices().into_iter().enumerate() {
        let phi = element.evaluate_basis(&xi);

        let mut expected = OMatrix::<f64, U1, U3>::zeros();
        expected[i] = 1.0;

        assert_approx_matrix_eq!(phi, expected, abstol = 1e-12);
    }
}

#[test]
fn tri6d2_lagrange_property() {
    // We expect that N_i(x_j) = delta_ij
    // where N_i is the ith basis function, j is the vertex associated with the ith node,
    // and delta_ij is the Kronecker delta.
    let element = Tri6d2Element::reference();

    for (i, xi) in element.vertices().into_iter().enumerate() {
        let phi = element.evaluate_basis(&xi);

        let mut expected = OMatrix::<f64, U1, U6>::zeros();
        expected[i] = 1.0;

        assert_approx_matrix_eq!(phi, expected, abstol = 1e-12);
    }
}

#[test]
fn quad9_lagrange_property() {
    // We expect that N_i(x_j) = delta_ij
    // where N_i is the ith basis function, j is the vertex associated with the ith node,
    // and delta_ij is the Kronecker delta.
    let element = Quad9d2Element::reference();

    for (i, xi) in element.vertices().into_iter().enumerate() {
        let phi = element.evaluate_basis(&xi);

        let mut expected = OMatrix::<f64, U1, U9>::zeros();
        expected[i] = 1.0;

        assert_approx_matrix_eq!(phi, expected, abstol = 1e-12);
    }
}

#[test]
fn tet4_lagrange_property() {
    // We expect that N_i(x_j) = delta_ij
    // where N_i is the ith basis function, j is the vertex associated with the ith node,
    // and delta_ij is the Kronecker delta.
    let element = Tet4Element::reference();

    for (i, xi) in element.vertices().into_iter().enumerate() {
        let phi = element.evaluate_basis(&xi);

        let mut expected = OMatrix::<f64, U1, U4>::zeros();
        expected[i] = 1.0;

        assert_approx_matrix_eq!(phi, expected, abstol = 1e-12);
    }
}

#[test]
fn tet10_lagrange_property() {
    // We expect that N_i(x_j) = delta_ij
    // where N_i is the ith basis function, j is the vertex associated with the ith node,
    // and delta_ij is the Kronecker delta.
    let element = Tet10Element::reference();

    for (i, xi) in element.vertices().into_iter().enumerate() {
        let phi = element.evaluate_basis(&xi);

        let mut expected = OMatrix::<f64, U1, U10>::zeros();
        expected[i] = 1.0;

        assert_approx_matrix_eq!(phi, expected, abstol = 1e-12);
    }
}

#[test]
fn tet20_lagrange_property() {
    // We expect that N_i(x_j) = delta_ij
    // where N_i is the ith basis function, j is the vertex associated with the ith node,
    // and delta_ij is the Kronecker delta.
    let element = Tet20Element::reference();

    for (i, xi) in element.vertices().into_iter().enumerate() {
        let phi = element.evaluate_basis(&xi);

        let mut expected = OMatrix::<f64, U1, U20>::zeros();
        expected[i] = 1.0;

        assert_approx_matrix_eq!(phi, expected, abstol = 1e-12);
    }
}

#[test]
fn hex8_lagrange_property() {
    // We expect that N_i(x_j) = delta_ij
    // where N_i is the ith basis function, j is the vertex associated with the ith node,
    // and delta_ij is the Kronecker delta.
    let element = Hex8Element::reference();

    for (i, xi) in element.vertices().into_iter().enumerate() {
        let phi = element.evaluate_basis(&xi);

        let mut expected = OMatrix::<f64, U1, U8>::zeros();
        expected[i] = 1.0;

        assert_approx_matrix_eq!(phi, expected, abstol = 1e-12);
    }
}

#[test]
fn hex27_lagrange_property() {
    // We expect that N_i(x_j) = delta_ij
    // where N_i is the ith basis function, j is the vertex associated with the ith node,
    // and delta_ij is the Kronecker delta.
    let element = Hex27Element::reference();

    for (i, xi) in element.vertices().into_iter().enumerate() {
        let phi = element.evaluate_basis(&xi);

        let mut expected = OMatrix::<f64, U1, U27>::zeros();
        expected[i] = 1.0;

        assert_approx_matrix_eq!(phi, expected, abstol = 1e-12);
    }
}

#[test]
fn hex20_lagrange_property() {
    // We expect that N_i(x_j) = delta_ij
    // where N_i is the ith basis function, j is the vertex associated with the ith node,
    // and delta_ij is the Kronecker delta.
    let element = Hex20Element::reference();

    for (i, xi) in element.vertices().into_iter().enumerate() {
        let phi = element.evaluate_basis(&xi);

        let mut expected = OMatrix::<f64, U1, U20>::zeros();
        expected[i] = 1.0;

        assert_approx_matrix_eq!(phi, expected, abstol = 1e-12);
    }
}

#[test]
fn quad4_bilinear_function_exact_error() {
    let quad = Quad2d([
        Point2::new(-1.0, -1.0),
        Point2::new(2.0, -2.0),
        Point2::new(4.0, 1.0),
        Point2::new(-2.0, 3.0),
    ]);
    let element = Quad4d2Element::from(quad);
    // If u_exact is a bilinear function, then we can exactly represent it
    // with a Quad4 element. Then the basis weights of u_h are given by
    // u_exact(x_i), where x_i are the coordinates of each node of the element.
    let u_exact = |p: &Point2<f64>| {
        let x = p[0];
        let y = p[1];
        5.0 * x * y + 3.0 * x - 2.0 * y - 5.0
    };
    let u_weights = DVector::from_vec(quad.0.iter().map(u_exact).collect::<Vec<_>>());

    // TODO: Use lower strength quadrature
    let (weights, points) = quadrature::total_order::quadrilateral(11).unwrap();
    let error = estimate_element_L2_error(
        &element,
        &|x: &Point2<_>| Vector1::new(u_exact(x)),
        MatrixView::from(&u_weights),
        &weights,
        &points,
        &mut IntegrationWorkspace::default(),
    );

    // Note: The solution here is obtained by symbolic integration. See
    // the accompanying notebooks
    assert_scalar_eq!(
        error,
        (9955.0f64 / 12.0).sqrt(),
        comp = abs,
        tol = element.diameter() * 1e-12
    );
}

#[test]
fn hex27_triquadratic_function_exact_error() {
    let element = Hex27Element::reference();
    // If u_exact is a triquadratic function, then we can exactly represent it
    // with a Hex27 element. Then the basis weights of u_h are given by
    // u_exact(x_i), where x_i are the coordinates of each node of the element.
    let u_exact = |p: &Point3<f64>| {
        let x = p[0];
        let y = p[1];
        let z = p[2];
        // Three arbitrary quadratic functions in each argument x, y, z
        let f = 3.0 * x * x - 2.0 * x + 5.0;
        let g = -2.0 * y * y + 3.0 * y + 1.5;
        let h = 1.5 * z * z + 1.2 * z - 3.0;

        f * g * h
    };
    let u_weights = DVector::from_vec(element.vertices().iter().map(u_exact).collect());

    // TODO: Use lower strength quadrature
    let (weights, points) = quadrature::total_order::hexahedron(11).unwrap();
    let error = estimate_element_L2_error(
        &element,
        &|x: &Point3<_>| Vector1::new(u_exact(x)),
        MatrixView::from(&u_weights),
        &weights,
        &points,
        &mut IntegrationWorkspace::default(),
    );

    // Note: The solution here is obtained by symbolic integration. See
    // the accompanying notebooks
    assert_scalar_eq!(error, 0.0, comp = abs, tol = 1e-12);
}

#[test]
fn hex20_quadratic_function_exact_error() {
    let element = Hex20Element::reference();
    // If u_exact is a quadratic function, then we can exactly represent it
    // with a Hex20 element. Then the basis weights of u_h are given by
    // u_exact(x_i), where x_i are the coordinates of each node of the element.
    // Note that the space of functions spanned by the Hex20 basis functions is larger
    // than the space of quadratic functions, so this is not a sufficient condition
    // for checking correctness
    let u_exact = |p: &Point3<f64>| {
        let x = p[0];
        let y = p[1];
        let z = p[2];

        2.0 * x * x + 4.0 * y * y - 3.0 * z * z + 3.0 * x * y - 5.0 * x * z + 1.5 * y * z + 3.0 * x - 2.0 * y
            + 3.0 * z
            + 9.0
    };
    let u_weights = DVector::from_vec(element.vertices().iter().map(|x| u_exact(x)).collect());

    // TODO: Use lower strength quadrature
    let (weights, points) = quadrature::total_order::hexahedron(11).unwrap();
    let error = estimate_element_L2_error(
        &element,
        &|x: &Point3<_>| Vector1::new(u_exact(x)),
        DVectorView::from(&u_weights),
        &weights,
        &points,
        &mut IntegrationWorkspace::default(),
    );

    // Note: The solution here is obtained by symbolic integration. See
    // the accompanying notebooks
    assert_scalar_eq!(error, 0.0, comp = abs, tol = 1e-12);
}

// TODO: Test all gradients of basis functions

fn point_in_tri_ref_domain() -> impl Strategy<Value = Point2<f64>> {
    // Generate points x, y in [-1, 1]^2 such that
    // x + y <= 0
    (-1.0..=1.0)
        .prop_flat_map(|x: f64| (Just(x), -1.0..=-x))
        .prop_map(|(x, y)| Point2::new(x, y))
}

fn point_in_quad_ref_domain() -> impl Strategy<Value = Point2<f64>> {
    // Generate points x, y, z in [-1, 1]^3
    let r = -1.0..=1.0;
    [r.clone(), r].prop_map(|[x, y]| Point2::new(x, y))
}

fn point_in_hex_ref_domain() -> impl Strategy<Value = Point3<f64>> {
    // Generate points x, y, z in [-1, 1]^3
    let r = -1.0..=1.0;
    [r.clone(), r.clone(), r].prop_map(|[x, y, z]| Point3::new(x, y, z))
}

fn point_in_tet_ref_domain() -> impl Strategy<Value = Point3<f64>> {
    // Generate points x, y, z in [-1, 1]^3 such that
    // x + y + z <= 0
    (-1.0..=1.0)
        .prop_flat_map(|x: f64| (Just(x), -1.0..=-x))
        .prop_flat_map(|(x, y)| {
            let z_range = -1.0..=(-(x + y));
            (Just(x), Just(y), z_range)
        })
        .prop_map(|(x, y, z)| Point3::new(x, y, z))
}

macro_rules! partition_of_unity_test {
    ($test_name:ident, $ref_domain_strategy:expr, $ref_element:expr) => {
        proptest! {
            #[test]
            fn $test_name(xi in $ref_domain_strategy) {
                let xi = xi;
                let element = $ref_element;
                let phi = element.evaluate_basis(&xi);
                let phi_sum: f64 = phi.sum();

                prop_assert!( (phi_sum - 1.0f64).abs() <= 1e-12);
            }
        }
    };
}

macro_rules! partition_of_unity_gradient_test {
    ($test_name:ident, $ref_domain_strategy:expr, $ref_element:expr) => {
        proptest! {
            #[test]
            fn $test_name(xi in $ref_domain_strategy) {
                // Since the sum of basis functions is 1, the sum of the gradients must be 0
                let xi = xi;
                let element = $ref_element;
                let grad = element.gradients(&xi);
                let grad_sum = grad.column_sum();

                let mut zero = grad_sum.clone();
                zero.fill(0.0);

                assert_approx_matrix_eq!(grad_sum, zero, abstol=1e-12);
            }
        }
    };
}

partition_of_unity_test!(
    tri3d2_partition_of_unity,
    point_in_tri_ref_domain(),
    Tri3d2Element::reference()
);
partition_of_unity_test!(
    tri6d2_partition_of_unity,
    point_in_tri_ref_domain(),
    Tri6d2Element::reference()
);
partition_of_unity_test!(
    quad4_partition_of_unity,
    point_in_quad_ref_domain(),
    Tri6d2Element::reference()
);
partition_of_unity_test!(
    quad9_partition_of_unity,
    point_in_quad_ref_domain(),
    Tri6d2Element::reference()
);

partition_of_unity_test!(
    hex27_partition_of_unity,
    point_in_hex_ref_domain(),
    Hex27Element::reference()
);

partition_of_unity_test!(
    hex20_partition_of_unity,
    point_in_hex_ref_domain(),
    Hex20Element::reference()
);

partition_of_unity_gradient_test!(
    tri3d2_partition_of_unity_gradient,
    point_in_tri_ref_domain(),
    Tri3d2Element::reference()
);
partition_of_unity_gradient_test!(
    tri6d2_partition_of_unity_gradient,
    point_in_tri_ref_domain(),
    Tri6d2Element::reference()
);
partition_of_unity_gradient_test!(
    quad4_partition_of_unity_gradient,
    point_in_quad_ref_domain(),
    Quad4d2Element::reference()
);
partition_of_unity_gradient_test!(
    quad9_partition_of_unity_gradient,
    point_in_quad_ref_domain(),
    Quad9d2Element::reference()
);

partition_of_unity_gradient_test!(
    hex27_partition_of_unity_gradient,
    point_in_hex_ref_domain(),
    Hex27Element::reference()
);

partition_of_unity_gradient_test!(
    hex20_partition_of_unity_gradient,
    point_in_hex_ref_domain(),
    Hex20Element::reference()
);

proptest! {
    #[test]
    fn tri3_affine_function_error_is_zero(tri in clockwise_triangle2d_strategy_f64()) {
        let element = Tri3d2Element::from(tri);
        // If u_exact is an affine function, then we can exactly represent it
        // with a Tri3 element. Then the basis weights of u_h are given by
        // u_exact(x_i), where x_i are the coordinates of each node of the element.
        let u_exact = |x: &Point2<f64>| 2.0 * x[0] - 3.0 * x[1] + 1.5;
        let u_weights = DVector::from_vec(tri.0.iter().map(|x| u_exact(x)).collect::<Vec<_>>());

        // TODO: Use lower strength quadrature
        let (weights, points) = quadrature::total_order::triangle(5).unwrap();
        let error = estimate_element_L2_error(
            &element,
            &|x: &Point2<_>| Vector1::new(u_exact(x)),
            DVectorView::from(&u_weights),
            &weights,
            &points,
            &mut IntegrationWorkspace::default());

        assert_scalar_eq!(error, 0.0, comp=abs, tol=element.diameter() * 1e-12);
    }

    #[test]
    fn tri6_affine_function_error_is_zero(tri in clockwise_triangle2d_strategy_f64()) {
        let element = Tri6d2Element::from(Tri3d2Element::from(tri));
        // If u_exact is an affine function, then we can exactly represent it
        // with a Tri3 element. Then the basis weights of u_h are given by
        // u_exact(x_i), where x_i are the coordinates of each node of the element.
        let u_exact = |x: &Point2<f64>| 2.0 * x[0] - 3.0 * x[1] + 1.5;
        let u_weights = DVector::from_vec(element.vertices().iter().map(|x| u_exact(x)).collect::<Vec<_>>());

        // TODO: Use lower strength quadrature
        let (weights, points) = quadrature::total_order::triangle(5).unwrap();
        let error = estimate_element_L2_error(
            &element,
            &|x: &Point2<_>| Vector1::new(u_exact(x)),
            DVectorView::from(&u_weights),
            &weights,
            &points,
            &mut IntegrationWorkspace::default());

        assert_scalar_eq!(error, 0.0, comp=abs, tol=element.diameter() * 1e-12);
    }

    #[test]
    fn tri6_quadratic_function_error_is_zero(tri in clockwise_triangle2d_strategy_f64()) {
        let element = Tri6d2Element::from(Tri3d2Element::from(tri));
        // If u_exact is an affine function, then we can exactly represent it
        // with a Tri3 element. Then the basis weights of u_h are given by
        // u_exact(x_i), where x_i are the coordinates of each node of the element.
        let u_exact = |x: &Point2<f64>| 2.0 * x[0] * x[0] - 3.0 * x[1] * x[0] + 0.5 * x[1] + 1.5;
        let u_weights = DVector::from_vec(element.vertices().iter().map(|x| u_exact(x)).collect::<Vec<_>>());

        // TODO: Use lower strength quadrature
        let (weights, points) = quadrature::total_order::triangle(5).unwrap();
        let error = estimate_element_L2_error(
            &element,
            &|x: &Point2<_>| Vector1::new(u_exact(x)),
            DVectorView::from(&u_weights),
            &weights,
            &points,
            &mut IntegrationWorkspace::default());

        // TODO: Check tolerance
        assert_scalar_eq!(error, 0.0, comp=abs, tol=element.diameter() * element.diameter() * 1e-12);
    }

    #[test]
    fn quad4_affine_function_error_is_zero(quad in nondegenerate_convex_quad2d_strategy_f64()) {
        let element = Quad4d2Element::from(quad);
        // If u_exact is an affine function, then we can exactly represent it
        // with a Quad4 element. Then the basis weights of u_h are given by
        // u_exact(x_i), where x_i are the coordinates of each node of the element.
        // Note that this is not true for a general bilinear function unless
        // the quad and the reference quad are related by an affine transformation
        // (i.e. the quad is a parallellogram).
        let u_exact = |x: &Point2<f64>| {
            let y = x[1];
            let x = x[0];
            3.0 * x + 2.0 * y - 3.0
        };
        let u_weights = DVector::from_vec(quad.0
                                                    .iter()
                                                    .map(|x| u_exact(x))
                                                    .collect::<Vec<_>>());

        let (weights, points) = quadrature::total_order::quadrilateral(11).unwrap();
        let error = estimate_element_L2_error(
            &element,
            &|x: &Point2<_>| Vector1::new(u_exact(x)),
            MatrixView::from(&u_weights),
            &weights,
            &points,
            &mut IntegrationWorkspace::default());

        assert_scalar_eq!(error, 0.0, comp=abs, tol=element.diameter() * 1e-12);
    }

    #[test]
    fn edge2d_element_jacobian_is_derivative_of_transform(
        (a, b, xi) in (point2_f64_strategy(), point2_f64_strategy(), -1.0..=1.0)
    ) {
        let segment = LineSegment2d::from_end_points(a, b);
        let element = Segment2d2Element::from(segment);
        let xi = Point1::new(xi);

        // Finite difference parameter
        let h = 1e-6;
        let hvec = Vector1::new(h);

        // TODO: Extend VectorFunction and approximate_jacobian to allow
        // maps between domains of different dimension
        let j = element.reference_jacobian(&xi);

        // Approximate Jacobian by finite differences
        let x_plus = element.map_reference_coords(&(xi + hvec));
        let x_minus = element.map_reference_coords(&(xi - hvec));
        let j_approx = (x_plus - x_minus) / (2.0 * h);

        let tol = (a.coords +  b.coords).norm() * h;
        assert_approx_matrix_eq!(j, j_approx, abstol=tol);

    }

    #[test]
    fn edge2d_element_project_physical_coords_with_perturbation(
        (a, b, xi, eps) in (point2_f64_strategy(), point2_f64_strategy(),
                            -1.0..=1.0, prop_oneof!(Just(0.0), -1.0 .. 1.0))
    ) {
        let segment = LineSegment2d::from_end_points(a, b);

        prop_assume!(segment.length() > 0.0);

        let element = Segment2d2Element::from(&segment);
        let xi = Point1::new(xi);
        let x = element.map_reference_coords(&xi);
        // Perturb the surface point in the direction normal to the surface. This checks
        // that the projection actually still manages to correctly reproduce the original point.
        let x_perturbed = &x + eps * segment.normal_dir();
        let xi_proj = project_physical_coordinates(&element, &Point2::from(x_perturbed)).unwrap();
        let x_reconstructed = element.map_reference_coords(&xi_proj);

        prop_assert!((x_reconstructed - x).norm() <= 1e-12);
    }

    #[test]
    fn tet4_partition_of_unity(xi in point_in_tet_ref_domain()) {
        let element = Tet4Element::reference();
        let phi = element.evaluate_basis(&xi);
        let phi_sum: f64 = phi.sum();
        prop_assert!( (phi_sum - 1.0f64).abs() <= 1e-12);
    }

    #[test]
    fn tet10_partition_of_unity(xi in point_in_tet_ref_domain()) {
        let element = Tet10Element::reference();
        let phi = element.evaluate_basis(&xi);
        let phi_sum: f64 = phi.sum();
        prop_assert!( (phi_sum - 1.0f64).abs() <= 1e-12);
    }

    #[test]
    fn tet20_partition_of_unity(xi in point_in_tet_ref_domain()) {
        let element = Tet20Element::reference();
        let phi = element.evaluate_basis(&xi);
        let phi_sum: f64 = phi.sum();

        prop_assert!( (phi_sum - 1.0f64).abs() <= 1e-12);
    }

    #[test]
    fn hex8_partition_of_unity(xi in point_in_hex_ref_domain()) {
        let element = Hex8Element::reference();
        let phi = element.evaluate_basis(&xi);
        let phi_sum: f64 = phi.sum();
        prop_assert!( (phi_sum - 1.0f64).abs() <= 1e-12);
    }

    #[test]
    fn tet4_partition_of_unity_gradient((x, y, z) in (-1.0 ..= 1.0, -1.0 ..= 1.0, -1.0 ..= 1.0)) {
        // Since the sum of basis functions is 1, the sum of the gradients must be 0
        let xi = Point3::new(x, y, z);
        let element = Tet4Element::reference();
        let grad = element.gradients(&xi);
        let grad_sum = grad.column_sum();

        assert_approx_matrix_eq!(grad_sum, Vector3::<f64>::zeros(), abstol=1e-12);
    }

    #[test]
    fn tet10_partition_of_unity_gradient((x, y, z) in (-1.0 ..= 1.0, -1.0 ..= 1.0, -1.0 ..= 1.0)) {
        // Since the sum of basis functions is 1, the sum of the gradients must be 0
        let xi = Point3::new(x, y, z);
        let element = Tet10Element::reference();
        let grad = element.gradients(&xi);
        let grad_sum = grad.column_sum();

        assert_approx_matrix_eq!(grad_sum, Vector3::<f64>::zeros(), abstol=1e-12);
    }

    #[test]
    fn tet20_partition_of_unity_gradient((x, y, z) in (-1.0 ..= 1.0, -1.0 ..= 1.0, -1.0 ..= 1.0)) {
        // Since the sum of basis functions is 1, the sum of the gradients must be 0
        let xi = Point3::new(x, y, z);
        let element = Tet20Element::reference();
        let grad = element.gradients(&xi);
        let grad_sum = grad.column_sum();

        assert_approx_matrix_eq!(grad_sum, Vector3::<f64>::zeros(), abstol=1e-12);
    }

    #[test]
    fn tet4_affine_function_error_is_zero(element in any::<Tet4Element<f64>>()) {
        // If u_exact is an affine function, then we can exactly represent it
        // with a Tet4 element. Then the basis weights of u_h are given by
        // u_exact(x_i), where x_i are the coordinates of each node of the element.
        let u_exact = |p: &Point3<f64>| {
            let x = p.x;
            let y = p.y;
            let z = p.z;
            3.0 * x + 2.0 * y - 3.0 * z + 3.0
        };
        let u_weights = DVector::from_vec(element.vertices()
                                                    .iter()
                                                    .map(|x| u_exact(x))
                                                    .collect::<Vec<_>>());

        let (weights, points) = quadrature::total_order::tetrahedron(10).unwrap();
        let error = estimate_element_L2_error(
            &element, &|x: &Point3<_>| Vector1::new(u_exact(x)),
            MatrixView::from(&u_weights),
            &weights,
            &points,
            &mut IntegrationWorkspace::default());

        assert_scalar_eq!(error, 0.0, comp=abs, tol=element.diameter() * 1e-12);
    }

    #[test]
    fn tri3d2_element_gradient_is_derivative_of_transform(
        (tri, xi) in (any::<Triangle2d<f64>>(), point_in_tri_ref_domain())
    ) {

        let elem = Tri3d2Element::from(tri);

        // Finite difference parameter
        let h = 1e-6;
        // Note: Function values are given as row vectors, so we transpose to get the result,
        // and we must also transpose the end result
        let f = VectorFunctionBuilder::with_dimension(3).with_function(move |x, xi| {
            let xi = OPoint::from(xi.generic_view((0, 0), (U2::name(), U1::name())).clone_owned());
            x.copy_from(&elem.evaluate_basis(&xi).transpose());
        });

        let grad = elem.gradients(&xi);
        let grad_approx = approximate_jacobian(f, &DVectorView::<_, Dyn>::from(&xi.coords).clone_owned(), &h).transpose();

        assert_approx_matrix_eq!(grad, &grad_approx, abstol=1e-5);
    }

    #[test]
    fn tri6d2_element_gradient_is_derivative_of_transform(
        (tri, xi) in (any::<Triangle2d<f64>>(), point_in_tri_ref_domain())
    ) {

        let elem = Tri6d2Element::from(Tri3d2Element::from(tri));

        // Finite difference parameter
        let h = 1e-6;
        // Note: Function values are given as row vectors, so we transpose to get the result,
        // and we must also transpose the end result
        let f = VectorFunctionBuilder::with_dimension(6).with_function(move |x, xi| {
            let xi = OPoint::from(xi.generic_view((0, 0), (U2::name(), U1::name())).clone_owned());
            x.copy_from(&elem.evaluate_basis(&xi).transpose());
        });

        let grad = elem.gradients(&xi);
        let grad_approx = approximate_jacobian(f, &DVectorView::<_, Dyn>::from(&xi.coords).clone_owned(), &h).transpose();

        assert_approx_matrix_eq!(grad, &grad_approx, abstol=1e-5);
    }

    #[test]
    fn tet4_element_gradient_is_derivative_of_transform(
        (tet, xi) in (any::<Tet4Element<f64>>(), point_in_tet_ref_domain())
    ) {
        // Finite difference parameter
        let h = 1e-6;
        // Note: Function values are given as row vectors, so we transpose to get the result,
        // and we must also transpose the end result
        let f = VectorFunctionBuilder::with_dimension(4).with_function(move |x, xi| {
            let xi = OPoint::from(xi.generic_view((0, 0), (U3::name(), U1::name())).clone_owned());
            x.copy_from(&tet.evaluate_basis(&xi).transpose());
        });

        let grad = tet.gradients(&xi);
        let grad_approx = approximate_jacobian(f, &DVectorView::<_, Dyn>::from(&xi.coords).clone_owned(), &h).transpose();

        assert_approx_matrix_eq!(grad, &grad_approx, abstol=1e-5);
    }

    #[test]
    fn tet10_reference_element_gradient_is_derivative_of_transform(
        xi in point_in_tet_ref_domain()
    ) {
        let tet = Tet10Element::reference();
        // Finite difference parameter
        let h = 1e-6;
        // Note: Function values are given as row vectors, so we transpose to get the result,
        // and we must also transpose the end result
        let f = VectorFunctionBuilder::with_dimension(10).with_function(move |x, xi| {
            let xi = OPoint::from(xi.generic_view((0, 0), (U3::name(), U1::name())).clone_owned());
            x.copy_from(&tet.evaluate_basis(&xi).transpose());
        });

        let grad = tet.gradients(&xi);
        let grad_approx = approximate_jacobian(f, &DVectorView::<_, Dyn>::from(&xi.coords).clone_owned(), &h).transpose();

        assert_approx_matrix_eq!(grad, &grad_approx, abstol=1e-5);
    }

    #[test]
    fn tet20_reference_element_gradient_is_derivative_of_transform(
        xi in point_in_tet_ref_domain()
    ) {
        let tet = Tet20Element::reference();
        // Finite difference parameter
        let h = 1e-6;
        // Note: Function values are given as row vectors, so we transpose to get the result,
        // and we must also transpose the end result
        let f = VectorFunctionBuilder::with_dimension(20).with_function(move |x, xi| {
            let xi = OPoint::from(xi.generic_view((0, 0), (U3::name(), U1::name())).clone_owned());
            x.copy_from(&tet.evaluate_basis(&xi).transpose());
        });

        let grad = tet.gradients(&xi);
        let grad_approx = approximate_jacobian(f, &DVectorView::<_, Dyn>::from(&xi.coords).clone_owned(), &h).transpose();

        assert_approx_matrix_eq!(grad, &grad_approx, abstol=1e-5);
    }

    #[test]
    fn hex27_reference_element_gradient_is_derivative_of_transform(
        xi in point_in_hex_ref_domain()
    ) {
        let hex = Hex27Element::reference();
        // Finite difference parameter
        let h = 1e-6;
        // Note: Function values are given as row vectors, so we transpose to get the result,
        // and we must also transpose the end result
        let f = VectorFunctionBuilder::with_dimension(27).with_function(move |x, xi| {
            let xi = OPoint::from(xi.generic_view((0, 0), (U3::name(), U1::name())).clone_owned());
            x.copy_from(&hex.evaluate_basis(&xi).transpose());
        });

        let grad = hex.gradients(&xi);
        let xi = DVectorView::<_, Dyn>::from(&xi.coords).clone_owned();
        let grad_approx = approximate_jacobian(f, &xi, &h).transpose();

        assert_approx_matrix_eq!(grad, &grad_approx, abstol=1e-5);
    }

    #[test]
    fn hex20_reference_element_gradient_is_derivative_of_transform(
        xi in point_in_hex_ref_domain()
    ) {
        let hex = Hex20Element::reference();
        // Finite difference parameter
        let h = 1e-6;
        // Note: Function values are given as row vectors, so we transpose to get the result,
        // and we must also transpose the end result
        let f = VectorFunctionBuilder::with_dimension(20).with_function(move |x, xi| {
            let xi = OPoint::from(xi.generic_view((0, 0), (U3::name(), U1::name())).clone_owned());
            x.copy_from(&hex.evaluate_basis(&xi).transpose());
        });

        let grad = hex.gradients(&xi);
        let xi = DVectorView::<_, Dyn>::from(&xi.coords).clone_owned();
        let grad_approx = approximate_jacobian(f, &xi, &h).transpose();

        assert_approx_matrix_eq!(grad, &grad_approx, abstol=1e-5);
    }

    #[test]
    fn tet4_element_jacobian_is_derivative_of_transform(
        (tet, xi) in (any::<Tet4Element<f64>>(), point_in_tet_ref_domain())
    ) {
        // Finite difference parameter
        let h = 1e-6;
        // Function is x = f(xi)
        let f = VectorFunctionBuilder::with_dimension(3).with_function(move |x, xi| {
            let xi = OPoint::from(xi.generic_view((0, 0), (U3::name(), U1::name())).clone_owned());
            x.copy_from(&tet.map_reference_coords(&xi).coords);
        });

        let j = tet.reference_jacobian(&xi);
        let j_approx = approximate_jacobian(f, &DVectorView::<_, Dyn>::from(&xi.coords).clone_owned(), &h);
        assert_approx_matrix_eq!(j, &j_approx, abstol=1e-5);
    }

    #[test]
    fn tet4_element_jacobian_has_non_negative_jacobian(
        (tet, xi) in (any::<Tet4Element<f64>>(), point_in_tet_ref_domain())
    ) {
        let j = tet.reference_jacobian(&xi);
        prop_assert!(j.determinant() >= 0.0);
    }

    #[test]
    fn tri3_closest_point_in_interior_is_identity(element: Tri3d2Element<f64>, xi in point_in_tri_ref_domain()) {
        // We cannot compare the reference coordinates directly, because the element
        // may be degenerate. So instead we check that the closest point in reference coords
        // maps back to the same point in physical space
        let x = element.map_reference_coords(&xi);
        let xi2 = element.closest_point(&x).point().clone();
        let x2 = element.map_reference_coords(&xi2);
        prop_assert_matrix_eq!(x2.coords, x.coords, comp = abs, tol = element.diameter() * 1e-9);
    }

    #[test]
    fn tri3_exterior_closest_point(
        element: Tri3d2Element<f64>,
        t in 0.0 ..= 1.0,
        edge in 0 .. 3usize,
        extrusion in 0.0 ..= 5.0)
    {
        let reference = Tri3d2Element::reference();
        let ref_edge = Triangle(*reference.vertices()).edge(edge);
        let element_edge = Triangle(*element.vertices()).edge(edge);
        let xi0 = ref_edge.point_from_parameter(t);
        let x0 = element.map_reference_coords(&xi0);

        // Produce a point by extruding along the normal direction of the edge.
        // The closest point to this point will by definition be the starting point.
        let x = x0 + extrusion * element_edge.normal_dir();
        let result = element.closest_point(&x);
        prop_assert!(matches!(result, ClosestPoint::ClosestPoint(_)));
        let mapped_result = element.map_reference_coords(result.point());
        prop_assert_matrix_eq!(mapped_result.coords, x0.coords,
                               comp = abs, tol = element.diameter() * 1e-9);
    }
}

#[test]
fn tri3d2_closest_point_is_a_vertex() {
    // We test the case where the closest point is a vertex because the proptests don't cover it.
    let ref_vertices = Tri3d2Element::reference().vertices().clone();
    let vertices = [[1.0, 0.0], [2.0, 1.0], [-1.0, 2.0]].map(Point2::from);
    let element = Tri3d2Element::from_vertices(vertices);

    macro_rules! assert_exterior_closest_point {
        ($point:expr, ref_coords = $ref_coords:expr) => {{
            let result = element.closest_point(&$point);
            assert!(matches!(result, ClosestPoint::ClosestPoint(_)));
            assert_matrix_eq!(
                result.point().coords,
                $ref_coords.coords,
                comp = abs,
                tol = 1e-9 * element.diameter()
            );
        }};
    }

    assert_exterior_closest_point!(point![5.0, 2.0], ref_coords = ref_vertices[1]);
    assert_exterior_closest_point!(point![2.0, -1.0], ref_coords = ref_vertices[0]);
    assert_exterior_closest_point!(point![-3.0, 2.0], ref_coords = ref_vertices[2]);
}

#[test]
fn tri3d2_closest_point_interior_point() {
    let vertices = [[1.0, 0.0], [2.0, 1.0], [-1.0, 2.0]].map(Point2::from);
    let element = Tri3d2Element::from_vertices(vertices);

    let xi = point![-0.5, -0.5];
    let x = element.map_reference_coords(&xi);
    let result = element.closest_point(&x);
    assert!(matches!(result, ClosestPoint::InElement(_)));
    assert_matrix_eq!(
        result.point().coords,
        xi.coords,
        comp = abs,
        tol = 1e-9 * element.diameter()
    );
}

#[test]
fn tri3d2_closest_point_degenerate_elements() {
    // TODO: These tests are not at all comprehensive. I've tried to generate degenerate
    // inputs, but it's difficult to find inputs that actually cause serious problems.
    // I think ultimately such problematic inputs have to come from problematic real problems...

    // Degenerate to a single point
    {
        let v = point![3.0, 3.0];
        let element = Tri3d2Element::from_vertices([v, v, v]);

        let x = point![2.0, 2.0];
        let xi = element.closest_point(&x).point().clone();
        let x_element = element.map_reference_coords(&xi);
        assert_matrix_eq!(x_element.coords, v.coords, comp = abs, tol = 1e-12);
    }

    // Degenerate to a line
    {
        let vertices = [[1.0, 1.0], [2.0, 1.0], [0.5, 1.0]].map(Point2::from);
        let element = Tri3d2Element::from_vertices(vertices);

        let x = point![1.3, 1.5];
        let xi = element.closest_point(&x).point().clone();
        let x_element = element.map_reference_coords(&xi);

        assert_matrix_eq!(x_element.coords, point![1.3, 1.0].coords, comp = abs, tol = 1e-12);
    }

    // *Almost* degenerate to a line
    {
        let eps = 1e-15;
        let vertices = [[1.0, 1.0], [3.0, 2.0], [2.0, 1.5 + eps]].map(Point2::from);
        let element = Tri3d2Element::from_vertices(vertices);

        let x = point![2.0, 1.5 + eps / 2.0];
        let xi = element.closest_point(&x).point().clone();
        let x_element = element.map_reference_coords(&xi);

        assert_matrix_eq!(x_element.coords, x.coords, comp = abs, tol = 1e-12);
    }
}

#[test]
fn tri3d2_closest_point_boundary_points() {
    // Map points from the boundary of the reference element
    // to physical space, then check that the closest point returned by the element
    // has the same reference coordinates
    let mesh: TriangleMesh2d<f64> = create_unit_square_uniform_tri_mesh_2d(10);
    let boundary_points = [
        [-1.0, -1.0],
        [1.0, -1.0],
        [-1.0, 1.0],
        [-1.0, 0.5],
        [0.5, -1.0],
        [0.0, 0.0],
    ]
    .map(Point2::from);

    for conn in mesh.connectivity() {
        let element = conn.element(mesh.vertices()).unwrap();
        for xi in &boundary_points {
            let x = element.map_reference_coords(xi);
            let xi_closest = element.closest_point(&x).point().clone();
            assert_matrix_eq!(xi_closest.coords, xi.coords, comp = abs, tol = 1e-12);
        }
    }
}
