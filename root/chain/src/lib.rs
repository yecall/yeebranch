// Copyright (C) 2019 Yee Foundation.
//
// This file is part of YeeChain.
//
// YeeChain is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// YeeChain is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with YeeChain.  If not, see <https://www.gnu.org/licenses/>.


use std::path::PathBuf;
use substrate_service::{
	ChainSpec, RuntimeGenesis, FactoryFullConfiguration, Configuration, ServiceFactory,
	Roles, TaskExecutor, Arc, LightComponents, Components, Service, FactoryBlock,
	FullClient, LightClient
};
use names::{Generator, Name};
use yee_cli::{Factory, NodeConfig, get_initial_info, InitialInfo, FactoryBlockNumber};
use yee_bootnodes_router::BootnodesRouterConf;
use std::time;
use log::{info, warn};
use substrate_network::{SyncState};
use ansi_term::Colour;
use std::fmt;
use substrate_network::service::SyncProvider;
use substrate_network::multiaddr::Protocol;
use futures::stream::Stream;
use sr_primitives::traits::As;
use std::net::Ipv4Addr;
use std::iter;
use yee_sharding::{ShardingDigestItem, ScaleOutPhaseDigestItem};
use sr_primitives::traits::{DigestItemFor, ProvideRuntimeApi};
use substrate_client::ChainHead;
use yee_sharding_primitives::{ShardingAPI};
use yee_pow_primitives::YeePOWApi;

pub mod error;

const IMPL_NAME : &str = "yee-node";
const NODE_NAME_MAX_LENGTH: usize = 32;

pub struct Params {
	pub database_path : String, // branch chain database_path
	pub keystore_path: String, // branch chain keystore_path
	pub version_commit: &'static str,
	pub version_version: &'static str,
	pub trigger_exit: Arc<dyn yee_consensus::TriggerExit>,
	pub root_bootnodes_router_conf: Option<BootnodesRouterConf>,
	pub root_port: Option<u16>,
}

pub struct RootChain;

impl RootChain {

	pub fn new(params: Params, executor: &TaskExecutor) -> error::Result<()> {

		let config = create_config::<Factory>(&params)?;

		let service = &*LightComponents::<Factory>::new(config, executor.clone()).map_err(|e|format!("Start root chain failed: {:?}", e))?;

		monitor_network(&service, &executor);

		Ok(())
	}

}

fn monitor_network<C: Components>(service: &Service<C>, executor: &TaskExecutor) {

	let network = service.network();
	let client = service.client();
	let txpool = service.transaction_pool();

	let mut last_number = None;
	let mut last_update = time::Instant::now();

	let display_notifications = network.status().for_each(move |sync_status| {

		if let Ok(info) = client.info() {
			let best_number: u64 = info.chain.best_number.as_();
			let best_hash = info.chain.best_hash;
			let _num_peers = sync_status.num_peers;
			let speed = move || speed(best_number, last_number, last_update);
			last_update = time::Instant::now();
			let (status, target) = match (sync_status.sync.state, sync_status.sync.best_seen_block) {
				(SyncState::Idle, _) => ("Idle".into(), "".into()),
				(SyncState::Downloading, None) => (format!("Syncing{}", speed()), "".into()),
				(SyncState::Downloading, Some(n)) => (format!("Syncing{}", speed()), format!(", target=#{}", n)),
			};
			last_number = Some(best_number);
			let _txpool_status = txpool.status();
			let finalized_number: u64 = info.chain.finalized_number.as_();
			let bandwidth_download = network.average_download_per_sec();
			let bandwidth_upload = network.average_upload_per_sec();
			info!("root chain: {}{} ({} peers), best: #{} ({}), finalized #{} ({}), ⬇ {} ⬆ {}",
				Colour::White.bold().paint(&status),
				target,
				Colour::White.bold().paint(format!("{}", sync_status.num_peers)),
				Colour::White.paint(format!("{}", best_number)),
				best_hash,
				Colour::White.paint(format!("{}", finalized_number)),
				info.chain.finalized_hash,
				TransferRateFormat(bandwidth_download),
				TransferRateFormat(bandwidth_upload),
			);

		} else {
			warn!("Error getting best block information");
		}

		Ok(())
	});

	executor.spawn(display_notifications);
}

