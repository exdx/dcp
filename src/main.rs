fn main() {
    if let Err(e) = dcp::get_args().and_then(dcp::run) {
        eprintln!("{}", e);
        std::process::exit(1)
    }
}
