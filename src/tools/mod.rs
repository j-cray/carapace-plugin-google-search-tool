//! Tool modules for the Google Search Carapace plugin.

pub mod search;

pub use search::{
    execute_search_with, parse_and_validate_args, tool_input_schema, TOOL_DESCRIPTION, TOOL_NAME,
};
