use hocon::HoconLoader;

fn main() -> anyhow::Result<()> {
    let str_config = r"
    a:${ABC}
    ";
    let c = HoconLoader::new().load_str(str_config)?.hocon()?;
    println!("{:?}", c["a"].as_string());
    Ok(())
}
