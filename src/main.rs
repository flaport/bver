mod finders;
mod loader;
mod schema;

use finders::find_project_root;

fn main() {
    println!("{:?}", find_project_root());
}
