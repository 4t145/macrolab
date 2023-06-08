#![feature(generic_arg_infer)]
#[derive(Debug, Clone, Copy)]
pub struct Matrix<T, const X: usize, const Y: usize>([[T; X]; Y]);
macro_rules! mat_tranform {
    ($BX:expr, $BY: expr; $a:ident=$a_val:expr; $b:ident[$x:ident][$y:ident] := $def: expr) => {{
        use std::mem::*;
        let mut $b: [[_; $BX]; $BY] = unsafe {
            #[allow(clippy::uninit_assumed_init)]
            MaybeUninit::uninit().assume_init()
        };
        let mut $a = $a_val;
        for $x in 0..($BX) {
            for $y in 0..$BY {
                swap(&mut $b[$x][$y], &mut ($def));
            }
        }
        Matrix($b)
    }};
}

macro_rules! mat_biop {
    ($BX:expr, $BY: expr; $a:ident=$a_val:expr; $b:ident=$b_val:expr; $c:ident[$x:ident][$y:ident] := $def: expr) => {{
        use std::mem::*;
        let mut $c: [[_; $BX]; $BY] = unsafe {
            #[allow(clippy::uninit_assumed_init)]
            MaybeUninit::uninit().assume_init()
        };
        let $a = $a_val;
        let $b = $a_val;
        for $x in 0..($BX) {
            for $y in 0..$BY {
                swap(&mut $c[$x][$y], &mut ($def));
            }
        }
        Matrix($c)
    }};
}

impl<T, const X: usize, const Y: usize> Matrix<T, X, Y> {
    pub fn transpose(self) -> Matrix<T, Y, X> {
        mat_tranform!(Y,X; a=self.0; b[x][y] := a[y][x])
    }
    pub fn flip_x(self) -> Self {
        mat_tranform!(X,Y; a=self.0; b[x][y] := a[X-x][y])
    }
}

impl<T, const X: usize, const Y: usize> std::ops::Not for Matrix<T, X, Y> {
    type Output = Matrix<T, Y, X>;
    fn not(self) -> Self::Output {
        mat_tranform!(Y,X; a=self.0; b[x][y] := a[y][x])
    }
}
use std::ops::*;
impl<T, const X: usize, const Y: usize> Add for Matrix<T, X, Y>
where
    T: Add<Output = T> + Copy,
{
    type Output = Matrix<T, X, Y>;
    fn add(self, rhs: Self) -> Self::Output {
        mat_biop!(X,Y; a=self.0; b=rhs.0; c[x][y] := a[x][y] + b[x][y])
    }
}
impl<T, const X: usize, const Y: usize, const Z: usize> Mul<Matrix<T, Y, Z>> for Matrix<T, X, Y>
where
    T: Mul<Output = T> + Add<Output = T> + Copy + Default,
{
    type Output = Matrix<T, X, Z>;
    fn mul(self, rhs: Matrix<T, Y, Z>) -> Self::Output {
        mat_biop!(X,Z; a=self.0; b=rhs.0; c[x][z] := (0..Y).map(|y|a[x][y] * b[y][z]).fold(T::default(), T::add))
    }
}
macro_rules! mat {
    ($([$($e: expr),*$(,)?]),*$(,)?) => {
        $crate::mat::Matrix([
            $([$($e),*],)*
        ])
    }
}


macro_rules! max {
    ($first:expr $(,)?) => {
        $first
    };
    ($first:expr, $($v:expr),*$(,)?) => {
        $first.max(max!($($v),*))
    };
}

#[test]
fn test() {
    let mut a = 1;
    let mut b = 2;
    let max = max!(a, b, a, b, a, b);
}


pub struct Tree<T> {
    pub value: T,
    pub children: Vec<Tree<T>>,
}

macro_rules! tree {
    ($($tree:tt)+) => {
        tree_internal!($($tree)+)
    };
}
macro_rules! tree_internal {
    ($val:expr => [$($tt:tt)*]) => {
        tree_internal!( @tree $val => $($tt)* )
    };
    (@children [$($elems:expr,)*]) => {
        vec![$($elems,)*]
    };
    // done
    (@tree $object:ident () () ()) => {};
    (@tree $object:ident $val:expr => $($tt:tt)*) => {
        Tree {
            value: $val,
            children: vec![
                tree_internal!(@children $($tt)*)
            ]
        }
    };
    (@leaf $val:expr) => {
        Tree {
            value: $val,
            children: vec![]
        }
    }
}

pub fn test_tree() {
    tree!(1 => [
        2 => [
            3,
            4
        ],
        5
    ]);
}