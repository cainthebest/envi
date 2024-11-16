mod component;

use component::system::{System, SystemArgs};

fn main() {
    let sys = sysinfo::System::new_all();

    let args = SystemArgs {
        os: true,
        mem: true,
        cpu: true,
    };

    let system = System::new(args, &sys);

    println!("{:#?}", system);
}
