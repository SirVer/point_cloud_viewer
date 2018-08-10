// NOCOM(#sirver): remove
#![allow(unused_imports)]

extern crate prost;
extern crate futures;
#[macro_use]
extern crate prost_derive;
extern crate tokio_core;
extern crate tower_h2;
extern crate tower_grpc;
extern crate point_viewer;
extern crate cgmath;
#[macro_use]
extern crate clap;
extern crate collision;

use futures::sync::mpsc;
use std::path::PathBuf;
use futures::{future, stream, Future, Stream, Sink};
use point_viewer::octree::{read_meta_proto, NodeId, Octree, OnDiskOctree, PositionEncoding};
use point_viewer::InternalIterator;
use std::sync::Arc;
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;
use tower_grpc::{Request, Response, Streaming};
use tower_h2::Server;
use cgmath::Point3;
use collision::Aabb3;

pub mod point_viewer_prost {
    pub mod proto {
        include!(concat!(env!("OUT_DIR"), "/point_viewer.proto.rs"));
    }

    pub mod grpc {
        pub mod proto {
            include!(concat!(env!("OUT_DIR"), "/point_viewer.grpc.proto.rs"));
        }
    }
}

use point_viewer_prost::grpc::proto::{server, GetMetaRequest, GetMetaReply, GetNodeDataRequest, GetNodeDataReply, GetPointsInBoxRequest, GetPointsInBoxReply};


#[derive(Debug, Clone)]
struct OctreeService {
    state: Arc<State>,
}

#[derive(Debug)]
struct State {
    octree: OnDiskOctree,
    meta: point_viewer_prost::proto::Meta,
}

impl point_viewer_prost::grpc::proto::server::Octree for OctreeService {
     type GetMetaFuture = future::FutureResult<Response<GetMetaReply>, tower_grpc::Error>;
        fn get_meta(&mut self, request: Request<GetMetaRequest>) -> Self::GetMetaFuture {
            println!("GetMeta = {:?}", request);
            let response = Response::new(GetMetaReply {
                meta: Some(self.state.meta.clone()),
            });
            future::ok(response)
        }

     type GetNodeDataFuture = future::FutureResult<Response<GetNodeDataReply>, tower_grpc::Error>;
     fn get_node_data(&mut self, request: Request<GetNodeDataRequest>) -> Self::GetNodeDataFuture {
         println!("GetNodeData = {:?}", request);
         let node_id = NodeId::from_str(&request.get_ref().id);
         let data = self.state.octree
             .get_node_data(&node_id)
             .unwrap();
         let mut node_proto = point_viewer::proto::Node::new();
         node_proto.set_position_encoding(data.meta.position_encoding.to_proto());
         node_proto.set_num_points(data.meta.num_points);
         node_proto.mut_id().set_level(node_id.level() as i32);
         node_proto.mut_id().set_index(node_id.index() as i64);
         let response = Response::new(GetNodeDataReply {
             node: Some(to_prost(&node_proto)),
             position: data.position,
             color: data.color
         });
         future::ok(response)
     }
    
     type GetPointsInBoxStream = Box<Stream<Item = GetPointsInBoxReply, Error = tower_grpc::Error>>;
      type GetPointsInBoxFuture = future::FutureResult<Response<Self::GetPointsInBoxStream>, tower_grpc::Error>;
      fn get_points_in_box(&mut self, request: Request<GetPointsInBoxRequest>) -> Self::GetPointsInBoxFuture {
          use std::thread;

        println!("GetPointsInBox = {:?}", request);

        let (tx, rx) = mpsc::channel(4);

        let state = self.state.clone();

        thread::spawn(move || {
            let mut tx = tx.wait();

            let bounding_box = {
                let bounding_box = request.get_ref().bounding_box.clone().unwrap();
                let min = bounding_box.min.unwrap();
                let max = bounding_box.max.unwrap();
                Aabb3::new(
                    Point3::new(min.x, min.y, min.z),
                    Point3::new(max.x, max.y, max.z),
                    )
            };
            let mut reply = point_viewer_prost::grpc::proto::GetPointsInBoxReply {
                points: Vec::new()
            };
            const NUM_POINTS: usize = 100000;

            // Proto message must be below 4 MB.
            state.octree.points_in_box(&bounding_box).for_each(|p| {
                reply.points.push(point_viewer_prost::proto::Vector3f {
                    x: p.position.x,
                    y: p.position.y,
                    z: p.position.z
                });
                if reply.points.len() >= NUM_POINTS {
                    println!("  => send {} points", reply.points.len());
                    tx.send(reply.clone()).unwrap();
                    reply.points.clear();
                }
            });
        });

        // NOCOM(#sirver): foo
        let rx = rx.map_err(|_| unimplemented!());
        future::ok(Response::new(Box::new(rx)))
      }
}

