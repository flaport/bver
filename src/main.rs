mod finders;

use finders::find_project_root;

fn main() {
    println!("{:?}", find_project_root());
}
