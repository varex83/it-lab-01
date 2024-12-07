use std::fs::File;
use std::io::BufWriter;
use serde::Serialize;

pub fn save_to_file<T>(data: &T, path: &str) -> Result<(), anyhow::Error>
where
    T: Serialize,
{
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    let mut ser = serde_json::Serializer::new(writer);
    data.serialize(&mut ser)?;
    Ok(())
}

pub fn load_from_file<T>(path: &str) -> Result<T, anyhow::Error>
where
    T: serde::de::DeserializeOwned,
{
    let file = File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let data = serde_json::from_reader(reader)?;
    Ok(data)
}
