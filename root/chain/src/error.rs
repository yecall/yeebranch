//! Initialization errors.

use error_chain::*;

error_chain! {
	links {
		Cli(substrate_cli::error::Error, substrate_cli::error::ErrorKind) #[doc="Cli error"];
	}
	errors {
		/// Not implemented yet
		Unimplemented {
			description("not yet implemented"),
			display("Method Not Implemented"),
		}
		LoadSpecFailed {
			description("load spec failed"),
			display("Load spec failed"),
		}
	}

}
