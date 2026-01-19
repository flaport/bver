mod finders;
mod loader;
mod schema;

use finders::find_project_root;
use loader::load_config;

fn main() {
    println!("project_root: {:?}", find_project_root());
    println!("config: {:?}", load_config());
}
