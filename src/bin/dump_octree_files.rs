// Copyright 2016 Google Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use point_viewer::octree::{self, NodeId, OctreeDataProvider, OnDiskOctreeDataProvider};
use point_viewer::proto;
use protobuf::Message;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "upgrade_octree")]
struct CommandlineArguments {
    /// Directory of octree to upgrade.
    #[structopt(parse(from_os_str))]
    directory: PathBuf,
}

fn main() {
	let args = CommandlineArguments::from_args();
	let data_provider = OnDiskOctreeDataProvider {
directory: args.directory.clone(),
	};

	let meta = data_provider
		.meta_proto()
		.expect("Could not read meta proto.");

	println!("meta.pb");
	for node_proto in meta.nodes.iter() {
		let id = node_proto.id.as_ref().unwrap();
		let node_id = NodeId::from_proto(id);
		println!("{}.xyz", node_id);
		println!("{}.rgb", node_id);
		println!("{}.intensity", node_id);
	}
}
