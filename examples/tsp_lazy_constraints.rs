#![allow(unused_doc_comments)]
use fnv::FnvHashMap;
use grb::callback;
use grb::prelude::*;
use std::iter::IntoIterator;

const N: usize = 29;

/// Data taken from [ZIB](http://elib.zib.de/pub/mp-testdata/tsp/tsplib/tsp/bays29.tsp)
const TRAVEL_TIMES: [usize; N * N] = [
    0, 107, 241, 190, 124, 80, 316, 76, 152, 157, 283, 133, 113, 297, 228, 129, 348, 276, 188, 150,
    65, 341, 184, 67, 221, 169, 108, 45, 167, 107, 0, 148, 137, 88, 127, 336, 183, 134, 95, 254,
    180, 101, 234, 175, 176, 265, 199, 182, 67, 42, 278, 271, 146, 251, 105, 191, 139, 79, 241,
    148, 0, 374, 171, 259, 509, 317, 217, 232, 491, 312, 280, 391, 412, 349, 422, 356, 355, 204,
    182, 435, 417, 292, 424, 116, 337, 273, 77, 190, 137, 374, 0, 202, 234, 222, 192, 248, 42, 117,
    287, 79, 107, 38, 121, 152, 86, 68, 70, 137, 151, 239, 135, 137, 242, 165, 228, 205, 124, 88,
    171, 202, 0, 61, 392, 202, 46, 160, 319, 112, 163, 322, 240, 232, 314, 287, 238, 155, 65, 366,
    300, 175, 307, 57, 220, 121, 97, 80, 127, 259, 234, 61, 0, 386, 141, 72, 167, 351, 55, 157,
    331, 272, 226, 362, 296, 232, 164, 85, 375, 249, 147, 301, 118, 188, 60, 185, 316, 336, 509,
    222, 392, 386, 0, 233, 438, 254, 202, 439, 235, 254, 210, 187, 313, 266, 154, 282, 321, 298,
    168, 249, 95, 437, 190, 314, 435, 76, 183, 317, 192, 202, 141, 233, 0, 213, 188, 272, 193, 131,
    302, 233, 98, 344, 289, 177, 216, 141, 346, 108, 57, 190, 245, 43, 81, 243, 152, 134, 217, 248,
    46, 72, 438, 213, 0, 206, 365, 89, 209, 368, 286, 278, 360, 333, 284, 201, 111, 412, 321, 221,
    353, 72, 266, 132, 111, 157, 95, 232, 42, 160, 167, 254, 188, 206, 0, 159, 220, 57, 149, 80,
    132, 193, 127, 100, 28, 95, 193, 241, 131, 169, 200, 161, 189, 163, 283, 254, 491, 117, 319,
    351, 202, 272, 365, 159, 0, 404, 176, 106, 79, 161, 165, 141, 95, 187, 254, 103, 279, 215, 117,
    359, 216, 308, 322, 133, 180, 312, 287, 112, 55, 439, 193, 89, 220, 404, 0, 210, 384, 325, 279,
    415, 349, 285, 217, 138, 428, 310, 200, 354, 169, 241, 112, 238, 113, 101, 280, 79, 163, 157,
    235, 131, 209, 57, 176, 210, 0, 186, 117, 75, 231, 165, 81, 85, 92, 230, 184, 74, 150, 208,
    104, 158, 206, 297, 234, 391, 107, 322, 331, 254, 302, 368, 149, 106, 384, 186, 0, 69, 191, 59,
    35, 125, 167, 255, 44, 309, 245, 169, 327, 246, 335, 288, 228, 175, 412, 38, 240, 272, 210,
    233, 286, 80, 79, 325, 117, 69, 0, 122, 122, 56, 56, 108, 175, 113, 240, 176, 125, 280, 177,
    266, 243, 129, 176, 349, 121, 232, 226, 187, 98, 278, 132, 161, 279, 75, 191, 122, 0, 244, 178,
    66, 160, 161, 235, 118, 62, 92, 277, 55, 155, 275, 348, 265, 422, 152, 314, 362, 313, 344, 360,
    193, 165, 415, 231, 59, 122, 244, 0, 66, 178, 198, 286, 77, 362, 287, 228, 358, 299, 380, 319,
    276, 199, 356, 86, 287, 296, 266, 289, 333, 127, 141, 349, 165, 35, 56, 178, 66, 0, 112, 132,
    220, 79, 296, 232, 181, 292, 233, 314, 253, 188, 182, 355, 68, 238, 232, 154, 177, 284, 100,
    95, 285, 81, 125, 56, 66, 178, 112, 0, 128, 167, 169, 179, 120, 69, 283, 121, 213, 281, 150,
    67, 204, 70, 155, 164, 282, 216, 201, 28, 187, 217, 85, 167, 108, 160, 198, 132, 128, 0, 88,
    211, 269, 159, 197, 172, 189, 182, 135, 65, 42, 182, 137, 65, 85, 321, 141, 111, 95, 254, 138,
    92, 255, 175, 161, 286, 220, 167, 88, 0, 299, 229, 104, 236, 110, 149, 97, 108, 341, 278, 435,
    151, 366, 375, 298, 346, 412, 193, 103, 428, 230, 44, 113, 235, 77, 79, 169, 211, 299, 0, 353,
    289, 213, 371, 290, 379, 332, 184, 271, 417, 239, 300, 249, 168, 108, 321, 241, 279, 310, 184,
    309, 240, 118, 362, 296, 179, 269, 229, 353, 0, 121, 162, 345, 80, 189, 342, 67, 146, 292, 135,
    175, 147, 249, 57, 221, 131, 215, 200, 74, 245, 176, 62, 287, 232, 120, 159, 104, 289, 121, 0,
    154, 220, 41, 93, 218, 221, 251, 424, 137, 307, 301, 95, 190, 353, 169, 117, 354, 150, 169,
    125, 92, 228, 181, 69, 197, 236, 213, 162, 154, 0, 352, 147, 247, 350, 169, 105, 116, 242, 57,
    118, 437, 245, 72, 200, 359, 169, 208, 327, 280, 277, 358, 292, 283, 172, 110, 371, 345, 220,
    352, 0, 265, 178, 39, 108, 191, 337, 165, 220, 188, 190, 43, 266, 161, 216, 241, 104, 246, 177,
    55, 299, 233, 121, 189, 149, 290, 80, 41, 147, 265, 0, 124, 263, 45, 139, 273, 228, 121, 60,
    314, 81, 132, 189, 308, 112, 158, 335, 266, 155, 380, 314, 213, 182, 97, 379, 189, 93, 247,
    178, 124, 0, 199, 167, 79, 77, 205, 97, 185, 435, 243, 111, 163, 322, 238, 206, 288, 243, 275,
    319, 253, 281, 135, 108, 332, 342, 218, 350, 39, 263, 199, 0,
];

