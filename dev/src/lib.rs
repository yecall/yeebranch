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

mod error;

const ROOT_PORT : u16 = 30335;
const ROOT_BOOTNODES_ROUTER : &str = "http://127.0.0.1:50001";

pub struct RunParams{
    pub root_port: u16,
    pub root_bootnodes_routers: Vec<String>,
}

pub fn get_run_params() -> error::Result<RunParams>{

    let root_port = ROOT_PORT;
    let root_bootnodes_routers = vec![ROOT_BOOTNODES_ROUTER.to_string()];

    Ok(RunParams{
        root_port,
        root_bootnodes_routers,
    })

}
