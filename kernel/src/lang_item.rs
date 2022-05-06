use crate::sbi::shutdown;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    if let Some(location) = _info.location() {
        println!(
            "Paniced at {}:{} {}",
            location.file(),
            location.line(),
            _info.message().unwrap()
        );
    } else {
        println!("Paniced {}", _info.message().unwrap());
    }
    shutdown();
}
