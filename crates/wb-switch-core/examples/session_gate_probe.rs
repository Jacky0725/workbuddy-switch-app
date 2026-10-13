use wb_switch_core::modules::{session_safety, variant::WbVariant};

fn main() {
    for variant in [WbVariant::Cn, WbVariant::Ai] {
        match session_safety::require_verified_local_client(variant) {
            Ok(()) => println!("{}: compatible local-session profile", variant.as_str()),
            Err(error) => println!("{}: blocked ({error})", variant.as_str()),
        }
    }
}