fn create_config<F>(params: &Params) -> error::Result<FactoryFullConfiguration<F>> where
	F: ServiceFactory<Configuration=NodeConfig<F>>,
	DigestItemFor<FactoryBlock<F>>: ShardingDigestItem<u16> + ScaleOutPhaseDigestItem<FactoryBlockNumber<F>, u16>,
	FullClient<F>: ProvideRuntimeApi + ChainHead<FactoryBlock<F>>,
	<FullClient<F> as ProvideRuntimeApi>::Api: ShardingAPI<FactoryBlock<F>> + YeePOWApi<FactoryBlock<F>>,
	LightClient<F>: ProvideRuntimeApi + ChainHead<FactoryBlock<F>>,
	<LightClient<F> as ProvideRuntimeApi>::Api: ShardingAPI<FactoryBlock<F>> + YeePOWApi<FactoryBlock<F>>,
{

	let shard_num = 0; //TODO

	let spec_path = params.database_path.clone() + "/../../../conf/root-chain-spec.json";

	let spec = load_spec(PathBuf::from(&spec_path))?;
	let mut config = Configuration::<NodeConfig<F>, _>::default_with_spec(spec.clone());

	config.impl_name = IMPL_NAME;
	config.impl_commit = params.version_commit;
	config.impl_version = params.version_version;

	config.name = generate_node_name();

	config.keystore_path = params.keystore_path.clone();

	config.database_path = params.database_path.clone().replace("chains", "root_chains");

	config.roles = Roles::LIGHT;

	config.custom.trigger_exit = Some(params.trigger_exit.clone());

	config.network.client_version = config.client_id();

	config.network.boot_nodes = params.root_bootnodes_router_conf.as_ref()
		.and_then(|x|x.shards.get(&format!("{}", shard_num)))
		.and_then(|x|Some(x.native.clone())).unwrap_or(vec![]);

	match params.root_port{
		Some(root_port) => {
			config.network.listen_addresses = vec![
				iter::once(Protocol::Ip4(Ipv4Addr::new(0, 0, 0, 0)))
					.chain(iter::once(Protocol::Tcp(root_port)))
					.collect()
			];
		},
		_ => (),
	}

	let InitialInfo{context, shard_num, shard_count, ..} = get_initial_info::<F>(&config, shard_num)?;

	config.custom.shard_num = shard_num;
	config.custom.shard_count = shard_count;
	config.custom.context = Some(context);

	Ok(config)

}

fn load_spec<G: RuntimeGenesis>(spec_path: PathBuf) -> error::Result<ChainSpec<G>> {

	ChainSpec::from_json_file(spec_path).map_err(|_|error::ErrorKind::LoadSpecFailed.into())

}

fn generate_node_name() -> String {
	let result = loop {
		let node_name = Generator::with_naming(Name::Numbered).next().unwrap();
		let count = node_name.chars().count();

		if count < NODE_NAME_MAX_LENGTH {
			break node_name
		}
	};

	result
}

fn speed(best_number: u64, last_number: Option<u64>, last_update: time::Instant) -> String {
	let since_last_millis = last_update.elapsed().as_secs() * 1000;
	let since_last_subsec_millis = last_update.elapsed().subsec_millis() as u64;
	let speed = match last_number {
		Some(num) => (best_number.saturating_sub(num) * 10_000 / (since_last_millis + since_last_subsec_millis)) as f64,
		None => 0.0
	};

	if speed < 1.0 {
		"".into()
	} else {
		format!(" {:4.1} bps", speed / 10.0)
	}
}

struct TransferRateFormat(u64);
impl fmt::Display for TransferRateFormat {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		// Special case 0.
		if self.0 == 0 {
			return write!(f, "0")
		}

		// Under 0.1 kiB, display plain bytes.
		if self.0 < 100 {
			return write!(f, "{} B/s", self.0)
		}

		// Under 1.0 MiB/sec, display the value in kiB/sec.
		if self.0 < 1024 * 1024 {
			return write!(f, "{:.1}kiB/s", self.0 as f64 / 1024.0)
		}

		write!(f, "{:.1}MiB/s", self.0 as f64 / (1024.0 * 1024.0))
	}
}
