use std::ops::Neg;

use crate::chunk::CHUNK_SIZE;

/// A position in block coordinates.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct BlockPosition(pub i64, pub i64, pub i64);

impl BlockPosition {
    /// The size of a chunk in block coordinates.
    pub const CHUNK_SIZE: Self = Self::new(CHUNK_SIZE as i64, CHUNK_SIZE as i64, CHUNK_SIZE as i64);

    /// Creates a new BlockPosition.
    pub const fn new(x: i64, y: i64, z: i64) -> Self {
        Self(x, y, z)
    }

    /// Converts the block position to a tuple of i64 coordinates.
    pub fn to_tuple(&self) -> (i64, i64, i64) {
        (self.0, self.1, self.2)
    }
    /// Wrap the block position within chunk bounds.
    pub fn chunk_normalize(&self) -> Self {
        Self(
            self.0.rem_euclid(CHUNK_SIZE as i64),
            self.1.rem_euclid(CHUNK_SIZE as i64),
            self.2.rem_euclid(CHUNK_SIZE as i64),
        )
    }

    /// Subtracts the normalized chunk position from the original position.
    pub fn chunk_reduce(&self) -> Self {
        *self - self.chunk_normalize()
    }

    /// Offsets the block position by one unit in the direction of the cardinal direction.
    pub fn offset(&self, dir: crate::graphics::CardinalDirection) -> Self {
        let (dx, dy, dz) = dir.normal_i64();
        Self(self.0 + dx, self.1 + dy, self.2 + dz)
    }

    /// Applies the given inspector function to all components, returning true if all pass.
    pub fn all(&self, inspector: impl Fn(i64) -> bool) -> bool {
        inspector(self.0) && inspector(self.1) && inspector(self.2)
    }
}

// time for math impl chaingun ughm

macro_rules! math_op {
    ($op: tt, $func_name: ident, $trait: ident) => {
        impl std::ops::$trait for BlockPosition {
            type Output = BlockPosition;

            fn $func_name(self, other: BlockPosition) -> BlockPosition {
                BlockPosition(
                    self.0 $op other.0,
                    self.1 $op other.1,
                    self.2 $op other.2,
                )
            }
        }
    };

    // support for multiple traits at once
    (($($op: tt),*), ($($func_name: ident),*), ($($trait: ident),*)) => {
        $(
            math_op!($op, $func_name, $trait);
        )*
    };
}

math_op!(
    (+, -, *, /, %),
    (add, sub, mul, div, rem),
    (Add, Sub, Mul, Div, Rem)
);

impl From<(i64, i64, i64)> for BlockPosition {
    fn from(tuple: (i64, i64, i64)) -> Self {
        BlockPosition(tuple.0, tuple.1, tuple.2)
    }
}

impl Neg for BlockPosition {
    type Output = BlockPosition;

    fn neg(self) -> BlockPosition {
        BlockPosition(-self.0, -self.1, -self.2)
    }
}

/// Convenience function to create a BlockPosition.
pub fn bp(x: i64, y: i64, z: i64) -> BlockPosition {
    BlockPosition::new(x, y, z)
}
