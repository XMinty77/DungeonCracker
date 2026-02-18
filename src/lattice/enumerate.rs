use crate::math::big_fraction::BigFraction;
use crate::math::big_matrix::BigMatrix;
use crate::math::big_vector::BigVector;
use crate::math::lu_decomposition;
use crate::math::optimize::{Optimize, OptimizeBuilder};
use num_bigint::BigInt;
use num_traits::One;

/// High-level enumerate function matching Java's Enumerate.enumerate(basis, lower, upper, offset).
/// This is used by RandomReverser.findAllValidSeeds().
pub fn enumerate_bounds(
    basis: &BigMatrix,
    lower: &BigVector,
    upper: &BigVector,
    origin: &BigVector,
) -> Vec<BigVector> {
    let size = basis.row_count();
    let mut builder = OptimizeBuilder::of_size(size);
    for i in 0..size {
        builder = builder
            .with_lower_bound_idx(i, lower.get(i))
            .with_upper_bound_idx(i, upper.get(i));
    }
    let constraints = builder.build();
    enumerate(basis, origin, &constraints)
}

/// Get the total number of depth-0 branches for the enumeration tree.
/// Returns (total_branches, narrowest_dimension_index).
/// Used by the parallel WASM interface to split work across workers.
pub fn get_branch_count(
    basis: &BigMatrix,
    lower: &BigVector,
    upper: &BigVector,
    origin: &BigVector,
) -> i64 {
    let size = basis.row_count();
    let mut builder = OptimizeBuilder::of_size(size);
    for i in 0..size {
        builder = builder
            .with_lower_bound_idx(i, lower.get(i))
            .with_upper_bound_idx(i, upper.get(i));
    }
    let constraints = builder.build();

    let root_inverse = lu_decomposition::inverse(basis);
    let root_origin = root_inverse.multiply_vector(origin);

    // Compute widths and find narrowest dimension (same logic as enumerate)
    let mut widths: Vec<BigFraction> = Vec::with_capacity(size);
    let mut order: Vec<usize> = Vec::with_capacity(size);

    for i in 0..size {
        let gradient = root_inverse.get_row(i);
        let (_, min_val) = constraints.clone().minimize(&gradient);
        let (_, max_val) = constraints.clone().maximize(&gradient);
        let w = max_val.sub_frac(&min_val);
        widths.push(w);
        order.push(i);
    }

    order.sort_by(|&a, &b| widths[a].cmp(&widths[b]));

    // The narrowest dimension is order[0] — that's what depth-0 explores.
    let index = order[0];
    let gradient = root_inverse.get_row(index);
    let offset = root_origin.get(index).clone();

    let (_, min_val) = constraints.clone().minimize(&gradient);
    let (_, max_val) = constraints.clone().maximize(&gradient);

    let min_int = min_val.sub_frac(&offset).ceil();
    let max_int = max_val.sub_frac(&offset).floor();

    if min_int > max_int {
        return 0;
    }

    // Number of integer values = max_int - min_int + 1
    let count = &max_int - &min_int + BigInt::one();
    // Clamp to i64 (should always fit for reasonable problems)
    use num_traits::ToPrimitive;
    count.to_i64().unwrap_or(i64::MAX)
}

/// Enumerate only a subset of depth-0 branches [branch_start, branch_end).
/// Each "branch" is one integer value at depth 0. The values are enumerated
/// in the same order as the full enumeration (center-outward).
pub fn enumerate_bounds_partial(
    basis: &BigMatrix,
    lower: &BigVector,
    upper: &BigVector,
    origin: &BigVector,
    branch_start: i64,
    branch_end: i64,
) -> Vec<BigVector> {
    let size = basis.row_count();
    let mut builder = OptimizeBuilder::of_size(size);
    for i in 0..size {
        builder = builder
            .with_lower_bound_idx(i, lower.get(i))
            .with_upper_bound_idx(i, upper.get(i));
    }
    let constraints = builder.build();
    enumerate_partial(basis, origin, &constraints, branch_start, branch_end)
}

/// Partial enumerate: only processes depth-0 branches in [branch_start, branch_end).
fn enumerate_partial(
    basis: &BigMatrix,
    origin: &BigVector,
    constraints: &Optimize,
    branch_start: i64,
    branch_end: i64,
) -> Vec<BigVector> {
    let root_inverse = lu_decomposition::inverse(basis);
    let root_origin = root_inverse.multiply_vector(origin);
    enumerate_rt_partial(basis, origin, constraints, &root_inverse, &root_origin, branch_start, branch_end)
}

