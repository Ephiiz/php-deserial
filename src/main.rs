use std::any::Any;

use php::parse_string;
use tracing::{info, Level};
use tracing_subscriber::{FmtSubscriber};


mod util;
mod php;
fn main() {
    let sub = FmtSubscriber::builder().with_max_level(Level::TRACE).finish();
    tracing::subscriber::set_global_default(sub).expect("setting default subscriber failed");
}