fn to_prost(node: &point_viewer::proto::Node) -> point_viewer_prost::proto::Node {
    point_viewer_prost::proto::Node {
        position_encoding: match node.position_encoding {
            // TODO(sirver): Is there no automatic way for this?
            point_viewer::proto::Node_PositionEncoding::INVALID => 0,
            point_viewer::proto::Node_PositionEncoding::Uint8 => 1,
            point_viewer::proto::Node_PositionEncoding::Uint16 => 2,
            point_viewer::proto::Node_PositionEncoding::Float32 => 3,
        },
        num_points: node.num_points,
        id: Some(point_viewer_prost::proto::NodeId {
            level: node.id.as_ref().unwrap().level,
            index: node.id.as_ref().unwrap().index,
        }),
    }
}

fn main() {
    let matches = clap::App::new("octree_server_tower_grpc")
        .args(&[
            clap::Arg::with_name("port")
                .help("Port to listen on for connections. [50051]")
                .long("port")
                .takes_value(true),
            clap::Arg::with_name("octree_directory")
                .help("Input directory of the octree directory to serve.")
                .index(1)
                .required(true),
        ])
        .get_matches();

    let port = value_t!(matches, "port", u16).unwrap_or(50051);
    let octree_directory = PathBuf::from(matches.value_of("octree_directory").unwrap());

    let mut core = Core::new().unwrap();
    let reactor = core.handle();

    let meta = {
        // TODO(sirver): Converting from protobuf to prost.
        let other = read_meta_proto(&octree_directory).unwrap();
        point_viewer_prost::proto::Meta {
            version: other.version,
            bounding_box: Some(point_viewer_prost::proto::AxisAlignedCuboid {
                min: Some(point_viewer_prost::proto::Vector3f {
                    x: other.bounding_box.as_ref().unwrap().min.as_ref().unwrap().x,
                    y: other.bounding_box.as_ref().unwrap().min.as_ref().unwrap().y,
                    z: other.bounding_box.as_ref().unwrap().min.as_ref().unwrap().z,
                }),
                max: Some(point_viewer_prost::proto::Vector3f {
                    x: other.bounding_box.as_ref().unwrap().max.as_ref().unwrap().x,
                    y: other.bounding_box.as_ref().unwrap().max.as_ref().unwrap().y,
                    z: other.bounding_box.as_ref().unwrap().max.as_ref().unwrap().z,
                }),
            }),
            resolution: other.resolution,
            nodes: other.nodes.iter().map(|n| to_prost(n)).collect::<Vec<_>>(),
        }
    };
    let octree = OnDiskOctree::new(octree_directory).unwrap();
    let handler = OctreeService {
        state: Arc::new(State {
            meta, octree
        }),
    };

    let new_service = server::OctreeServer::new(handler);

    let h2 = Server::new(new_service, Default::default(), reactor.clone());

    let addr = format!("127.0.0.1:{}", port).parse().unwrap();
    let bind = TcpListener::bind(&addr, &reactor).expect("bind");

    println!("listining on {:?}", addr);
    let serve = bind.incoming()
        .fold((h2, reactor), |(h2, reactor), (sock, _)| {
            if let Err(e) = sock.set_nodelay(true) {
                return Err(e);
            }

            let serve = h2.serve(sock);
            reactor.spawn(serve.map_err(|e| panic!("h2 error: {:?}", e)));

            Ok((h2, reactor))
        });

    core.run(serve).unwrap();
}
