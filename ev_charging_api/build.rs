// build.rs

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 💡 tonic_build ではなく、新設された tonic_prost_build を使います！
    tonic_prost_build::compile_protos("proto/charging.proto")?;

    // protoファイルに変更があった時だけ再コンパイルするトリガー
    println!("cargo:rerun-if-changed=proto/charging.proto");

    Ok(())
}
