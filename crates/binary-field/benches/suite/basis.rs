//! Changes of basis between the polynomial basis of a row and the tower basis of a crate.
//!
//! A tower starts from a field in a polynomial basis over GF(2) and doubles its degree by
//! quadratic extensions. [`Basis::new`] maps it into the row's field: a root of the base
//! polynomial among the elements of the matching subfield, then a root of each quadratic, which
//! is a linear equation over GF(2) because squaring is linear in characteristic 2.

use core::iter::successors;

use bench_compare::Limbs;

use super::reference::{Modulus, bit, flip, is_zero, xor};

/// A crate's basis of `GF(2^n)`: a tower of quadratic extensions over a field in a polynomial
/// basis.
pub struct Tower {
    /// The field at the bottom: bit `i` of an element is the coefficient of `t^i`.
    pub base: Modulus,
    /// The quadratic extensions from the bottom up.
    pub levels: Vec<Level>,
}

/// The extension `L[y] / (y² + a·y + b)` of the level `L` below. An element of it is
/// `lo + hi·y`, with the bits of `lo` below those of `hi`.
pub struct Level {
    /// The coefficient of `y`, in the bits of `L`.
    pub a: Limbs,
    /// The constant term, in the bits of `L`.
    pub b: Limbs,
}

/// An isomorphism between a row's field and a crate's basis of it.
pub struct Basis {
    /// Row element to the crate's bits.
    to_crate: Linear,
    /// The crate's bits to row element.
    to_row: Linear,
}

impl Basis {
    /// The isomorphism that sends the tower's generators to roots of their equations in `field`.
    /// Panics unless the tower spans `field`.
    pub fn new(field: &Modulus, tower: &Tower) -> Self {
        let mut images = powers(field, &base_root(field, &tower.base), tower.base.degree);
        for level in &tower.levels {
            let below = Linear {
                columns: images.clone(),
            };
            let y = quadratic_root(field, &below.apply(&level.a), &below.apply(&level.b));
            let high: Vec<Limbs> = images.iter().map(|image| field.mul(image, &y)).collect();
            images.extend(high);
        }
        assert_eq!(
            images.len(),
            field.degree,
            "a tower of {} bits for a field of degree {}",
            images.len(),
            field.degree
        );
        let to_row = Linear { columns: images };
        Self {
            to_crate: to_row.inverse(field.degree),
            to_row,
        }
    }

    /// The crate's bits of the row element `limbs`.
    pub fn crate_bits(&self, limbs: &[u64]) -> Limbs {
        self.to_crate.apply(limbs)
    }

    /// The row element of the crate's `bits`.
    pub fn row_element(&self, bits: &[u64]) -> Limbs {
        self.to_row.apply(bits)
    }
}

/// `1, z, z², …, z^(count - 1)` in `field`.
fn powers(field: &Modulus, z: &[u64], count: usize) -> Vec<Limbs> {
    successors(Some(field.power_of_x(0)), |power| Some(field.mul(power, z)))
        .take(count)
        .collect()
}

/// A root in `field` of the polynomial `base`, among the elements of the subfield of
/// `GF(2^degree(base))`: the kernel of `z -> z^(2^degree(base)) + z`.
fn base_root(field: &Modulus, base: &Modulus) -> Limbs {
    let frobenius: Vec<Limbs> = (0..field.degree)
        .map(|index| {
            let mut power = field.power_of_x(index);
            for _ in 0..base.degree {
                power = field.square(&power);
            }
            xor(&mut power, &field.power_of_x(index));
            power
        })
        .collect();
    let subfield = Linear {
        columns: Echelon::new(&frobenius, field.degree).kernel(),
    };
    assert_eq!(
        subfield.columns.len(),
        base.degree,
        "a subfield of {} bits for a base of degree {}",
        subfield.columns.len(),
        base.degree
    );
    (1..1u64 << base.degree)
        .map(|choice| subfield.apply(&[choice]))
        .find(|candidate| {
            let powers = powers(field, candidate, base.degree + 1);
            let mut value = powers[base.degree].clone();
            for tap in base.taps {
                xor(&mut value, &powers[*tap]);
            }
            is_zero(&value)
        })
        .unwrap_or_else(|| panic!("no root of {base:?} in {field:?}"))
}

