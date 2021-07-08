use crate::allocators::BiDimAllocator;
use crate::nalgebra::allocator::Allocator;
use crate::nalgebra::{
    DMatrixSliceMut, DVectorSlice, DefaultAllocator, DimName, MatrixMN, RealField, Scalar, VectorN,
};
use crate::SmallDim;

mod laplace;

pub use laplace::*;

pub trait Operator {
    type SolutionDim: SmallDim;

    /// The parameters associated with the operator.
    ///
    /// Typically this encodes material information, such as density, stiffness and other physical
    /// quantities. This is intended to be paired with data associated with individual
    /// quadrature points during numerical integration.
    type Parameters: Default + Clone + 'static;
}

pub trait EllipticOperator<T, GeometryDim>: Operator
where
    T: Scalar,
    GeometryDim: SmallDim,
    DefaultAllocator: Allocator<T, GeometryDim, Self::SolutionDim>,
{
    /// TODO: Find better name
    fn compute_elliptic_term(
        &self,
        gradient: &MatrixMN<T, GeometryDim, Self::SolutionDim>,
        data: &Self::Parameters,
    ) -> MatrixMN<T, GeometryDim, Self::SolutionDim>;
}

/// A contraction operator encoding derivative information for an elliptic operator.
///
/// The contraction operator for an elliptic operator $g = g(\nabla u)$ evaluated at $\nabla u$
/// is defined as the $s \times s$ matrix associated with vectors $a, b \in \mathbb{R}^d$ by
///
/// $$ \\mathcal{C}\_{g} (\nabla u, a, b)
///     := a_k \pd{g_{ki}}{G_{mj}} (\nabla u) \\, b_m \enspace e_i \otimes e_j, $$
///
/// where $G = \nabla u$. We have used Einstein summation notation to simplify the notation
/// for the above expression.
///
/// TODO: Maybe return results in impls...?
/// TODO: Decide how to model symmetry
pub trait EllipticContraction<T, GeometryDim>: Operator
where
    T: RealField,
    GeometryDim: SmallDim,
    DefaultAllocator: BiDimAllocator<T, GeometryDim, Self::SolutionDim>,
{
    /// Compute $ C_g(\nabla u, a, b)$ with the given parameters.
    fn contract(
        &self,
        gradient: &MatrixMN<T, GeometryDim, Self::SolutionDim>,
        a: &VectorN<T, GeometryDim>,
        b: &VectorN<T, GeometryDim>,
        parameters: &Self::Parameters,
    ) -> MatrixMN<T, Self::SolutionDim, Self::SolutionDim>;

    /// Compute the contraction for a number of vectors at the same time, with the given
    /// parameters.
    ///
    /// The vectors $a \in \mathbb{R}^{dM}$ and $b \in \mathbb{R}^{dN}$ are stacked vectors
    /// $$
    /// \begin{align*}
    /// a := \begin{pmatrix}
    /// a_1 \newline
    /// \vdots \newline
    /// a_M
    /// \end{pmatrix},
    /// \qquad
    /// b:= \begin{pmatrix}
    /// b_1 \newline
    /// \vdots \newline
    /// b_N
    /// \end{pmatrix}
    /// \end{align*}
    /// $$
    /// and $a_I \in \mathbb{R}^d$, $b_J \in \mathbb{R}^d$ for $I = 1, \dots, M$, $J = 1, \dots, N$.
    /// Let $C \in \mathbb{R}^{sM \times sN}$ denote the output matrix,
    /// which is a block matrix of the form
    /// $$
    /// \begin{align*}
    /// C := \begin{pmatrix}
    /// C_{11} & \dots  & C_{1N} \newline
    /// \vdots & \ddots & \vdots \newline
    /// C_{M1} & \dots  & C_{MN}
    /// \end{pmatrix}
    /// \end{align*},
    /// $$
    /// where each block $C_{IJ}$ is an $s \times s$ matrix. This method **accumulates** the
    /// block-wise **scaled** contractions in the following manner:
    ///
    /// $$
    /// C_{IJ} \gets C_{IJ} + \alpha C_g(\nabla u, a_I, b_J).
    /// $$
    ///
    /// The default implementation repeatedly calls [contract](Self::contract). However,
    /// this might often be inefficient: Since $\nabla u$ is constant for all vectors
    /// $a_I, b_J$, it's often possible to compute the operation for all vectors
    /// at once much more efficiently than one at a time. For performance reasons, it is therefore
    /// often advisable to override this method.
    ///
    /// # Panics
    ///
    /// Panics if `a.len() != b.len()` or `a.len()` is not divisible by $d$ (`GeometryDim`).
    ///
    /// Panics if `output.nrows() != s * M` or `output.ncols() != output.ncols() * N`.
    #[allow(non_snake_case)]
    fn accumulate_contractions_into(
        &self,
        mut output: DMatrixSliceMut<T>,
        alpha: T,
        gradient: &MatrixMN<T, GeometryDim, Self::SolutionDim>,
        a: DVectorSlice<T>,
        b: DVectorSlice<T>,
        parameters: &Self::Parameters,
    ) {
        let d = GeometryDim::dim();
        let s = Self::SolutionDim::dim();
        assert_eq!(
            a.len() % d,
            0,
            "Dimension of a must be divisible by d (GeometryDim)"
        );
        assert_eq!(
            b.len() % d,
            0,
            "Dimension of b must be divisible by d (GeometryDim)"
        );
        let M = a.len() / d;
        let N = b.len() / d;
        assert_eq!(
            output.nrows(),
            s * M,
            "Number of rows in output matrix is not consistent with a"
        );
        assert_eq!(
            output.ncols(),
            s * N,
            "Number of columns in output matrix is not consistent with b"
        );
        let s_times_s = (Self::SolutionDim::name(), Self::SolutionDim::name());

        // Note: We fill the matrix column-by-column since the matrix is stored in column-major
        // format
        for J in 0..N {
            for I in 0..M {
                let a_I = a.rows_generic(d * I, GeometryDim::name()).clone_owned();
                let b_J = b.rows_generic(d * J, GeometryDim::name()).clone_owned();
                let mut c_IJ = output.generic_slice_mut((s * I, s * J), s_times_s);
                let contraction = self.contract(gradient, &a_I, &b_J, parameters);
                c_IJ += contraction * alpha;
            }
        }
    }
}