/// Low-level partial enumerate.
fn enumerate_rt_partial(
    basis: &BigMatrix,
    origin: &BigVector,
    constraints: &Optimize,
    root_inverse: &BigMatrix,
    root_origin: &BigVector,
    branch_start: i64,
    branch_end: i64,
) -> Vec<BigVector> {
    let root_size = basis.row_count();
    let root_fixed = BigVector::new(root_size);
    let root_constraints = constraints.clone();

    // Compute widths and sort (same as full enumerate)
    let mut widths: Vec<BigFraction> = Vec::with_capacity(root_size);
    let mut order: Vec<usize> = Vec::with_capacity(root_size);

    eprintln!("[enumerate-partial] Computing dimension widths for {} dimensions...", root_size);

    for i in 0..root_size {
        let gradient = root_inverse.get_row(i);
        let (_, min_val) = root_constraints.clone().minimize(&gradient);
        let (_, max_val) = root_constraints.clone().maximize(&gradient);
        let w = max_val.sub_frac(&min_val);
        widths.push(w);
        order.push(i);
    }

    order.sort_by(|&a, &b| widths[a].cmp(&widths[b]));

    let root = SearchNode {
        size: root_size,
        depth: 0,
        inverse: root_inverse.clone(),
        origin: root_origin.clone(),
        fixed: root_fixed,
        constraints: root_constraints,
        order,
    };

    // Only explore depth-0 branches in [branch_start, branch_end)
    let mut results = Vec::new();
    collect_solutions_depth0_partial(&root, &mut results, branch_start, branch_end);

    results
        .into_iter()
        .map(|fixed| {
            let transformed = basis.multiply_vector(&fixed);
            origin.add(&transformed)
        })
        .collect()
}

/// Enumerate lattice points within the feasible region defined by constraints.
/// Faithful port of LattiCG's Enumerate.java + EnumerateRt.java + SearchNode.java.
pub fn enumerate(
    basis: &BigMatrix,
    origin: &BigVector,
    constraints: &Optimize,
) -> Vec<BigVector> {
    let root_inverse = lu_decomposition::inverse(basis);
    let root_origin = root_inverse.multiply_vector(origin);
    enumerate_rt(basis, origin, constraints, &root_inverse, &root_origin)
}

/// Low-level enumerate matching EnumerateRt.enumerate().
fn enumerate_rt(
    basis: &BigMatrix,
    origin: &BigVector,
    constraints: &Optimize,
    root_inverse: &BigMatrix,
    root_origin: &BigVector,
) -> Vec<BigVector> {
    let root_size = basis.row_count();
    let root_fixed = BigVector::new(root_size);
    let root_constraints = constraints.clone();

    // Compute widths for each dimension and sort by width (narrow first)
    let mut widths: Vec<BigFraction> = Vec::with_capacity(root_size);
    let mut order: Vec<usize> = Vec::with_capacity(root_size);

    eprintln!("[enumerate] Computing dimension widths for {} dimensions (LP table: {}x{})...",
             root_size, root_constraints.table_size().0, root_constraints.table_size().1);

    for i in 0..root_size {
        let gradient = root_inverse.get_row(i);
        let (_, min_val) = root_constraints.clone().minimize(&gradient);
        let (_, max_val) = root_constraints.clone().maximize(&gradient);
        let w = max_val.sub_frac(&min_val);
        eprintln!("[enumerate]   dim {} width = {} (min={}, max={})", i, w, min_val, max_val);
        widths.push(w);
        order.push(i);
    }

    order.sort_by(|&a, &b| widths[a].cmp(&widths[b]));

    // Recursive search
    let mut results = Vec::new();
    let root = SearchNode {
        size: root_size,
        depth: 0,
        inverse: root_inverse.clone(),
        origin: root_origin.clone(),
        fixed: root_fixed,
        constraints: root_constraints,
        order,
    };

    collect_solutions(&root, &mut results);

    // Map back: result = basis * fixed + origin
    results
        .into_iter()
        .map(|fixed| {
            let transformed = basis.multiply_vector(&fixed);
            origin.add(&transformed)
        })
        .collect()
}

