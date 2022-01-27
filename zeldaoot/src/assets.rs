use log::debug;
use serde::Deserialize;
use std::io::BufReader;
use std::path::{Path, PathBuf};

pub fn deserialize_offset<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: num_traits::FromPrimitive + num_traits::Num,
{
    let s = String::deserialize(deserializer)?;

    match s.strip_prefix("0x") {
        Some(s) => T::from_str_radix(s, 16),
        None => T::from_str_radix(&s, 10),
    }
    .map_err(|e| serde::de::Error::custom("can't convert string to number: {}"))
}

#[derive(Debug, Deserialize)]
enum Format {
    #[serde(rename = "i8")]
    I8,
}

#[derive(Debug, Deserialize)]
struct Texture {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "OutName")]
    out_name: String,
    #[serde(rename = "Format")]
    format: Format,
    #[serde(rename = "Width")]
    width: usize,
    #[serde(rename = "Height")]
    height: usize,
    #[serde(rename = "Offset", deserialize_with = "deserialize_offset")]
    offset: usize,
}

#[derive(Debug, Deserialize)]
struct DisplayList {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Offset", deserialize_with = "deserialize_offset")]
    offset: usize,
}

#[derive(Debug, Deserialize)]
struct File {
    #[serde(rename = "Texture", default)]
    textures: Vec<Texture>,
    #[serde(rename = "DList", default)]
    dlists: Vec<DisplayList>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    #[serde(rename = "File")]
    files: Vec<File>,
}

fn load_texture(texture: &Texture) -> anyhow::Result<()> {
    Ok(())
}

pub fn load(filename: &str) -> anyhow::Result<()> {
    let basedir = Path::new("assets/xml");
    let path = basedir.join(filename);

    let f = std::fs::File::open(path)?;
    let reader = BufReader::new(f);
    let asset: Asset = quick_xml::de::from_reader(reader)?;

    for texture in &asset.files[0].textures {
        load_texture(texture)?;
        break;
    }
    debug!("{:#?}", asset);

    Ok(())
}
