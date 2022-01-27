mod table;
use table::*;

use anyhow::{anyhow, Context};
use convert_case::{Case, Casing};
use quote::quote;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::BufReader;
use std::io::Write;
use walkdir::WalkDir;
use enum_dispatch::enum_dispatch;
use proc_macro2::TokenStream;

pub fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    match s.as_str() {
        "On" => Ok(true),
        "Off" => Ok(false),
        _ => Err(serde::de::Error::custom("unsupported value for `Static`")),
    }
}

pub fn deserialize_number<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: num_traits::FromPrimitive + num_traits::Num,
{
    let s = String::deserialize(deserializer)?;

    match s.strip_prefix("0x") {
        Some(s) => T::from_str_radix(s, 16),
        None => T::from_str_radix(&s, 10),
    }
    .map_err(|_| serde::de::Error::custom(&format!("can't convert string to number: {}", s)))
}

pub fn deserialize_opt_number<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: num_traits::FromPrimitive + num_traits::Num,
{
    Ok(Some(deserialize_number(deserializer)?))
}

#[enum_dispatch(Item)]
trait Name {
    fn name(&self) -> &str;

/*
    fn ty(&self) -> TokenStream {
        quote!{()}
    }

    fn loadfn(&self) -> proc_macro2::Ident {
        "load_stub".to_ident()
    }
*/

    fn ty(&self) -> TokenStream {
        quote!{::std::sync::Arc<Vec<u8>>}
    }

    fn loadfn(&self) -> proc_macro2::Ident {
        "load_texture".to_ident()
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
enum Format {
    #[serde(rename = "rgba32")]
    Rgba32,
    #[serde(rename = "rgba16")]
    Rgba16,
    #[serde(rename = "i4")]
    I4,
    #[serde(rename = "i8")]
    I8,
    #[serde(rename = "ia4")]
    Ia4,
    #[serde(rename = "ia8")]
    Ia8,
    #[serde(rename = "ia16")]
    Ia16,
    #[serde(rename = "ci4")]
    Ci4,
    #[serde(rename = "ci8")]
    Ci8,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Texture {
    #[serde(rename = "Name")]
    name: String,
    #[serde(
        rename = "Offset",
        deserialize_with = "deserialize_opt_number",
        default
    )]
    offset: Option<usize>,
    #[serde(rename = "OutName")]
    out_name: Option<String>,
    #[serde(rename = "Format")]
    format: Format,
    #[serde(rename = "Width")]
    width: usize,
    #[serde(rename = "Height")]
    height: usize,
    #[serde(
        rename = "TlutOffset",
        deserialize_with = "deserialize_opt_number",
        default
    )]
    tlut_offset: Option<usize>,
    #[serde(rename = "Static", deserialize_with = "deserialize_bool", default)]
    is_static: bool,
}

impl Name for Texture {
    fn name(&self) -> &str {
        self.out_name.as_ref().unwrap_or(&self.name)
    }


    fn ty(&self) -> TokenStream {
        quote!{::std::sync::Arc<Vec<u8>>}
    }

