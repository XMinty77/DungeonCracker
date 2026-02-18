use super::big_fraction::BigFraction;
use num_bigint::BigInt;
use std::fmt;

/// A vector of BigFraction values.
#[derive(Clone, Debug)]
pub struct BigVector {
    data: Vec<BigFraction>,
}

impl BigVector {
    pub fn new(dimension: usize) -> Self {
        BigVector {
            data: vec![BigFraction::zero(); dimension],
        }
    }

    pub fn from_data(data: Vec<BigFraction>) -> Self {
        BigVector { data }
    }

    pub fn dimension(&self) -> usize {
        self.data.len()
    }

    pub fn get(&self, i: usize) -> &BigFraction {
        &self.data[i]
    }

    pub fn set(&mut self, i: usize, value: BigFraction) {
        self.data[i] = value;
    }

    pub fn magnitude_sq(&self) -> BigFraction {
        let mut mag = BigFraction::zero();
        for i in 0..self.dimension() {
            mag = mag.add_frac(&self.data[i].mul_frac(&self.data[i]));
        }
        mag
    }

    pub fn is_zero(&self) -> bool {
        self.data.iter().all(|x| x.signum() == 0)
    }

    pub fn add(&self, other: &BigVector) -> BigVector {
        assert_eq!(self.dimension(), other.dimension());
        BigVector {
            data: self
                .data
                .iter()
                .zip(other.data.iter())
                .map(|(a, b)| a.add_frac(b))
                .collect(),
        }
    }

    pub fn subtract(&self, other: &BigVector) -> BigVector {
        assert_eq!(self.dimension(), other.dimension());
        BigVector {
            data: self
                .data
                .iter()
                .zip(other.data.iter())
                .map(|(a, b)| a.sub_frac(b))
                .collect(),
        }
    }

    pub fn subtract_assign(&mut self, other: &BigVector) {
        assert_eq!(self.dimension(), other.dimension());
        for i in 0..self.dimension() {
            self.data[i] = self.data[i].sub_frac(&other.data[i]);
        }
    }

    pub fn add_assign(&mut self, other: &BigVector) {
        assert_eq!(self.dimension(), other.dimension());
        for i in 0..self.dimension() {
            self.data[i] = self.data[i].add_frac(&other.data[i]);
        }
    }

    pub fn multiply_scalar(&self, scalar: &BigFraction) -> BigVector {
        BigVector {
            data: self.data.iter().map(|x| x.mul_frac(scalar)).collect(),
        }
    }

    pub fn multiply_bigint(&self, scalar: &BigInt) -> BigVector {
        BigVector {
            data: self.data.iter().map(|x| x.mul_bigint(scalar)).collect(),
        }
    }

    pub fn multiply_scalar_assign(&mut self, scalar: &BigFraction) {
        for i in 0..self.dimension() {
            self.data[i] = self.data[i].mul_frac(scalar);
        }
    }

    pub fn divide_scalar_assign(&mut self, scalar: &BigFraction) {
        let recip = scalar.reciprocal();
        self.multiply_scalar_assign(&recip);
    }

    pub fn dot(&self, other: &BigVector) -> BigFraction {
        assert_eq!(self.dimension(), other.dimension());
        let mut result = BigFraction::zero();
        for i in 0..self.dimension() {
            result = result.add_frac(&self.data[i].mul_frac(&other.data[i]));
        }
        result
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        self.data.swap(i, j);
    }

    /// Create a basis vector e_i of given dimension with value `scale` at position `i`.
    pub fn basis(size: usize, i: usize, scale: BigFraction) -> BigVector {
        let mut v = BigVector::new(size);
        v.set(i, scale);
        v
    }

    pub fn basis_one(size: usize, i: usize) -> BigVector {
        Self::basis(size, i, BigFraction::one())
    }
}

impl fmt::Display for BigVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        for i in 0..self.dimension() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", self.data[i])?;
        }
        write!(f, "}}")
    }
}
