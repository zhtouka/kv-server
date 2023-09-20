
fn main() {
    prost_build::Config::default()
        .out_dir("./src/pb")
        .compile_protos(&["./abi.proto"], &["."])
        .unwrap();
}