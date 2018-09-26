extern crate sled;
extern crate point_viewer;
extern crate pbr;

use std::path::Path;
use std::fs::File;
use std::io::Read;
use point_viewer::octree::NodeId;

fn put_file<'a>(tree: &mut sled::Tree, key: &str, filename: &Path) {
    let mut data = Vec::new();
    File::open(filename).unwrap().read_to_end(&mut data).unwrap();
    tree.set(key.as_bytes().to_vec(), data).unwrap();
}

pub fn main() {
    let directory = Path::new("/Users/sirver/Downloads/pointcloud_just_colors/");
    let meta = point_viewer::octree::read_old_meta_proto(directory).unwrap();

    const SIZE: usize = 10*1024*1024*1024;
    let config = sled::ConfigBuilder::new()
      .path("/Users/sirver/Downloads/pointcloud_just_colors.sled")
      .use_compression(false)
      .cache_capacity(SIZE)
      .zero_copy_storage(true)
      .build();
    let mut tree = sled::Tree::start(config).unwrap();

    put_file(&mut tree, "meta", &directory.join("meta.pb"));

    for node in pbr::PbIter::new(meta.nodes.iter()) {
        let node_id = NodeId::from_level_index(
            node.id.as_ref().unwrap().level as u8,
            node.id.as_ref().unwrap().index as usize,
        );
        let xyz = format!("{}.xyz", node_id) ;
        let rgb = format!("{}.rgb", node_id) ;
        let intensity = format!("{}.intensity", node_id) ;
        put_file(&mut tree, &xyz, &directory.join(&xyz));
        put_file(&mut tree, &rgb, &directory.join(&rgb));
        put_file(&mut tree, &intensity, &directory.join(&intensity));
        println!("#sirver node_id: {}", node_id);
    }
 // db.put(b"my key", b"my value");
 // match db.get(b"my key") {
    // Ok(Some(value)) => println!("retrieved value {}", value.to_utf8().unwrap()),
    // Ok(None) => println!("value not found"),
    // Err(e) => println!("operational problem encountered: {}", e),
 // }
 // db.delete(b"my key").unwrap();
    
}
