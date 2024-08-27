use unreal_asset::engine_version::EngineVersion as E;

#[derive(clap::Parser)]
#[command()]
pub struct Cli {
    /// path to blueprint
    #[arg(value_name = "blueprint")]
    pub orig: Option<std::path::PathBuf>,
    /// engine version used to create the blueprints [default: 5.1]
    #[arg(short, value_parser = clap::value_parser!(Version))]
    pub version: Option<Version>,
    /// path to save the hooked blueprint to [default: overwrites original]
    #[arg(short, value_name = "output path")]
    pub output: Option<std::path::PathBuf>,
}

#[derive(Clone)]
pub struct Version(pub unreal_asset::engine_version::EngineVersion);

impl clap::builder::ValueParserFactory for Version {
    type Parser = VersionParser;
    fn value_parser() -> Self::Parser {
        VersionParser
    }
}

#[derive(Clone)]
pub struct VersionParser;
impl VersionParser {
    pub fn parse(value: &str) -> Option<Version> {
        let value = value.trim();
        VERSIONS
            .iter()
            .find_map(|(ver, name)| (&value == name).then_some(Version(*ver)))
    }
}
impl clap::builder::TypedValueParser for VersionParser {
    type Value = Version;

    fn parse_ref(
        &self,
        _: &clap::Command,
        _: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        Self::parse(&value.to_str().unwrap_or_default())
            .ok_or_else(|| clap::Error::new(clap::error::ErrorKind::InvalidValue))
    }
}

const VERSIONS: [(E, &str); 33] = [
    (E::UNKNOWN, "unknown"),
    (E::VER_UE4_OLDEST_LOADABLE_PACKAGE, "oldest"),
    (E::VER_UE4_0, "4.0"),
    (E::VER_UE4_1, "4.1"),
    (E::VER_UE4_2, "4.2"),
    (E::VER_UE4_3, "4.3"),
    (E::VER_UE4_4, "4.4"),
    (E::VER_UE4_5, "4.5"),
    (E::VER_UE4_6, "4.6"),
    (E::VER_UE4_7, "4.7"),
    (E::VER_UE4_8, "4.8"),
    (E::VER_UE4_9, "4.9"),
    (E::VER_UE4_10, "4.10"),
    (E::VER_UE4_11, "4.11"),
    (E::VER_UE4_12, "4.12"),
    (E::VER_UE4_13, "4.13"),
    (E::VER_UE4_14, "4.14"),
    (E::VER_UE4_15, "4.15"),
    (E::VER_UE4_16, "4.16"),
    (E::VER_UE4_17, "4.17"),
    (E::VER_UE4_18, "4.18"),
    (E::VER_UE4_19, "4.19"),
    (E::VER_UE4_20, "4.20"),
    (E::VER_UE4_21, "4.21"),
    (E::VER_UE4_22, "4.22"),
    (E::VER_UE4_23, "4.23"),
    (E::VER_UE4_24, "4.24"),
    (E::VER_UE4_25, "4.25"),
    (E::VER_UE4_26, "4.26"),
    (E::VER_UE4_27, "4.27"),
    (E::VER_UE5_0, "5.0"),
    (E::VER_UE5_1, "5.1"),
    (E::VER_UE5_2, "5.2"),
];
