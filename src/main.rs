use clap::Parser;
// export imported because cast macro doesn't use pattern
use unreal_asset::exports::{Export, ExportBaseTrait};

mod cli;
mod io;
mod kismet;

fn main() {
    let (orig_path, output, mut blueprint) = args();
    let mut name_map = blueprint.get_name_map().clone_resource();
    // duplicate functions
    let insert = blueprint.asset_data.exports.len();
    let functions: Vec<_> = blueprint
        .asset_data
        .exports
        .iter()
        .enumerate()
        .filter_map(|(i, ex)| {
            match !ex
                .get_base_export()
                .object_name
                .get_content(|name| name.starts_with("ExecuteUbergraph"))
            {
                true => unreal_asset::cast!(Export, FunctionExport, ex).map(|ex| (i, ex.clone())),
                false => None,
            }
        })
        .collect();
    let Some(class) = blueprint
        .asset_data
        .exports
        .iter_mut()
        .find_map(|ex| unreal_asset::cast!(Export, ClassExport, ex))
    else {
        eprintln!("class export couldn't be found");
        std::process::exit(0)
    };
    // two loops so class gets dropped
    for (i, (_, function)) in functions.iter().enumerate() {
        // replace old functions
        class.func_map.insert(
            function.get_base_export().object_name.clone(),
            unreal_asset::types::PackageIndex::new((insert + i) as i32 + 1),
        );
    }
    for (old, mut function) in functions {
        // duplication is super simple for functions since they have no export refs
        let name = &mut blueprint.asset_data.exports[old]
            .get_base_export_mut()
            .object_name;
        // need to improve name api to not clone as often
        *name = name_map
            .get_mut()
            .add_fname(&name.get_content(|name| format!("orig_{name}")));
        kismet::hook(&mut function, &mut name_map, &mut blueprint);
        blueprint
            .asset_data
            .exports
            .push(Export::FunctionExport(function));
    }
    io::save(&mut blueprint, output.unwrap_or(orig_path)).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(0);
    });
}

fn args() -> (
    std::path::PathBuf,
    Option<std::path::PathBuf>,
    unreal_asset::Asset<std::io::BufReader<std::fs::File>>,
) {
    let cli::Cli {
        orig: orig_path,
        mut output,
        version,
    } = cli::Cli::parse();
    let ignored = orig_path.is_none();
    let orig_path = orig_path.unwrap_or_else(|| {
        rfd::FileDialog::new()
            .set_title("select the blueprint to hook")
            .add_filter("unreal asset", &["uasset", "umap"])
            .pick_file()
            .unwrap_or_else(|| {
                eprintln!("no blueprint selected");
                std::process::exit(0);
            })
    });
    if ignored && output.is_none() {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("save hooked blueprint [default: overwrites original]")
            .add_filter("unreal asset", &["uasset", "umap"])
            .set_file_name(
                orig_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default(),
            )
            .save_file()
        {
            output = Some(path)
        }
    }
    let version = match version {
        Some(version) => version.0,
        None if ignored => {
            print!("version [default: 5.1]: ");
            std::io::Write::flush(&mut std::io::stdout())
                .map_err(clap::Error::from)
                .unwrap_or_else(|e| e.exit());
            let mut buf = String::new();
            std::io::stdin()
                .read_line(&mut buf)
                .map_err(clap::Error::from)
                .unwrap_or_else(|e| e.exit());
            cli::VersionParser::parse(&buf)
                .map(|v| v.0)
                .unwrap_or(unreal_asset::engine_version::EngineVersion::VER_UE5_1)
        }
        None => unreal_asset::engine_version::EngineVersion::VER_UE5_1,
    };
    let blueprint = io::open(&orig_path, version).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(0);
    });
    (orig_path, output, blueprint)
}
