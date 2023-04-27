use vulkano::VulkanLibrary;
use vulkano::instance::{Instance, InstanceCreateInfo};


fn main() {
	
	let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");
	let instance = Instance::new(library, InstanceCreateInfo::default())
		.expect("failed to create instance");
	let physical_device = instance
		.enumerate_physical_devices()
		.expect("could not enumerate devices")
		.next()
		.expect("no devices available");
	println!("ham")
	for family in physical_device.queue_family_properties() {
		println!("Found a queue family with {:?} queue(s)", family.queue_count);
	}


    /*println!("Hello, wordle!");
	println!("wow power");
	let mut loop_val: isize = 1;
	while loop_val <= 100 {
		print!("{loop_val}");
		loop_val *= 101;
		loop_val = loop_val % 13;
	}
	println!("build me")*/
	println!("gamed");
	println!("gamed2");
	println!("hamis");
}
