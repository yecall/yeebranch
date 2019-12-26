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

//! setup default params for dev mode

use crate::error;
use substrate_service::{FactoryFullConfiguration, ServiceFactory};
use crate::service::NodeConfig;
use crate::custom_param::YeeCliConfig;
use log::info;

pub fn process_dev_param<F>(config: &mut FactoryFullConfiguration<F>, custom_args: &mut YeeCliConfig) -> error::Result<()>
    where F: ServiceFactory<Configuration=NodeConfig> {

    let chain_spec_id = config.chain_spec.id();

    if chain_spec_id == "dev" && custom_args.dev_params {

        let run_params = yee_branch_dev::get_run_params().map_err(|e| format!("{:?}", e))?;

        info!("Dev params: ");
        info!("  root port: {}", run_params.root_port);
        info!("  root bootnodes routers: {:?}", run_params.root_bootnodes_routers);
        info!("  params: {}", get_dev_params(run_params.root_port, &run_params.root_bootnodes_routers));

        custom_args.root_port = Some(run_params.root_port);
        custom_args.root_bootnodes_routers = run_params.root_bootnodes_routers;

    }

    Ok(())
}

fn get_dev_params(
    root_port: u16,
    root_bootnodes_routers: &Vec<String>
) -> String{
    let root_bootnodes_routers = root_bootnodes_routers.iter().map(|x| format!("--root-bootnodes-routers={}", x)).collect::<Vec<String>>().join("");
    let params = format!("--root-port={} {}", root_port, root_bootnodes_routers);
    params
}