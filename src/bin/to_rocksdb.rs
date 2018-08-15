extern crate lmdb;
extern crate point_viewer;
extern crate pbr;

use std::path::Path;
use std::fs::File;
use std::io::Read;
use point_viewer::octree::NodeId;
use lmdb::Transaction;

fn put_file<'a>(db: lmdb::Database, txn: &mut lmdb::RwTransaction<'a>, key: &str, filename: &Path) {
    let mut data = Vec::new();
    File::open(filename).unwrap().read_to_end(&mut data).unwrap();
    txn.put(db, &key.as_bytes(), &data, lmdb::WriteFlags::empty()).unwrap();
}

pub fn main() {
    let directory = Path::new("/Users/sirver/Downloads/pointcloud_just_colors/");
    let meta = point_viewer::octree::read_old_meta_proto(directory).unwrap();

    const SIZE: usize = 10*1024*1024*1024;
    let env = lmdb::Environment::new()
        .set_map_size(SIZE)
        .open(Path::new("/Users/sirver/Downloads/pointcloud_just_colors.rocksdb")).unwrap();
    let db = env.create_db(None, lmdb::DatabaseFlags::empty()).unwrap();

    let mut txn = env.begin_rw_txn().unwrap();
    put_file(db, &mut txn, "meta", &directory.join("meta.pb"));

    for node in pbr::PbIter::new(meta.nodes.iter()) {
        let node_id = NodeId::from_level_index(
            node.id.as_ref().unwrap().level as u8,
            node.id.as_ref().unwrap().index as usize,
        );
        let xyz = format!("{}.xyz", node_id) ;
        let rgb = format!("{}.rgb", node_id) ;
        let intensity = format!("{}.intensity", node_id) ;
        put_file(db, &mut txn, &xyz, &directory.join(&xyz));
        put_file(db, &mut txn, &rgb, &directory.join(&rgb));
        put_file(db, &mut txn, &intensity, &directory.join(&intensity));
        println!("#sirver node_id: {}", node_id);
    }
    txn.commit().unwrap();
 // db.put(b"my key", b"my value");
 // match db.get(b"my key") {
    // Ok(Some(value)) => println!("retrieved value {}", value.to_utf8().unwrap()),
    // Ok(None) => println!("value not found"),
    // Err(e) => println!("operational problem encountered: {}", e),
 // }
 // db.delete(b"my key").unwrap();
    
}