/// Recursively collect all lattice point solutions.
fn collect_solutions(node: &SearchNode, results: &mut Vec<BigVector>) {
    if node.depth == node.size {
        results.push(node.fixed.clone());
        if results.len() % 100 == 0 {
            eprintln!("[enumerate] Found {} solutions so far...", results.len());
        }
        return;
    }

    if node.depth <= 1 {
        eprintln!("[enumerate] Exploring depth={}/{} (dimension index={})", node.depth, node.size, node.order[node.depth]);
    }

    let index = node.order[node.depth];
    let gradient = node.inverse.get_row(index);
    let offset = node.origin.get(index).clone();

    // Minimize and maximize to find integer range
    let (_, min_val) = node.constraints.clone().minimize(&gradient);
    let (_, max_val) = node.constraints.clone().maximize(&gradient);

    let min_int = min_val.sub_frac(&offset).ceil();
    let max_int = max_val.sub_frac(&offset).floor();

    if min_int > max_int {
        return;
    }

    // Enumerate from center outward (like the Java version)
    let lower_start: BigInt = (&min_int + &max_int) >> 1;
    let upper_start = &lower_start + BigInt::one();

    let mut lower = lower_start.clone();
    let mut upper = upper_start;
    let mut either = true;

    while either {
        either = false;

        if lower >= min_int {
            let child = create_child(node, index, &lower);
            collect_solutions(&child, results);
            lower -= BigInt::one();
            either = true;
        }

        if upper <= max_int {
            let child = create_child(node, index, &upper);
            collect_solutions(&child, results);
            upper += BigInt::one();
            either = true;
        }
    }
}

/// Collect solutions for only depth-0 branches indexed [branch_start, branch_end).
/// Branch index 0 = center, then alternating outward (matching the center-outward pattern).
fn collect_solutions_depth0_partial(
    node: &SearchNode,
    results: &mut Vec<BigVector>,
    branch_start: i64,
    branch_end: i64,
) {
    assert_eq!(node.depth, 0, "collect_solutions_depth0_partial must start at depth 0");

    let index = node.order[0];
    let gradient = node.inverse.get_row(index);
    let offset = node.origin.get(index).clone();

    let (_, min_val) = node.constraints.clone().minimize(&gradient);
    let (_, max_val) = node.constraints.clone().maximize(&gradient);

    let min_int = min_val.sub_frac(&offset).ceil();
    let max_int = max_val.sub_frac(&offset).floor();

    if min_int > max_int {
        return;
    }

    // Build the full list of depth-0 integer values in center-outward order
    let center: BigInt = (&min_int + &max_int) >> 1;
    let mut all_values: Vec<BigInt> = Vec::new();

    let mut lower = center.clone();
    let upper_start = &center + BigInt::one();
    let mut upper = upper_start;
    let mut either = true;

    while either {
        either = false;
        if lower >= min_int {
            all_values.push(lower.clone());
            lower -= BigInt::one();
            either = true;
        }
        if upper <= max_int {
            all_values.push(upper.clone());
            upper += BigInt::one();
            either = true;
        }
    }

    let total = all_values.len() as i64;
    let start = branch_start.max(0) as usize;
    let end = (branch_end.min(total) as usize).min(all_values.len());

    eprintln!("[enumerate-partial] Exploring branches {}-{} of {} at depth 0 (dim index={})",
             start, end, total, index);

    for idx in start..end {
        let val = &all_values[idx];
        let child = create_child(node, index, val);
        collect_solutions(&child, results);
    }
}

fn create_child(parent: &SearchNode, index: usize, i: &BigInt) -> SearchNode {
    let gradient = parent.inverse.get_row(index);
    let offset = parent.origin.get(index).clone();
    let value = BigFraction::from_bigint(i.clone());

    let next_constraints = parent.constraints.with_strict_bound(&gradient, &value.add_frac(&offset));
    let basis_vec = BigVector::basis(parent.size, index, value);
    let next_fixed = parent.fixed.add(&basis_vec);

    SearchNode {
        size: parent.size,
        depth: parent.depth + 1,
        inverse: parent.inverse.clone(),
        origin: parent.origin.clone(),
        fixed: next_fixed,
        constraints: next_constraints,
        order: parent.order.clone(),
    }
}

struct SearchNode {
    size: usize,
    depth: usize,
    inverse: BigMatrix,
    origin: BigVector,
    fixed: BigVector,
    constraints: Optimize,
    order: Vec<usize>,
}
