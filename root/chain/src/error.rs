//! Initialization errors.

use error_chain::*;

error_chain! {
	foreign_links {
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
