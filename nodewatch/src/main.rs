use sysinfo::System;

fn main() {
    let mut sys = System::new_all();

    sys.refresh_all();

    println!("Num of CPUs: {}", sys.cpus().len());
    for i in 1..=sys.cpus().len()-1{
        println!("{:?}", sys.cpus()[i]);
    }
    
    println!("Total memory: {} MB", sys.total_memory() / 1024);
    println!("Used memory: {} MB", sys.used_memory() / 1024);

    // Process info
    println!("Number of processes: {}", sys.processes().len());
    if let Some(process) = sys.process(sysinfo::get_current_pid().unwrap()) {
        println!("Current process name: {:?}", process.name());
        println!("Memory usage: {:?} KB", process.memory());
    }

}