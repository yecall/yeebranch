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

use {
    structopt::StructOpt,
    substrate_cli::{impl_augment_clap},
};
use log::{info, warn};
use substrate_service::{FactoryFullConfiguration, ServiceFactory};
use crate::error;
use crate::service::{NodeConfig};
use yee_bootnodes_router;
use yee_bootnodes_router::BootnodesRouterConf;
use substrate_cli::VersionInfo;

#[derive(Clone, Debug, Default, StructOpt)]
pub struct YeeCliConfig {

    /// Specify a list of root chain bootnodes-routers
    #[structopt(long = "root-bootnodes-routers", value_name = "URL")]
    pub root_bootnodes_routers: Vec<String>,

    /// Specify p2p protocol TCP port to participate to root chain
    #[structopt(long = "root-port", value_name = "PORT")]
    pub root_port: Option<u16>,

    /// Whether use dev params or not
    #[structopt(long = "dev-params")]
    pub dev_params: bool,

}

impl_augment_clap!(YeeCliConfig);

pub fn process_custom_args<F>(config: &mut FactoryFullConfiguration<F>, custom_args: &YeeCliConfig, version: &VersionInfo) -> error::Result<()>
where
    F: ServiceFactory<Configuration=NodeConfig>,
{

    let root_bootnodes_routers = custom_args.root_bootnodes_routers.clone();

    if root_bootnodes_routers.len() > 0{

        match get_bootnodes_router_conf(&root_bootnodes_routers){
            Ok(root_bootnodes_router_conf) => {
                config.custom.root_bootnodes_router_conf = Some(root_bootnodes_router_conf);
            },
            Err(_) => {
                warn!("Failed to get root bootnodes router conf: {:?}", root_bootnodes_routers);
            }
        }
    }

    config.custom.root_port = custom_args.root_port;
    config.custom.version_commit = version.commit;
    config.custom.version_version = version.version;

    info!("Custom params: ");
    info!("  root port: {:?}", config.custom.root_port);
    info!("  root bootnodes router conf: {:?}", config.custom.root_bootnodes_router_conf);
    Ok(())
}

fn get_bootnodes_router_conf(bootnodes_routers :&Vec<String>) -> error::Result<BootnodesRouterConf>{

    yee_bootnodes_router::client::call(|mut client|{
        let result = client.bootnodes().call().map_err(|e|format!("{:?}", e))?;
        Ok(result)
    }, bootnodes_routers).map_err(|e|format!("{:?}", e).into())

}
