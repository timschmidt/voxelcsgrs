use crate::VoxelCSG;
use grid_tree::{NodePtr, VisitCommand};

#[cfg(test)]
mod tests {
    use super::*;
    use grid_tree::glam::IVec3;

    /// A helper to count how many leaf voxels are `true` in the entire octree.
    fn count_filled_voxels(csg: &VoxelCSG) -> usize {
        let mut count = 0;
        for (root_key, root_node) in csg.tree.iter_roots() {
            let root_ptr = NodePtr::new(root_key.level, root_node.self_ptr);
            csg.tree.visit_tree_depth_first(root_ptr, root_key.coordinates, 0, |ptr, _coords| {
                if ptr.level() == 0 {
                    if let Some(&val) = csg.tree.get_value(ptr) {
                        if val {
                            count += 1;
                        }
                    }
                }
                VisitCommand::Continue
            });
        }
        count
    }

    /// Asserts that each voxel in `coords_set` is `true` and all others
    /// in the bounding region are `false`.
    /// - `bounds_min`, `bounds_max` are the inclusive bounding region for the check.
    fn assert_voxels_match(
        csg: &VoxelCSG,
        coords_set: &std::collections::HashSet<IVec3>,
        bounds_min: IVec3,
        bounds_max: IVec3,
    ) {
        for z in bounds_min.z..=bounds_max.z {
            for y in bounds_min.y..=bounds_max.y {
                for x in bounds_min.x..=bounds_max.x {
                    let p = IVec3::new(x, y, z);
                    let expected = coords_set.contains(&p);
                    let actual = csg.get_voxel(p);
                    assert_eq!(
                        actual, expected,
                        "Mismatch at {:?}: expected {}, got {}",
                        p, expected, actual
                    );
                }
            }
        }
    }

    // ------------------------------------------------------------
    // 1) Basic creation & properties
    // ------------------------------------------------------------
    #[test]
    fn test_new_voxel_csg() {
        let height = 4;
        let csg = VoxelCSG::new(height);
        assert_eq!(csg.tree.height(), height as u8);
        // Initially, it should have no filled nodes
        assert_eq!(count_filled_voxels(&csg), 0);
    }

    #[test]
    #[should_panic(expected = "height must be > 1")]
    fn test_new_voxel_csg_height_too_small() {
        // If your crate has a requirement that height > 1, we test that it panics or returns Err.
        // (Adjust this test to match your actual error-handling behavior.)
        let _csg = VoxelCSG::new(1);
    }

    // ------------------------------------------------------------
    // 2) Filling primitives (cube, sphere, cylinder, polyhedron)
    // ------------------------------------------------------------
    #[test]
    fn test_fill_cube() {
        let mut csg = VoxelCSG::new(4);
        // Fill a cube from (0,0,0) to (2,2,2). This should fill 2x2x2 = 8 voxels.
        csg.fill_cube(IVec3::new(0, 0, 0), IVec3::new(2, 2, 2));

        // Check we have 8 total filled voxels
        assert_eq!(count_filled_voxels(&csg), 8);

        // Spot check a few known-filled coordinates:
        assert!(csg.get_voxel(IVec3::new(0, 0, 0)));
        assert!(csg.get_voxel(IVec3::new(1, 1, 1)));

        // Spot check a few known-empty coordinates:
        assert!(!csg.get_voxel(IVec3::new(2, 2, 2)));
        assert!(!csg.get_voxel(IVec3::new(-1, -1, -1)));
    }

    #[test]
    fn test_fill_sphere() {
        let mut csg = VoxelCSG::new(5);
        // Fill a sphere of radius 2.0 at center (0,0,0).
        csg.fill_sphere(IVec3::new(0, 0, 0), 2.0);

        // Rough check: the sphere of radius 2 should fill points that satisfy x^2+y^2+z^2 <= 4
        // That includes:
        //   (0,0,0), (1,0,0), (0,1,0), (0,0,1), (-1,0,0), etc.
        // We do a quick approximate check for some coordinates.

        // Definitely inside:
        assert!(csg.get_voxel(IVec3::new(0, 0, 0)));
        assert!(csg.get_voxel(IVec3::new(1, 0, 0)));
        assert!(csg.get_voxel(IVec3::new(-1, -1, 0)));

        // Definitely outside:
        assert!(!csg.get_voxel(IVec3::new(2, 2, 0)));
        assert!(!csg.get_voxel(IVec3::new(3, 0, 0)));

        // Optionally, check total count is within an expected range. 
        // The volume of a radius-2 sphere in continuous space is ~33.51, 
        // but with integer lattice, we won't match exactly. We just 
        // expect it to be in some plausible range, e.g. 25..45.
        let filled_count = count_filled_voxels(&csg);
        assert!(
            (25..50).contains(&filled_count),
            "Unexpected count of filled voxels: {}",
            filled_count
        );
    }