/// A root `y` of `y² + a·y + b` in `field`, from the linear map `y -> y² + a·y`.
fn quadratic_root(field: &Modulus, a: &[u64], b: &[u64]) -> Limbs {
    let map: Vec<Limbs> = (0..field.degree)
        .map(|index| {
            let power = field.power_of_x(index);
            let mut image = field.square(&power);
            xor(&mut image, &field.mul(a, &power));
            image
        })
        .collect();
    Echelon::new(&map, field.degree)
        .solve(b)
        .unwrap_or_else(|| panic!("y² + {a:x?}·y + {b:x?} has no root in {field:?}"))
}

/// A linear map over GF(2), by the images of the basis vectors.
struct Linear {
    columns: Vec<Limbs>,
}

impl Linear {
    /// The image of `bits`. Panics if `bits` has a set bit past the columns.
    fn apply(&self, bits: &[u64]) -> Limbs {
        let mut image = vec![0; self.columns.first().map_or(0, Vec::len)];
        for index in (0..bits.len() * 64).filter(|index| bit(bits, *index)) {
            xor(&mut image, &self.columns[index]);
        }
        image
    }

    /// The inverse map, from images of `bits` bits back. Panics unless the map is invertible.
    fn inverse(&self, bits: usize) -> Self {
        let echelon = Echelon::new(&self.columns, bits);
        let columns = (0..bits)
            .map(|index| {
                let mut unit = vec![0; bits.div_ceil(64)];
                flip(&mut unit, index);
                echelon
                    .solve(&unit)
                    .unwrap_or_else(|| panic!("a singular map: no preimage of bit {index}"))
            })
            .collect();
        Self { columns }
    }
}

/// The Gauss–Jordan elimination of a linear map: rows that pair an image with the combination of
/// basis vectors that gives it. Each pivot row is the only row with its pivot bit; the rows
/// without a pivot have the image zero, and their combinations span the kernel.
struct Echelon {
    /// Each row: the image and the combination.
    rows: Vec<(Limbs, Limbs)>,
    /// The pivot bit of each of the first rows.
    pivots: Vec<usize>,
}

impl Echelon {
    /// The elimination of the map with `columns`, whose images have `bits` bits.
    fn new(columns: &[Limbs], bits: usize) -> Self {
        let mut rows: Vec<(Limbs, Limbs)> = columns
            .iter()
            .enumerate()
            .map(|(index, column)| {
                let mut combination = vec![0; columns.len().div_ceil(64)];
                flip(&mut combination, index);
                (column.clone(), combination)
            })
            .collect();
        let mut pivots = Vec::new();
        for pivot in 0..bits {
            let Some(found) = (pivots.len()..rows.len()).find(|row| bit(&rows[*row].0, pivot))
            else {
                continue;
            };
            rows.swap(pivots.len(), found);
            let (image, combination) = rows[pivots.len()].clone();
            for (index, row) in rows.iter_mut().enumerate() {
                if index != pivots.len() && bit(&row.0, pivot) {
                    xor(&mut row.0, &image);
                    xor(&mut row.1, &combination);
                }
            }
            pivots.push(pivot);
        }
        Self { rows, pivots }
    }

    /// A combination whose image is `target`, or `None` when `target` is outside the image.
    fn solve(&self, target: &[u64]) -> Option<Limbs> {
        let mut rest = target.to_vec();
        let mut solution = vec![0; self.rows.len().div_ceil(64)];
        for ((image, combination), pivot) in self.rows.iter().zip(&self.pivots) {
            if bit(&rest, *pivot) {
                xor(&mut rest, image);
                xor(&mut solution, combination);
            }
        }
        is_zero(&rest).then_some(solution)
    }

    /// A basis of the kernel.
    fn kernel(&self) -> Vec<Limbs> {
        self.rows[self.pivots.len()..]
            .iter()
            .map(|(_, combination)| combination.clone())
            .collect()
    }
}
