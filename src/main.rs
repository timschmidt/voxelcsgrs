use grid_tree::glam::IVec3;
use voxelcsgrs::VoxelCSG;

mod tests;

// Import your voxel CSG module here.
// Suppose you have `mod voxel_csg;` where `VoxelCSG` is defined.
// Or if it's a separate crate, use the appropriate `use` statement.
//
// use your_crate::voxel_csg::VoxelCSG;

fn main() {
    // 1) Construct some shapes.

    // Shape A: a cube and a sphere overlapping.
    let mut shape1 = VoxelCSG::new(5);
    shape1.fill_cube(IVec3::new(0, 0, 0), IVec3::new(10, 10, 10));
    shape1.fill_sphere(IVec3::new(8, 8, 8), 5.0);
    shape1.save_to_magicavoxel("shape1.vox").expect("Failed to save shape1.vox");

    // Shape B: a cylinder plus a smaller cube inside it.
    let mut shape2 = VoxelCSG::new(5);
    shape2.fill_cylinder(IVec3::new(0, 0, 0), 8, 4.0);
    shape2.fill_cube(IVec3::new(2, 2, 2), IVec3::new(6, 6, 6));
    shape2.save_to_magicavoxel("shape2.vox").expect("Failed to save shape2.vox");

    // Shape C: a "polyhedron" example (with a dummy inside test).
    // If you have a real polyhedron & "point_in_polyhedron" test, replace it accordingly.
    let mut shape_poly = VoxelCSG::new(5);
    let vertices = [
        IVec3::new(0, 0, 0),
        IVec3::new(10, 0, 0),
        IVec3::new(0, 10, 0),
        IVec3::new(0, 0, 10),
    ];
    // Simple tetrahedron indices
    let indices = [
        (0, 1, 2),
        (0, 1, 3),
        (0, 2, 3),
        (1, 2, 3),
    ];
    shape_poly.fill_polyhedron(IVec3::new(0, 0, 0), IVec3::new(10, 10, 10), &vertices, &indices);
    shape_poly.save_to_magicavoxel("shape_poly.vox").expect("Failed to save shape_poly.vox");

    // 2) Perform Boolean CSG operations between shape1 and shape2.

    // 2a) Union
    let union_shape = shape1.union(&shape2);
    union_shape.save_to_magicavoxel("union.vox").expect("Failed to save union.vox");

    // 2b) Intersection
    let intersection_shape = shape1.intersection(&shape2);
    intersection_shape.save_to_magicavoxel("intersection.vox").expect("Failed to save intersection.vox");

    // 2c) Difference (shape1 minus shape2)
    let difference_shape = shape1.difference(&shape2);
    difference_shape.save_to_magicavoxel("difference.vox").expect("Failed to save difference.vox");

    // 3) Inversion of shape1 (in-place vs. producing a new shape).

    // Invert in-place:
    let mut shape1_inverted = shape1.clone();
    shape1_inverted.invert_in_place();
    shape1_inverted.save_to_magicavoxel("shape1_inverted.vox").expect("Failed to save shape1_inverted.vox");

    // Or produce a new shape that is the inverted copy:
    let shape1_invert_copy = shape1.invert();
    shape1_invert_copy.save_to_magicavoxel("shape1_invert_copy.vox").expect("Failed to save shape1_invert_copy.vox");

    println!("All shapes exported as .vox files!");
}
