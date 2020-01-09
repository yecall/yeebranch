//! Service and ServiceFactory implementation. Specialized wrapper over Substrate service.

#![warn(unused_extern_crates)]

use std::sync::Arc;
use log::info;
use transaction_pool::{self, txpool::{Pool as TransactionPool}};
use yee_branch_runtime::{self, GenesisConfig, opaque::Block, RuntimeApi};
use substrate_service::{
	FactoryFullConfiguration, LightComponents, FullComponents, FullBackend,
	FullClient, LightClient, LightBackend, FullExecutor, LightExecutor,
	TaskExecutor, DefaultRpcHandlerConstructor
};
use basic_authorship::ProposerFactory;
use consensus::{import_queue, start_aura, AuraImportQueue, SlotDuration, NothingExtra};
use substrate_client as client;
use primitives::{ed25519::Pair, Pair as PairT};
use inherents::InherentDataProviders;
use network::{construct_simple_protocol, DefaultIdentifySpecialization};
use substrate_executor::native_executor_instance;
use substrate_service::construct_service_factory;

pub use substrate_executor::NativeExecutor;
use yee_bootnodes_router::BootnodesRouterConf;
use yee_root_chain;
use crate::cli::{CliTriggerExit, CliSignal};
use substrate_cli::TriggerExit;

pub const IMPL_NAME : &str = "yee-branch-node";

// Our native executor instance.
native_executor_instance!(
	pub Executor,
	yee_branch_runtime::api::dispatch,
	yee_branch_runtime::native_version,
	include_bytes!("../runtime/wasm/target/wasm32-unknown-unknown/release/yee_branch_runtime_wasm.compact.wasm")
);

#[derive(Default)]
pub struct NodeConfig {
	inherent_data_providers: InherentDataProviders,
	pub root_bootnodes_router_conf: Option<BootnodesRouterConf>,
	pub root_port: Option<u16>,
	pub version_commit: &'static str,
	pub version_version: &'static str,
	pub trigger_exit: Option<Arc<dyn yee_consensus::TriggerExit>>,
}

impl yee_consensus::TriggerExit for CliTriggerExit<CliSignal>{
	fn trigger_restart(&self){
		self.trigger_exit(CliSignal::Restart);
	}

	fn trigger_stop(&self){
		self.trigger_exit(CliSignal::Stop);
	}
}

construct_simple_protocol! {
	/// Demo protocol attachment for substrate.
	pub struct NodeProtocol where Block = Block { }
}

construct_service_factory! {
	struct Factory {
		Block = Block,
		RuntimeApi = RuntimeApi,
		NetworkProtocol = NodeProtocol { |config| Ok(NodeProtocol::new()) },
		RuntimeDispatch = Executor,
		FullTransactionPoolApi = transaction_pool::ChainApi<client::Client<FullBackend<Self>, FullExecutor<Self>, Block, RuntimeApi>, Block>
			{ |config, client| Ok(TransactionPool::new(config, transaction_pool::ChainApi::new(client))) },
		LightTransactionPoolApi = transaction_pool::ChainApi<client::Client<LightBackend<Self>, LightExecutor<Self>, Block, RuntimeApi>, Block>
			{ |config, client| Ok(TransactionPool::new(config, transaction_pool::ChainApi::new(client))) },
		Genesis = GenesisConfig,
		Configuration = NodeConfig,
		FullService = FullComponents<Self>
			{ |config: FactoryFullConfiguration<Self>, executor: TaskExecutor|
				FullComponents::<Factory>::new(config, executor)
			},
		AuthoritySetup = {
			|service: Self::FullService, executor: TaskExecutor, key: Option<Arc<Pair>>| {
				if let Some(key) = key {
					info!("Using authority key {}", key.public());
					let proposer = Arc::new(ProposerFactory {
						client: service.client(),
						transaction_pool: service.transaction_pool(),
						inherents_pool: service.inherents_pool(),
					});
					let client = service.client();
					executor.clone().spawn(start_aura(
						SlotDuration::get_or_compute(&*client)?,
						key.clone(),
						client.clone(),
						client,
						proposer,
						service.network(),
						service.on_exit(),
						service.config.custom.inherent_data_providers.clone(),
						service.config.force_authoring,
					)?);
				}

				let root_chain_param = yee_root_chain::Params{
					database_path: service.config.database_path.clone(),
					keystore_path: service.config.keystore_path.clone(),
					version_commit: service.config.custom.version_commit,
					version_version: service.config.custom.version_version,
					trigger_exit: service.config.custom.trigger_exit.clone().expect("qed"),
					root_bootnodes_router_conf: service.config.custom.root_bootnodes_router_conf.clone(),
					root_port: service.config.custom.root_port,
				};
				let root_chain = yee_root_chain::RootChain::new(root_chain_param, &executor).map_err(|e|format!("{:?}", e))?;

				Ok(service)
			}
		},
		LightService = LightComponents<Self>
			{ |config, executor| <LightComponents<Factory>>::new(config, executor) },
		FullImportQueue = AuraImportQueue<
			Self::Block,
		>
			{ |config: &mut FactoryFullConfiguration<Self> , client: Arc<FullClient<Self>>| {
					import_queue::<_, _, _, Pair>(
						SlotDuration::get_or_compute(&*client)?,
						client.clone(),
						None,
						client,
						NothingExtra,
						config.custom.inherent_data_providers.clone(),
					).map_err(Into::into)
				}
			},
		LightImportQueue = AuraImportQueue<
			Self::Block,
		>
			{ |config: &mut FactoryFullConfiguration<Self>, client: Arc<LightClient<Self>>| {
					import_queue::<_, _, _, Pair>(
						SlotDuration::get_or_compute(&*client)?,
						client.clone(),
						None,
						client,
						NothingExtra,
						config.custom.inherent_data_providers.clone(),
					).map_err(Into::into)
				}
			},
		FullRpcHandlerConstructor = DefaultRpcHandlerConstructor,
		LightRpcHandlerConstructor = DefaultRpcHandlerConstructor,
		IdentifySpecialization = DefaultIdentifySpecialization { |config| Ok(DefaultIdentifySpecialization{}) },
	}
}
