#![no_std] // We're not using the Rust standard library
#![no_main] // We're not using main as the entry point for Rust program execution

use core::panic::PanicInfo;

// Panic handler
#[cfg(not(test))] // This line is used to disable rust-analyzer from winging duplicate panic definition as it is unable to see that we are not including std!
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
	loop {}
}

// Simple printing into VG whose buffer is located at address 0xb8000
static HELLO: &[u8] = b"Hello world!";

// Using the C naming convention output the "_start" function because 
// it is the default entry point function for most systems,
// but in this case it is what the LLVM linked looks for.
// It is a diverging function (i.e. does not ever return) because
// it runs continuously and invoked  by the bootloader, and 
// it only exits by shutting down the machine.
#[no_mangle]
pub extern "C" fn _start() -> ! {
	let vga_buffer = 0xb8000 as *mut u8;
	for (i, &byte) in HELLO.iter().enumerate() {
		unsafe {
			*vga_buffer.offset(i as isize * 2) = byte;
			*vga_buffer.offset(i as isize * 2 + 1) = 0xb;
		}
	}

	loop {}
}

// # Bare-bones compilation: `cargo rustc -- -C link-arg=-nostartfiles`
// # Kernel compilation: `cargo build --target x86_64-pucci.json`
// # Hello world compilation:
// - Enable rustc nightly, install llvm rustup components and the bootloader crate, then compile:
// 	```shell
//	rustup override set nightly
// 	rustup component add llvm-tools-preview
// 	cd ~; cargo install bootimage; cd -
// 	cargo bootimage --target x86_64-pucci.json
// 	```
// - Install and run on QEMU:
//	```shell
// 	sudo lscpu | grep Virtualization
// 	sudo modprobe kvm_intel nested=1
// 	cat /sys/module/kvm_intel/parameters/nested ### shoud say "Y"
// 	sudo pacman -S qemu virt-manager libvirt virt-viewer dnsmasq vde2 bridge-utils openbsd-netcat ebtables libguestfs
//	sudo systemctl enable --now libvirtd
//	sudo systemctl status libvirtd
// 	sudo nano /etc/libvirt/libvirtd.conf ### Uncomment 'unix_sock_group = "libvirt"' and 'unix_sock_rw_perms = "0770"'
// 	sudo usermod -a -G libvirt username
// 	sudo systemctl restart libvirtd
//	qemu-system-x86_64 -drive format=raw,file=target/x86_64-pucci/debug/bootimage-pucci.bin
// - Allow automatic QEMU  run when using cargo run:
//	+ Add the folling into `./cargo.config.toml`:
//		```toml
//		[target.'cfg(target_os = "none")']
//		runner = "bootimage runner"
//		```
//	+ Run: `cargo run --target x86_64-pucci.json`
//	````