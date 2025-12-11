fn main() {
    println!("cargo:rerun-if-changed=pro_engine/pro_engine.wit");

    // TODO: Generate wit-bindgen bindings when ready
    // wit_bindgen_rust::generate!({
    //     path: "pro_engine/pro_engine.wit",
    //     world: "costpilot-proengine",
    // });
}