    #[test]
    fn test_fill_cylinder() {
        let mut csg = VoxelCSG::new(5);
        // Fill a cylinder of radius=2, height=5, base at (0,0,0).
        csg.fill_cylinder(IVec3::new(0, 0, 0), 5, 2.0);

        // This cylinder extends from z=0 to z=4 (inclusive of 0, exclusive of 5 in your code, 
        // but adjust if you do something else).
        // Check some known inside points:
        assert!(csg.get_voxel(IVec3::new(0, 0, 0)));
        assert!(csg.get_voxel(IVec3::new(1, 1, 2)));
        // Check outside points:
        assert!(!csg.get_voxel(IVec3::new(3, 0, 1))); // outside radius
        assert!(!csg.get_voxel(IVec3::new(1, 1, 5))); // outside height
    }

    #[test]
    fn test_fill_polyhedron_stub() {
        let mut csg = VoxelCSG::new(4);

        // We'll define a small bounding box region for the test polyhedron
        // and supply a trivial set of vertices/faces.
        // The actual `point_in_polyhedron` here is a stub that checks (x+y+z)%2 == 0,
        // so we can predict which points are "inside".
        let poly_min = IVec3::new(-2, -2, -2);
        let poly_max = IVec3::new(2, 2, 2);

        let vertices = vec![
            IVec3::new(0, 0, 0),
            IVec3::new(1, 0, 0),
            IVec3::new(0, 1, 0),
            // etc...
        ];
        let indices = vec![(0,1,2)];

        csg.fill_polyhedron(poly_min, poly_max, &vertices, &indices);

        // Because the stub uses `(x + y + z) % 2 == 0` as "inside," let's
        // just spot check that some coords match:
        assert_eq!(csg.get_voxel(IVec3::new(0,0,0)), true);
        assert_eq!(csg.get_voxel(IVec3::new(1,0,0)), false);
        assert_eq!(csg.get_voxel(IVec3::new(-1,1,0)), true);
    }

    // ------------------------------------------------------------
    // 3) CSG operations (union, intersection, difference, inversion)
    // ------------------------------------------------------------

    #[test]
    fn test_union() {
        let mut csg1 = VoxelCSG::new(4);
        csg1.fill_cube(IVec3::new(0,0,0), IVec3::new(2,2,2)); // small 2x2x2 block

        let mut csg2 = VoxelCSG::new(4);
        csg2.fill_cube(IVec3::new(1,1,0), IVec3::new(3,2,2)); // partial overlap

        let union_csg = csg1.union(&csg2);
        // Count. The first shape has 2*2*2 = 8. The second shape has (2*1*2)=4 
        // but overlap region is (1,1,0) to (2,2,2) which is 1*1*2 = 2 voxels.
        // So the union should be 8 + 4 - 2 = 10.
        assert_eq!(count_filled_voxels(&union_csg), 10);

        // Quick spot check:
        // (1,1,0) is in both shapes => definitely in union.
        assert!(union_csg.get_voxel(IVec3::new(1,1,0)));
        // (2,1,1) only in csg2 => also in union.
        assert!(union_csg.get_voxel(IVec3::new(2,1,1)));
        // (3,1,1) only in csg2 => also in union.
        assert!(union_csg.get_voxel(IVec3::new(3,1,1)));
        // (0,0,0) only in csg1 => in union
        assert!(union_csg.get_voxel(IVec3::new(0,0,0)));
    }

    #[test]
    fn test_intersection() {
        let mut csg1 = VoxelCSG::new(4);
        csg1.fill_cube(IVec3::new(0,0,0), IVec3::new(2,2,2)); 

        let mut csg2 = VoxelCSG::new(4);
        csg2.fill_cube(IVec3::new(1,1,0), IVec3::new(3,2,2)); 

        let intersect_csg = csg1.intersection(&csg2);
        // Overlap region is from x=1..2, y=1..2, z=0..2 => that is 1*1*2=2 voxels: 
        // specifically (1,1,0) and (1,1,1).
        assert_eq!(count_filled_voxels(&intersect_csg), 2);

        // (1,1,0) is in both => true 
        assert!(intersect_csg.get_voxel(IVec3::new(1,1,0)));
        // (2,1,1) is in csg2 but outside csg1 => false
        assert!(!intersect_csg.get_voxel(IVec3::new(2,1,1)));
        // (0,0,0) is in csg1 but not csg2 => false
        assert!(!intersect_csg.get_voxel(IVec3::new(0,0,0)));
    }

