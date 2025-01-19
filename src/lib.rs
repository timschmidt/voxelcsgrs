use grid_tree::{
    glam::{IVec3, Vec3Swizzles},
    // The type alias for a 3D octree with i32 coordinates:
    OctreeI32,
    NodeKey, VisitCommand, NodePtr
};
use vox_writer::VoxWriter;

/// A basic boolean "voxel": `true` = voxel is filled, `false` = voxel is empty.
pub type Voxel = bool;

/// A simple container around an `OctreeI32<Voxel>`.
/// 
/// - `height` controls how many levels of detail the tree will have.
/// - We store `bool` at each node, so a `true` means "filled" and `false` means "empty."
#[derive(Clone)]
pub struct VoxelCSG {
    /// The underlying octree for storing voxels.
    pub tree: OctreeI32<Voxel>,
}

impl VoxelCSG {
    /// Create a new `VoxelCSG` with a desired `height`.
    /// 
    /// - `height` must be > 1 (the crate requirement).
    /// - At level 0 (leaf), each voxel is effectively one “cell” in 3D.
    /// - At level `height-1`, you have the topmost root(s).
    pub fn new(height: u32) -> Self {
        // Safety: We must guarantee that the shape used by OctreeI32 is correct,
        // but using `OctreeI32` is already safe by definition in grid-tree-rs.
        let tree = OctreeI32::new(height as u8);
        Self { tree }
    }
    
    /// A helper to query whether a single voxel coordinate is `true` or `false` in this CSG.
    /// Returns false if the node doesn't exist or is set to false.
    pub fn get_voxel(&self, coords: IVec3) -> bool {
        // We look for a leaf node at level 0.
        let key = NodeKey::new(0, coords);
        if let Some(child_relation) = self.tree.find_node(key) {
            if let Some(&val) = self.tree.get_value(child_relation.child) {
                return val;
            }
        }
        false
    }

