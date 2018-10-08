extern crate point_viewer;
extern crate pbr;

use std::path::Path;
use std::fs;
use point_viewer::octree::{OnDiskOctree, NodeIterator};
use point_viewer::{InternalIterator, FfiPoint, encode_points};
use point_viewer::math::Cube;

pub fn main() {
    let in_dir = Path::new("/Users/sirver/Downloads/pointcloud_just_colors/");
    let out_dir = Path::new("/Users/sirver/Downloads/pointcloud_just_colors.draco");
    let meta = point_viewer::octree::read_meta_proto(in_dir).unwrap();

    fs::create_dir(&out_dir).unwrap();
    fs::copy(in_dir.join("meta.pb"), out_dir.join("meta.pb")).unwrap();


    let mut output = vec![0u8; 200_000 * 8*4];
    let octree = OnDiskOctree::from_meta(meta, in_dir.to_path_buf()).unwrap();
    let root_cube = Cube::bounding(&octree.meta.bounding_box);
    for (node_id, _) in pbr::PbIter::new(octree.nodes.iter()) {
        let mut points = Vec::new(); 
        let mut clr_r = 0.;
        NodeIterator::from_disk(&octree.meta, node_id).unwrap().for_each(|p| {
            let clr = p.color;
            let ffi_p = FfiPoint  {
                position: [ p.position.x, p.position.y, p.position.z ],
                color: [ clr.red, clr.green, clr.blue, clr.alpha ],
                intensity: p.intensity.unwrap(),
            };
            clr_r += p.color.alpha as f32;
            points.push(ffi_p);
        });

        let bounding_cube = node_id.find_bounding_cube(&root_cube);
        let min_bits = (f64::from(bounding_cube.edge_length()) / octree.meta.resolution).log2() as u32 + 1;

        // NOCOM(#sirver): we guesstimate that the buffer is not getting larger.
        // println!("#sirver points.len(): {:#?}", points.len());
        let mut output_size = 0;
        unsafe {
        assert_eq!(0, encode_points(points.as_ptr(), points.len() as u32, min_bits, output.as_mut_ptr(), &mut output_size as *mut i32));
        // println!("#sirver points.len(): {:#?},output_size: {:#?}", points.len(), output_size);

        std::fs::write(out_dir.join(&format!("{}.data", node_id)), &output[0..output_size as usize]).unwrap();
        }

        // NOCOM(#sirver): next step is to decode and see if they agree.
    }
}