    fn loadfn(&self) -> proc_macro2::Ident {
        "load_texture".to_ident()
    }

}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
enum Ucode {
    #[serde(rename = "f3dex")]
    F3dex,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DisplayList {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Offset", deserialize_with = "deserialize_number")]
    offset: usize,
    #[serde(rename = "Static", deserialize_with = "deserialize_bool", default)]
    is_static: bool,
    #[serde(rename = "Ucode", default)]
    ucode: Option<Ucode>,
}

impl Name for DisplayList {
    fn name(&self) -> &str {
        &self.name
    }

/*
    fn ty(&self) -> TokenStream {
        quote!{::std::sync::Arc<super::DisplayList>}
    }

    fn loadfn(&self) -> proc_macro2::Ident {
        "load_displaylist".to_ident()
    }
*/
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Scene {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Offset", deserialize_with = "deserialize_number")]
    offset: usize,
}

impl Name for Scene {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
enum HackMode {
    #[serde(rename = "syotes_room")]
    SyotesRoom,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Room {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Offset", deserialize_with = "deserialize_number")]
    offset: usize,
    #[serde(rename = "HackMode", default)]
    hackmode: Option<HackMode>,
}

impl Name for Room {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Cutscene {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Offset", deserialize_with = "deserialize_number")]
    offset: usize,
}

impl Name for Cutscene {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Blob {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Offset", deserialize_with = "deserialize_number")]
    offset: usize,
    #[serde(rename = "Size", deserialize_with = "deserialize_number")]
    size: usize,
}

impl Name for Blob {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Path {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Offset", deserialize_with = "deserialize_number")]
    offset: usize,
}

impl Name for Path {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
enum VectorType {
    #[serde(rename = "s16")]
    S16,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Vector {
    #[serde(rename = "Type")]
    vector_type: VectorType,
    #[serde(rename = "Dimensions", deserialize_with = "deserialize_number")]
    dimensions: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
enum ArrayElement {
    Vtx,
    Vector(Vector),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Array {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Offset", deserialize_with = "deserialize_number")]
    offset: usize,
    #[serde(rename = "Count", deserialize_with = "deserialize_number")]
    count: usize,
    #[serde(rename = "Static", deserialize_with = "deserialize_bool", default)]
    is_static: bool,

    #[serde(rename = "$value")]
    element: ArrayElement,
}

impl Name for Array {
    fn name(&self) -> &str {
        &self.name
    }
}

fn default_datatype() -> String {
    "void*".to_string()
}

fn default_typesize() -> usize {
    4
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Symbol {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Offset", deserialize_with = "deserialize_number")]
    offset: usize,
    #[serde(rename = "Type", default = "default_datatype")]
    datatype: String,
    #[serde(
        rename = "TypeSize",
        deserialize_with = "deserialize_number",
        default = "default_typesize"
    )]
    typesize: usize,
    #[serde(rename = "Count", deserialize_with = "deserialize_opt_number")]
    count: Option<usize>,
}

impl Name for Symbol {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Collision {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Offset", deserialize_with = "deserialize_number")]
    offset: usize,
}

impl Name for Collision {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Animation {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Offset", deserialize_with = "deserialize_number")]
    offset: usize,
}

impl Name for Animation {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
enum SkeletonType {
    Normal,
    Flex,
    Curve,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Skeleton {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Offset", deserialize_with = "deserialize_number")]
    offset: usize,
    #[serde(rename = "Type")]
    skeleton_type: SkeletonType,
    #[serde(rename = "LimbType")]
    limb_type: LimbType,
}

impl Name for Skeleton {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CurveAnimation {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Offset", deserialize_with = "deserialize_number")]
    offset: usize,
    #[serde(rename = "SkelOffset", deserialize_with = "deserialize_number")]
    skel_offset: usize,
}

impl Name for CurveAnimation {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Limb {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Offset", deserialize_with = "deserialize_number")]
    offset: usize,
    #[serde(rename = "LimbType")]
    limb_type: LimbType,
}

impl Name for Limb {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PlayerAnimationData {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Offset", deserialize_with = "deserialize_number")]
    offset: usize,
    #[serde(rename = "FrameCount", deserialize_with = "deserialize_number")]
    frame_count: usize,
}

impl Name for PlayerAnimationData {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Mtx {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Offset", deserialize_with = "deserialize_number")]
    offset: usize,
}

impl Name for Mtx {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyAnimation {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Offset", deserialize_with = "deserialize_number")]
    offset: usize,
}

impl Name for LegacyAnimation {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
enum LimbType {
    Standard,
    #[serde(rename = "LOD")]
    Lod,
    Skin,
    Curve,
    Legacy,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LimbTable {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Offset", deserialize_with = "deserialize_number")]
    offset: usize,
    #[serde(rename = "LimbType")]
    limb_type: LimbType,
    #[serde(rename = "Count", deserialize_with = "deserialize_number")]
    count: usize,
}

impl Name for LimbTable {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PlayerAnimation {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Offset", deserialize_with = "deserialize_number")]
    offset: usize,
}

impl Name for PlayerAnimation {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[enum_dispatch]
enum Item {
    Texture(Texture),
    #[serde(rename = "DList")]
    DisplayList(DisplayList),
    Scene(Scene),
    Room(Room),
    Cutscene(Cutscene),
    Blob(Blob),
    Path(Path),
    Array(Array),
    Symbol(Symbol),
    Collision(Collision),
    Animation(Animation),
    Skeleton(Skeleton),
    CurveAnimation(CurveAnimation),
    Limb(Limb),
    PlayerAnimationData(PlayerAnimationData),
    Mtx(Mtx),
    LegacyAnimation(LegacyAnimation),
    LimbTable(LimbTable),
    PlayerAnimation(PlayerAnimation),
}

fn default_range_end() -> u32 {
    u32::MAX
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "OutName", default)]
    out_name: Option<String>,
    #[serde(rename = "$value")]
    items: Vec<Item>,
    #[serde(
        rename = "Segment",
        deserialize_with = "deserialize_opt_number",
        default
    )]
    segment: Option<usize>,
    #[serde(
        rename = "BaseAddress",
        deserialize_with = "deserialize_number",
        default
    )]
    base_address: u32,
    #[serde(
        rename = "RangeStart",
        deserialize_with = "deserialize_number",
        default
    )]
    range_start: u32,
    #[serde(
        rename = "RangeEnd",
        deserialize_with = "deserialize_number",
        default = "default_range_end"
    )]
    range_end: u32,
}

impl File {
    pub fn name(&self) -> &str {
        self.out_name.as_ref().unwrap_or(&self.name)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExternalFile {
    #[serde(rename = "XmlPath")]
    xml_path: std::path::PathBuf,
    #[serde(rename = "OutPath")]
    out_path: std::path::PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Asset {
    #[serde(rename = "File")]
    files: Vec<File>,
    #[serde(rename = "ExternalFile", default)]
    external_files: Vec<ExternalFile>,
}

impl Asset {
    pub fn load<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        let f = std::fs::File::open(path).context("can't open")?;
        let reader = BufReader::new(f);
        let asset: Asset = quick_xml::de::from_reader(reader).context("can't parse")?;
        Ok(asset)
    }
}

trait ToIdent: AsRef<str> {
    fn to_ident(&self) -> proc_macro2::Ident {
        proc_macro2::Ident::new(self.as_ref(), proc_macro2::Span::call_site())
    }

    fn sanitize(&self) -> String {
        let mut s = self.as_ref();
        let mut first = s.chars().nth(0).unwrap();
        let mut second = s.chars().nth(1).unwrap();
        if first == 'g' && second.is_uppercase() {
            s = &s[1..];
            first = s.chars().nth(0).unwrap();
        }

        if first.is_ascii_digit() {
            let number = match first {
                '0' => "Zero",
                '1' => "One",
                '2' => "Two",
                '3' => "Three",
                '4' => "Four",
                '5' => "Five",
                '6' => "Six",
                '7' => "Seven",
                '8' => "Eight",
                '9' => "Nine",
                _ => unreachable!(),
            };

            format!("{}{}", number, &s[1..])
        } else {
            s.to_string()
        }
    }
}

impl<T: AsRef<str>> ToIdent for T {}

pub fn rustfmt_generated_string(source: &str) -> std::io::Result<std::borrow::Cow<str>> {
    let rustfmt = "rustfmt";
    let mut cmd = std::process::Command::new(&*rustfmt);

    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped());

    let mut child = cmd.spawn()?;
    let mut child_stdin = child.stdin.take().unwrap();
    let mut child_stdout = child.stdout.take().unwrap();

    let source = source.to_owned();

    // Write to stdin in a new thread, so that we can read from stdout on this
    // thread. This keeps the child from blocking on writing to its stdout which
    // might block us from writing to its stdin.
    let stdin_handle = ::std::thread::spawn(move || {
        let _ = child_stdin.write_all(source.as_bytes());
        source
    });

    let mut output = vec![];
    std::io::copy(&mut child_stdout, &mut output)?;

    let status = child.wait()?;
    let source = stdin_handle.join().expect(
        "The thread writing to rustfmt's stdin doesn't do \
             anything that could panic",
    );

    match String::from_utf8(output) {
        Ok(bindings) => match status.code() {
            Some(0) => Ok(std::borrow::Cow::Owned(bindings)),
            Some(2) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Rustfmt parsing errors.".to_string(),
            )),
            Some(3) => {
                eprintln!("Rustfmt could not format some lines.");
                Ok(std::borrow::Cow::Owned(bindings))
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Internal rustfmt error".to_string(),
            )),
        },
        _ => Ok(std::borrow::Cow::Owned(source)),
    }
}

fn main() -> anyhow::Result<()> {
    let out_dir: std::path::PathBuf = std::env::var("OUT_DIR").unwrap().into();
    let mut out = std::fs::File::create(out_dir.join("generated.rs")).unwrap();

    let mut files = HashMap::new();
    for entry in WalkDir::new("../assets/xml")
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let f_name = entry.file_name().to_string_lossy();
        if !f_name.ends_with(".xml") {
            continue;
        }

        let asset = Asset::load(entry.path())
            .with_context(|| anyhow!("can't load {}", entry.path().to_string_lossy()))?;

        for file in asset.files {
            if FILE_NAMES.iter().find(|&name| name == &file.name).is_none() {
                return Err(anyhow!("unknown baserom file: {}", file.name));
            }

            if let Some(old) = files.insert(file.name().to_string(), file) {
                return Err(anyhow!("duplicate file: {}", old.name()));
            }
        }
    }
    let files = files;

    let mut most:usize = 0;
    let mut file_structs = Vec::with_capacity(files.len());
    for (_, file) in &files {
        most = most.max(file.items.len());

        let name = file.name().to_case(Case::Pascal).to_ident();
        let fields = file.items.iter().map(|item| {
            let name = item.name().sanitize().to_case(Case::Snake).to_ident();
            let ty = item.ty();

            quote!{ pub #name: #ty, }
        });

        let code_getoffset = if let Some((i, _)) = FILE_NAMES.iter().enumerate().find(|(_, name)| *name == &file.name()) {
            let i: u32 = i.try_into().unwrap();
            quote!{let offset = super::load_offset(baserom, #i)?;}
        } else {
            let offset = file.range_start;
            quote!{let offset = #offset;}
        };

        let code_loadfields = file.items.iter().map(|item| {
            let name = item.name().sanitize().to_case(Case::Snake).to_ident();
            let loadfn = item.loadfn();

            quote!{ let #name = super::#loadfn(baserom, offset)?; }
        });
        let code_assignfields = file.items.iter().map(|item| {
            let name = item.name().sanitize().to_case(Case::Snake).to_ident();
            quote!{ #name, }
        });

        file_structs.push(quote! {
            pub struct #name {
                #(#fields)*
            }

            impl super::Load for #name {
                fn load(baserom: &mut std::fs::File) -> anyhow::Result<Self> {
                    #code_getoffset
                    #(#code_loadfields)*

                    Ok(Self {
                        #(#code_assignfields)*
                    })
                }
            }
        })
    }

    let code = quote! {
            #(#file_structs)*

    /*
            #[derive(Debug, ::num_derive::ToPrimitive)]
            pub enum File {
                //#(variants)*
            }

            impl File {
                const fn offset(&self) -> u32 {
                    match self {
                    }
                }
            }
    */
        };

    let code = format!("{}", code);
    let code = rustfmt_generated_string(&code).unwrap();
    writeln!(out, "{}", code).unwrap();
    writeln!(out, "// num: {}, most: {}", files.len(), most).unwrap();

    let mut lengths:Vec<_> = files.iter().map(|(_, file)| (file.name().to_string(), file.items.len())).collect();
    lengths.sort_by(|(_, lena), (_, lenb)| lena.partial_cmp(lenb).unwrap());

    for (name, length) in lengths {
        writeln!(out, "// {}: {}", length, name).unwrap();
    }

    /*
        for filename in FILE_NAMES {
            writeln!(out, "    {},", filename.to_case(Case::Pascal)).unwrap();


        for (i, filename) in FILE_NAMES.iter().enumerate() {
            writeln!(out, "    Self::{} => FILE_TABLE_OFFSET + {} * 16,", filename.to_case(Case::Pascal), i).unwrap();
        }
    */

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=table.rs");

    Ok(())
}