    #[test]
    fn test_difference() {
        let mut csg1 = VoxelCSG::new(4);
        csg1.fill_cube(IVec3::new(0,0,0), IVec3::new(3,3,3)); // 3x3x3 = 27 voxels
        let mut csg2 = VoxelCSG::new(4);
        csg2.fill_cube(IVec3::new(1,1,0), IVec3::new(3,3,3)); // partial overlap

        let diff_csg = csg1.difference(&csg2);
        // Overlap region is x=1..3, y=1..3, z=0..3 => that's 2*2*3=12 voxels. 
        // csg1 has 27 total. 
        // difference => 27 - 12 = 15 remain.
        let count_diff = count_filled_voxels(&diff_csg);
        assert_eq!(count_diff, 15);

        // Check a few specific coordinates:
        // (0,0,0) => in csg1 but not in csg2, so difference => true
        assert!(diff_csg.get_voxel(IVec3::new(0,0,0)));
        // (1,1,1) => in csg1 and csg2, so difference => false
        assert!(!diff_csg.get_voxel(IVec3::new(1,1,1)));
        // (2,2,2) => in both => false
        assert!(!diff_csg.get_voxel(IVec3::new(2,2,2)));
    }

    #[test]
    fn test_invert_in_place() {
        let mut csg = VoxelCSG::new(4);
        // Fill a small set of voxels:
        csg.fill_cube(IVec3::new(0,0,0), IVec3::new(2,2,1)); 
        // That's 2*2*1=4 voxels: (0,0,0), (0,1,0), (1,0,0), (1,1,0)
        assert_eq!(count_filled_voxels(&csg), 4);

        // Invert in place
        csg.invert_in_place();

        // Now the previously "true" nodes become false, and all other *existing* nodes become true.
        // Since we only had 4 leaf nodes allocated and they were "true," each flips to "false."
        // So effectively, we expect all 4 to become "false."
        // However, note that "invert_in_place" will only flip *existing allocated nodes* in the tree.
        // This might leave 4 nodes that are now "true = false => flipped to false" 
        // or they might remain allocated as "false." 
        // If your implementation prunes them automatically, the total count is 0. 
        // If it doesn't prune them, we still have 4 nodes, but all are "false."

        let new_count = count_filled_voxels(&csg);
        assert_eq!(
            new_count, 
            0,
            "After inverting, we expect 0 'true' voxels if newly-false nodes are not replaced."
        );

        // Confirm each voxel we had is now false:
        for x in 0..2 {
            for y in 0..2 {
                assert!(!csg.get_voxel(IVec3::new(x,y,0)));
            }
        }
    }

    #[test]
    fn test_invert() {
        // This test verifies the non-in-place version that returns a new shape.
        let mut csg = VoxelCSG::new(4);
        csg.fill_cube(IVec3::new(-1,-1,0), IVec3::new(1,1,1)); // 2*2*1 = 4 voxels

        let inverted = csg.invert();
        // Original is unchanged:
        assert_eq!(count_filled_voxels(&csg), 4);
        // Inverted result should have 0 "true" (assuming no new nodes are allocated for previously non-existent coords).
        assert_eq!(count_filled_voxels(&inverted), 0);
    }

    // ------------------------------------------------------------
    // 4) Edge cases
    // ------------------------------------------------------------
    #[test]
    fn test_fill_cube_zero_sized() {
        let mut csg = VoxelCSG::new(4);
        // If min == max, we fill nothing.
        csg.fill_cube(IVec3::new(1,1,1), IVec3::new(1,1,1));
        assert_eq!(count_filled_voxels(&csg), 0);
    }

    #[test]
    fn test_fill_cube_negative_coords() {
        let mut csg = VoxelCSG::new(4);
        csg.fill_cube(IVec3::new(-2,-2,-2), IVec3::new(-1,-1,-1));
        // That is a 1x1x1 region => only the coordinate (-2, -2, -2) 
        // if we interpret [min, max) as an exclusive upper bound. 
        // Actually, if your fill logic is [min, max) for each dimension, 
        // that is size = (|-2 - (-1)|, etc.) = (1,1,1) => 1 voxel.
        assert_eq!(count_filled_voxels(&csg), 1);
        assert!(csg.get_voxel(IVec3::new(-2,-2,-2)));
    }

    #[test]
    fn test_large_fill() {
        let mut csg = VoxelCSG::new(5);
        // Level=0 cell resolution is 1 voxel per integer coordinate.
        // Height=5 => the max coordinate range is quite large, but let's just fill a portion:
        csg.fill_cube(IVec3::new(100,100,100), IVec3::new(105,105,105));
        // That's 5x5x5=125 voxels.
        assert_eq!(count_filled_voxels(&csg), 125);
    }

    // ------------------------------------------------------------
    // 5) Saving to .vox - basic smoke test
    // ------------------------------------------------------------
    #[test]
    fn test_save_to_magicavoxel_smoke() {
        // We won't fully parse the resulting file, but we can do a quick check
        // that "save_to_magicavoxel" doesn't error out.
        // You can manually inspect the resulting file if desired.
        let mut csg = VoxelCSG::new(4);
        csg.fill_cube(IVec3::new(0,0,0), IVec3::new(2,2,2));

        // Use a temp file in tests:
        let path = "test_output.vox";
        let result = csg.save_to_magicavoxel(path);
        assert!(result.is_ok(), "Saving to MagicaVoxel file failed: {:?}", result);

        // Optionally, clean up the file if you wish, e.g.
        // std::fs::remove_file(path).ok();
    }
}