/// Groups edges into subtours
fn build_cycles(edges_used: impl IntoIterator<Item = (usize, usize)>) -> Vec<Vec<(usize, usize)>> {
    // Each node must have exactly 2 neighbours
    let mut neighbours: FnvHashMap<usize, Vec<usize>> =
        (0..N).map(|i| (i, Vec::with_capacity(2))).collect();
    for (i, j) in edges_used {
        neighbours.get_mut(&i).unwrap().push(j);
        neighbours.get_mut(&j).unwrap().push(i);
    }

    let mut cycles = Vec::new();

    while let Some(&start) = neighbours.keys().next() {
        let mut cycle = Vec::new();
        let mut current = start;
        let mut neigh = neighbours.remove(&current).unwrap();

        // Here we move from the current node to an unvisited neighbouring node.
        // A node is unvisited if it still in the `neighbours` Map.
        loop {
            let n1 = neigh[0];
            let n2 = neigh[1];

            // try to move to n1 first
            let (next, next_neigh) = if let Some(next_neigh) = neighbours.remove(&n1) {
                (n1, next_neigh)
            } else {
                // otherwise,try to move to n2
                if let Some(next_neigh) = neighbours.remove(&n2) {
                    (n2, next_neigh)
                } else {
                    // If both neighbours have been visited, we must have completed the cycle
                    cycle.push(if current < start {
                        (current, start)
                    } else {
                        (start, current)
                    });
                    break;
                }
            };

            cycle.push(if current < next {
                (current, next)
            } else {
                (next, current)
            });
            neigh = next_neigh;
            current = next;
        }

        cycles.push(cycle)
    }
    cycles
}

