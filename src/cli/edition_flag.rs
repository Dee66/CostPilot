use crate::edition::{detect_edition, Edition};
use std::path::PathBuf;

pub fn print_edition_status(keys_dir: &PathBuf) {
    let ctx = detect_edition(keys_dir);

    match ctx.edition {
        Edition::Free => {
            println!("EDITION: Free");
        }
        Edition::Premium => {
            if let Some(expires) = ctx.expires_date() {
                println!("EDITION: Premium (valid until {})", expires);
            } else {
                println!("EDITION: Premium");
            }
        }
    }
}