    // -----------------------------------------------------------------------
    // 1) Cube
    // 
    //  Naive approach: iterate all points within [min, max) and fill them.
    //  This sets `tree` at leaf level = 0 for each coordinate in the range.
    // -----------------------------------------------------------------------
    pub fn fill_cube(&mut self, min: IVec3, max: IVec3) {
        // We'll fill all integer positions x in [min.x, max.x),
        // y in [min.y, max.y), z in [min.z, max.z).
        // For each voxel coordinate, we "turn on" that voxel in the octree.
        for z in min.z..max.z {
            for y in min.y..max.y {
                for x in min.x..max.x {
                    let coord = IVec3::new(x, y, z);
                    let leaf_key = NodeKey::new(0, coord);

                    // fill_path_to_node_from_root ensures all ancestor nodes exist
                    // and calls our closure at each step; we only need to set
                    // the occupant (leaf voxel) once:
                    self.tree.fill_path_to_node_from_root(leaf_key, |_, entry| {
                        // If vacant, allocate + set it to `true`
                        entry.or_insert_with(|| true);
                        VisitCommand::Continue
                    });
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // 2) Sphere
    // 
    //  We'll do a naive bounding-box iteration over the sphere’s bounding box,
    //  and if (x - cx)^2 + (y - cy)^2 + (z - cz)^2 <= r^2, we fill the voxel.
    // -----------------------------------------------------------------------
    pub fn fill_sphere(&mut self, center: IVec3, radius: f32) {
        let r_squared = radius * radius;
        // Rough integer bounding box around the sphere:
        let r_ceil = radius.ceil() as i32;
        let min = center - IVec3::new(r_ceil, r_ceil, r_ceil);
        let max = center + IVec3::new(r_ceil, r_ceil, r_ceil);

        for z in min.z..=max.z {
            for y in min.y..=max.y {
                for x in min.x..=max.x {
                    let p = IVec3::new(x, y, z);
                    let dist2 = (p - center).dot(p - center) as f32;
                    if dist2 <= r_squared {
                        let leaf_key = NodeKey::new(0, p);
                        self.tree.fill_path_to_node_from_root(leaf_key, |_, entry| {
                            entry.or_insert_with(|| true);
                            VisitCommand::Continue
                        });
                    }
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // 3) Cylinder
    // 
    //  We'll define a vertical cylinder aligned with, say, the Z axis.
    //  radius in X-Y plane, and `height` along Z. 
    //  bounding box is from z = base_z to z = base_z + height.
    // -----------------------------------------------------------------------
    pub fn fill_cylinder(
        &mut self,
        base_center_xy: IVec3, // (x, y, z_base)
        height: i32,
        radius: f32
    ) {
        let r_squared = radius * radius;
        let top_z = base_center_xy.z + height;

        // bounding box in X-Y around that circle:
        let r_ceil = radius.ceil() as i32;
        let min_xy = base_center_xy.xy() - IVec3::new(r_ceil, r_ceil, 0).xy();
        let max_xy = base_center_xy.xy() + IVec3::new(r_ceil, r_ceil, 0).xy();

        for z in base_center_xy.z..top_z {
            for y in min_xy.y..=max_xy.y {
                for x in min_xy.x..=max_xy.x {
                    // Check distance from center in XY:
                    let dx = x - base_center_xy.x;
                    let dy = y - base_center_xy.y;
                    let dist2 = (dx*dx + dy*dy) as f32;
                    if dist2 <= r_squared {
                        let p = IVec3::new(x, y, z);
                        let leaf_key = NodeKey::new(0, p);
                        self.tree.fill_path_to_node_from_root(leaf_key, |_, entry| {
                            entry.or_insert_with(|| true);
                            VisitCommand::Continue
                        });
                    }
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // 4) Polyhedron (naive approach)
    //
    //   For a convex polyhedron, we can test each point by a "point in polyhedron"
    //   function (e.g., checking half-spaces from each face).
    //   We'll just show the general structure: you’d implement your own `point_in_polyhedron`
    //   test. This is left to you to define, but here’s a placeholder:
    // -----------------------------------------------------------------------
    pub fn fill_polyhedron(
        &mut self,
        poly_min: IVec3,
        poly_max: IVec3,
        vertices: &[IVec3],
        indices: &[(usize, usize, usize)],
    ) {
        // A real "point in polyhedron" test for arbitrary polygons can be done via
        // "winding number" or "half-space intersection" (for convex shapes). We'll just
        // provide a stub function here:
        fn point_in_polyhedron(p: IVec3, verts: &[IVec3], inds: &[(usize, usize, usize)]) -> bool {
            // Insert your actual math test here!
            // For demonstration, we pretend everything is inside if x+y+z is even.
            (p.x + p.y + p.z) % 2 == 0
        }

        for z in poly_min.z..=poly_max.z {
            for y in poly_min.y..=poly_max.y {
                for x in poly_min.x..=poly_max.x {
                    let p = IVec3::new(x, y, z);
                    if point_in_polyhedron(p, vertices, indices) {
                        let leaf_key = NodeKey::new(0, p);
                        self.tree.fill_path_to_node_from_root(leaf_key, |_, entry| {
                            entry.or_insert_with(|| true);
                            VisitCommand::Continue
                        });
                    }
                }
            }
        }
    }
    
    // -----------------------------------------------------
    // 1) UNION
    //
    //  result[x,y,z] = self[x,y,z] OR other[x,y,z]
    //
    //  Approach:
    //    - Create a new VoxelCSG with a height that can hold both shapes.
    //    - Copy over all "true" leaf voxels from both shapes.
    // -----------------------------------------------------
    pub fn union(&self, other: &Self) -> Self {
        // The new tree must be at least as tall as the taller of the two.
        let new_height = self.tree.height().max(other.tree.height());
        let mut result = VoxelCSG::new(new_height as u32);

        // Helper function to copy all "true" leaves from `src` into `result`.
        let mut copy_true_leaves = |src: &VoxelCSG| {
            for (root_key, root_node) in src.tree.iter_roots() {
                let root_ptr = NodePtr::new(root_key.level, root_node.self_ptr);
                src.tree.visit_tree_depth_first(root_ptr, root_key.coordinates, 0, |ptr, coords| {
                    // Only care about leaf-level 0 that are "true".
                    if ptr.level() == 0 {
                        if let Some(&voxel_filled) = src.tree.get_value(ptr) {
                            if voxel_filled {
                                // Set in `result`.
                                result.tree.fill_path_to_node_from_root(NodeKey::new(0, coords), |_, entry| {
                                    entry.or_insert_with(|| true);
                                    VisitCommand::Continue
                                });
                            }
                        }
                    }
                    VisitCommand::Continue
                });
            }
        };

        // Copy from both shapes.
        copy_true_leaves(self);
        copy_true_leaves(other);

        result
    }
    
    // -----------------------------------------------------
    // 2) INTERSECTION
    //
    //  result[x,y,z] = self[x,y,z] AND other[x,y,z]
    //
    //  Approach:
    //    - For each leaf voxel in "self" that is "true",
    //      check if it is also "true" in "other".
    //    - If yes, set the result to "true" at that voxel.
    // -----------------------------------------------------
    pub fn intersection(&self, other: &Self) -> Self {
        let new_height = self.tree.height().max(other.tree.height());
        let mut result = VoxelCSG::new(new_height as u32);

        // For each leaf-level voxel in `self` that is "true", check `other`.
        for (root_key, root_node) in self.tree.iter_roots() {
            let root_ptr = NodePtr::new(root_key.level, root_node.self_ptr);
            self.tree.visit_tree_depth_first(root_ptr, root_key.coordinates, 0, |ptr, coords| {
                if ptr.level() == 0 {
                    if let Some(&val_self) = self.tree.get_value(ptr) {
                        if val_self {
                            // Check other
                            if other.get_voxel(coords) {
                                // Both are true => set in result
                                result.tree.fill_path_to_node_from_root(NodeKey::new(0, coords), |_, entry| {
                                    entry.or_insert_with(|| true);
                                    VisitCommand::Continue
                                });
                            }
                        }
                    }
                }
                VisitCommand::Continue
            });
        }

        result
    }

    // -----------------------------------------------------
    // 3) DIFFERENCE
    //
    //  result[x,y,z] = self[x,y,z] AND (NOT other[x,y,z])
    //
    //  Approach:
    //    - For each leaf voxel in "self" that is "true",
    //      check if "other" is "false" at that coordinate.
    //    - If "false" in other, keep it in result (set "true").
    // -----------------------------------------------------
    pub fn difference(&self, other: &Self) -> Self {
        let new_height = self.tree.height().max(other.tree.height());
        let mut result = VoxelCSG::new(new_height as u32);

        // For each leaf-level voxel in `self` that is "true",
        // only copy if `other` is false at that coordinate.
        for (root_key, root_node) in self.tree.iter_roots() {
            let root_ptr = NodePtr::new(root_key.level, root_node.self_ptr);
            self.tree.visit_tree_depth_first(root_ptr, root_key.coordinates, 0, |ptr, coords| {
                if ptr.level() == 0 {
                    if let Some(&val_self) = self.tree.get_value(ptr) {
                        if val_self {
                            // Keep if other is false
                            if !other.get_voxel(coords) {
                                result.tree.fill_path_to_node_from_root(NodeKey::new(0, coords), |_, entry| {
                                    entry.or_insert_with(|| true);
                                    VisitCommand::Continue
                                });
                            }
                        }
                    }
                }
                VisitCommand::Continue
            });
        }

        result
    }

    // -----------------------------------------------------
    // 4) INVERSE (bit-flip of existing nodes)
    //
    //   We flip "true" to "false" and "false" to "true" in
    //   the *already-stored* nodes. We do *not* add new
    //   nodes for previously non-existent coordinates, so
    //   this is NOT a true infinite complement.
    //
    //   You can do it "in-place" or produce a new shape.
    //   Below is an in-place toggle example.
    // -----------------------------------------------------
    pub fn invert_in_place(&mut self) {
        // We'll collect all nodes via DFS and flip them.
        // Because we're mutably editing the tree, we should
        // collect node pointers first, then flip them.

        let mut all_nodes = Vec::new();
        // Gather everything at every level:
        for (root_key, root_node) in self.tree.iter_roots() {
            let root_ptr = NodePtr::new(root_key.level, root_node.self_ptr);
            let mut stack = vec![(root_ptr, root_key.coordinates)];
            while let Some((ptr, coords)) = stack.pop() {
                all_nodes.push(ptr); // record the pointer
                if ptr.level() > 0 {
                    self.tree
                        .visit_children_with_coordinates(ptr, coords, |cptr, ccoords| {
                            stack.push((cptr, ccoords));
                        });
                }
            }
        }

        // Now flip each one.
        // If you want to prune out "false" nodes, you can do so,
        // but here we just flip the boolean stored in-place.
        for ptr in all_nodes {
            if let Some(value) = self.tree.get_value_mut(ptr) {
                *value = !*value;
            }
        }
    }

    // Alternatively, produce a new shape that is a toggle of the old:
    pub fn invert(&self) -> Self {
        let mut result = self.clone();
        result.invert_in_place();
        result
    }
    
    /// Saves all filled voxels (leaf level == 0) as a MagicaVoxel .vox file.
    ///
    /// By default, each voxel is assigned color 255 (white).
    /// Note that MagicaVoxel has an internal 3D grid that does *not* natively support
    /// negative coordinates, so if your voxels include negative indices, you may need to
    /// offset them or clamp them in some way before writing.
    pub fn save_to_magicavoxel(&self, path: &str) -> std::io::Result<()> {
        let mut vox = VoxWriter::create_empty();

        // We iterate over every root in the tree, then do a depth-first traversal
        // down to level 0. Each leaf that is 'true' gets written out as a colored voxel.
        for (root_key, root_node) in self.tree.iter_roots() {
            let root_ptr = NodePtr::new(root_key.level, root_node.self_ptr);

            // Depth-first visit from each root down to level=0 leaves:
            self.tree.visit_tree_depth_first(
                root_ptr, 
                root_key.coordinates, 
                0, 
                |node_ptr, coords| {
                    // Only write out if we're at a leaf (level == 0) and it is filled:
                    if node_ptr.level() == 0 {
                        if let Some(&filled) = self.tree.get_value(node_ptr) {
                            if filled {
                                // Assign a simple color (e.g. 255 = white). 
                                // MagicaVoxel uses "z as up," but if you want to
                                // treat `coords.z` as up, you can directly do:
                                vox.add_voxel(coords.x, coords.y, coords.z, 255);
                            }
                        }
                    }
                    VisitCommand::Continue
                },
            );
        }

        // Finally, save the .vox file:
        vox.save_to_file(path.to_string())
    }
}