fn get_edges_used(
    model: &Model,
    xvars: &FnvHashMap<(usize, usize), Var>,
) -> grb::Result<Vec<(usize, usize)>> {
    let mut edges = Vec::with_capacity(N);
    for (e, var) in xvars {
        if model.get_obj_attr(attr::X, var)? > 0.9 {
            edges.push(*e);
        }
    }
    Ok(edges)
}

/// Callback object to separate subtour-elimination cuts
struct SubtourElimination<'a> {
    edge_vars: &'a FnvHashMap<(usize, usize), Var>,
    subtours_found: usize,
}

impl<'a> SubtourElimination<'a> {
    pub fn new(edge_vars: &'a FnvHashMap<(usize, usize), Var>) -> SubtourElimination<'a> {
        SubtourElimination {
            edge_vars,
            subtours_found: 0,
        }
    }
}

// This must be implemented for the struct to work with `Model::optimize_with_callback`
impl callback::Callback for SubtourElimination<'_> {
    fn callback(&mut self, w: Where) -> callback::CbResult {
        if let Where::MIPSol(ctx) = w {
            let edges_used: Vec<_> = self
                .edge_vars
                .keys()
                .zip(ctx.get_solution(self.edge_vars.values())?)
                .filter_map(|(e, val)| if val > 0.9 { Some(*e) } else { None })
                .collect();

            let cycles = build_cycles(edges_used);

            if cycles.len() > 1 {
                // found subtours
                for cycle in cycles {
                    println!("subtour: {}", fmt_cycle(&cycle));
                    ctx.add_lazy(c!(
                        cycle.iter().map(|e| self.edge_vars[e]).grb_sum() <= cycle.len() - 1
                    ))?;
                    self.subtours_found += 1;
                }
            }
        }

        Ok(())
    }
}

/// Given `[ (a,b), (b,c), (c,d), (e,d),...,(a,z)]` returns `[a,b,c,d,e,...,z,a]`
fn cycle_to_node_order(cycle: &[(usize, usize)]) -> Vec<usize> {
    let mut order = Vec::with_capacity(cycle.len() + 1);

    let mut iter = cycle.iter();
    let (mut a, mut b) = iter.next().unwrap();
    // a or b could be the first node, fix this up afterwards to avoid a lookahead
    order.push(a);
    order.push(b);

    for &(i, j) in iter {
        order.push(if i != a && i != b { i } else { j });
        a = i;
        b = j;
    }

    // fix-up
    if order[0] != *order.last().unwrap() {
        order.swap(0, 1);
    }
    order
}

/// Pretty-format a cycle of edges
fn fmt_cycle(cycle: &[(usize, usize)]) -> String {
    let order: Vec<_> = cycle_to_node_order(cycle)
        .into_iter()
        .map(|i| format!("{}", i))
        .collect();
    order.join(" -> ")
}

fn main() -> grb::Result<()> {
    let mut model = Model::new("TSP")?;

    // Define variables and node -> edges lookup
    let mut edges_by_node = vec![vec![]; N];
    let mut x = FnvHashMap::default();
    for i in 0..N {
        for j in i + 1..N {
            edges_by_node[i].push((i, j));
            edges_by_node[j].push((i, j));
            x.insert(
                (i, j),
                add_binvar!(model, obj: TRAVEL_TIMES[i * N + j], name: &format!("X[{},{}]", i, j))?,
            );
        }
    }
    let edge_by_node = edges_by_node;
    let x = x;

    // Define constraints (only need cover constraints)
    let cover_const: grb::Result<Vec<_>> = (0..N)
        .map(|i| {
            model.add_constr(
                &format!("cover[{}]", i),
                c!(edge_by_node[i].iter().map(|edge| x[edge]).grb_sum() == 2),
            )
        })
        .collect();
    #[allow(unused_variables)]
    let cover_const = cover_const?;

    model.update()?;
    model.write("tsp.lp")?;

    // needs to be set whenever working with lazy constraints
    model.set_param(param::LazyConstraints, 1)?;

    let mut callback = SubtourElimination::new(&x);
    model.optimize_with_callback(&mut callback)?;

    let edges_used = get_edges_used(&model, &x)?;
    let cycles = build_cycles(edges_used.iter().copied());
    assert_eq!(cycles.len(), 1);
    println!("{}", fmt_cycle(&cycles[0]));
    println!("{} subtours cut off", callback.subtours_found);

    Ok(())
}
