fn main() {
    println!("Hello, wordle!");
	println!("wow power");
	let mut loop_val: isize = 1;
	while loop_val <= 100 {
		print!("{loop_val}");
		loop_val *= 101;
		loop_val = loop_val % 13;
	}
	println!("build me")
}
