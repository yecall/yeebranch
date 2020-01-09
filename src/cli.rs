use crate::service;
use futures::{future, Future, sync::oneshot};
use tokio::runtime::Runtime;
pub use substrate_cli::{VersionInfo, IntoExit, error};
use substrate_cli::{informant, parse_and_execute, NoCustom, TriggerExit};
use substrate_service::{ServiceFactory, Roles as ServiceRoles, Arc};
use crate::chain_spec;
use std::ops::Deref;
use log::info;
use parking_lot::Mutex;
use futures::sync::oneshot::Sender;
use signal_hook::{iterator::Signals, SIGUSR1, SIGINT, SIGTERM};
use std::thread;
use std::fmt::Debug;
use crate::custom_param::{YeeCliConfig, process_custom_args};
use crate::dev_param::process_dev_param;

/// Parse command line arguments into service configuration.
pub fn run<I, T, E>(args: I, exit: E, version: VersionInfo) -> error::Result<()> where
	I: IntoIterator<Item = T>,
	T: Into<std::ffi::OsString> + Clone,
	E: IntoExit<TriggerExit=CliTriggerExit<CliSignal>>,
{
	parse_and_execute::<service::Factory, NoCustom, YeeCliConfig, _, _, _, _, _>(
		load_spec, &version, service::IMPL_NAME, args, exit,
	 	|exit, mut custom_args, mut config| {
			info!("{}", version.name);
			info!("  version {}", config.full_version());
			info!("  by {}, 2017, 2018", version.author);
			info!("Chain specification: {}", config.chain_spec.name());
			info!("Node name: {}", config.name);
			info!("Roles: {:?}", config.roles);

		    process_dev_param::<service::Factory>(&mut config, &mut custom_args).map_err(|e| format!("{:?}", e))?;
		    process_custom_args::<service::Factory>(&mut config, &custom_args, &version).map_err(|e| format!("{:?}", e))?;

		    let (exit, trigger_exit) = exit.into_exit();

		    config.custom.trigger_exit = Some(Arc::new(trigger_exit));

		    let runtime = Runtime::new().map_err(|e| format!("{:?}", e))?;
			let executor = runtime.executor();
			match config.roles {
				ServiceRoles::LIGHT => run_until_exit::<_, _, E>(
					runtime,
				 	service::Factory::new_light(config, executor).map_err(|e| format!("{:?}", e))?,
					exit
				),
				_ => run_until_exit::<_, _, E>(
					runtime,
					service::Factory::new_full(config, executor).map_err(|e| format!("{:?}", e))?,
					exit
				),
			}.map_err(|e| format!("{:?}", e))
		}
	).map_err(Into::into).map(|_| ())
}

fn load_spec(id: &str) -> Result<Option<chain_spec::ChainSpec>, String> {
	Ok(match chain_spec::Alternative::from(id) {
		Some(spec) => Some(spec.load()?),
		None => None,
	})
}

fn run_until_exit<T, C, E>(
	mut runtime: Runtime,
	service: T,
	e: E::Exit,
) -> error::Result<()>
	where
		T: Deref<Target=substrate_service::Service<C>>,
		C: substrate_service::Components,
		E: IntoExit,
{
	let (exit_send, exit) = exit_future::signal();

	let executor = runtime.executor();
	informant::start(&service, exit.clone(), executor.clone());

	let _ = runtime.block_on(e);
	exit_send.fire();

	// we eagerly drop the service so that the internal exit future is fired,
	// but we need to keep holding a reference to the global telemetry guard
	let _telemetry = service.telemetry();
	drop(service);
	Ok(())
}

// handles ctrl-c
pub struct Exit;
impl IntoExit for Exit {
	type TriggerExit = CliTriggerExit<CliSignal>;
	type Exit = future::MapErr<oneshot::Receiver<<Self::TriggerExit as TriggerExit>::Signal>, fn(oneshot::Canceled) -> ()>;
	fn into_exit(self) -> (Self::Exit, Self::TriggerExit) {
		// can't use signal directly here because CtrlC takes only `Fn`.
		let (exit_send, exit) = oneshot::channel();

		let exit_send_cell = Arc::new(Mutex::new(Some(exit_send)));
		let exit_send_cell_clone = exit_send_cell.clone();

		let signals = Signals::new(&[SIGUSR1, SIGINT, SIGTERM]).unwrap();

		thread::spawn(move || {
			for sig in signals.forever() {
				info!("Received signal {:?}", sig);

				let signal = match sig{
					SIGUSR1 => CliSignal::Restart,
					_ => CliSignal::Stop,
				};

				if let Some(sender) = exit_send_cell.lock().take() {
					sender.send(signal).expect("Error sending exit signal");
				}
			}
		});

		(exit.map_err(drop), CliTriggerExit{sender: exit_send_cell_clone})
	}
}

#[derive(Debug)]
pub enum CliSignal{
	Stop,
	Restart,
}

pub struct CliTriggerExit<Signal>{
	sender: Arc<Mutex<Option<Sender<Signal>>>>,
}

impl<Signal> TriggerExit for CliTriggerExit<Signal>
	where Signal: Send + Debug + 'static {
	type Signal = Signal;
	fn trigger_exit(&self, signal: Self::Signal){

		if let Some(sender) = self.sender.lock().take() {
			sender.send(signal).expect("Error sending exit signal");
		}
	}
}