/// An energy function associated with an elliptic operator.
///
/// The elliptic energy is a function $\psi: \mathbb{R}^{d \times s} \rightarrow \mathbb{R}$
/// that represents some energy-like quantity *per unit volume*. Typically the elliptic energy
/// arises in applications as the total potential energy over the domain
///
/// $$ E[u] := \int_{\Omega} \psi (\nabla u) \dx. $$
///
/// The elliptic energy is then related to the elliptic operator
/// $g: \mathbb{R}^{d \times s} \rightarrow \mathbb{R}^{d \times s}$ by the relation
///
/// $$ g = \pd{\psi}{G} $$
///
/// where $G = \nabla u$.
/// This relationship lets us connect the total energy to the weak form associated with $g$
/// by noticing that the functional derivative gives us the functional differential
/// with respect to a test function $v$
///
/// $$ \partial E = \int_{\Omega} \pd{\psi}{G} : \nabla v \dx
///     = \int_{\Omega} g : \nabla v \dx. $$
///
/// The simplest example of an elliptic energy is the
/// [Dirichlet energy](https://en.wikipedia.org/wiki/Dirichlet_energy)
/// $$ E[u] = \int_{\Omega} \frac{1}{2} \| \nabla u \|^2 \dx $$
/// where in our framework, $ \psi (\nabla u) = \frac{1}{2} \| \nabla u \|^2$ and
/// $g = \nabla u$, which gives the weak form associated with Laplace's equation.
///
///
///
/// TODO: Extend elliptic energy to have an additional domain dependence,
/// e.g. $\psi = \psi(x, \nabla u)$.
pub trait EllipticEnergy<T, GeometryDim>: Operator
where
    T: RealField,
    GeometryDim: SmallDim,
    DefaultAllocator: BiDimAllocator<T, GeometryDim, Self::SolutionDim>,
{
    fn compute_energy(
        &self,
        gradient: &MatrixMN<T, GeometryDim, Self::SolutionDim>,
        parameters: &Self::Parameters,
    ) -> T;
}
