use clap::Parser;
// export imported because cast macro doesn't use pattern
use unreal_asset::exports::{Export, ExportBaseTrait};

mod cli;
mod io;
// have to inline kismet because of differences between engine versions
mod kismet;

fn main() {
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
    let mut blueprint = io::open(&orig_path, version).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(0);
    });
    let mut name_map = blueprint.get_name_map().clone_resource();
    // i could split like before but there's really no point
    let redirects: Vec<_> = blueprint
        .asset_data
        .exports
        .iter_mut()
        .enumerate()
        .filter_map(|(i, ex)| unreal_asset::cast!(Export, FunctionExport, ex).map(|ex| (i, ex)))
        .map(|(i, func)| {
            let name = name_map.get_mut().add_fname(
                &func
                    .get_base_export()
                    .object_name
                    .get_content(|name| format!("orig_{name}")),
            );
            func.get_base_export_mut().object_name = name.clone();
            // return these to be added to the function map in ClassExport
            (
                name,
                unreal_asset::types::PackageIndex {
                    index: i as i32 + 1,
                },
            )
        })
        .collect();
    // let insert = blueprint.asset_data.exports.len() as i32 + 1;
    let bytecode = kismet::init(name_map.get_mut(), &mut blueprint);
    let Some(class) = blueprint
        .asset_data
        .exports
        .iter_mut()
        .find_map(|ex| unreal_asset::cast!(Export, ClassExport, ex))
    else {
        eprintln!("class export couldn't be found");
        std::process::exit(0)
    };
    let Some(beginplay) = class
        .func_map
        .iter()
        .find_map(|(_, key, val)| (key == "ReceiveBeginPlay").then(|| val.clone()))
    else {
        eprintln!("adding ReceiveBeginPlay currently isn't supported");
        std::process::exit(0)
    };
    class.func_map.extend(redirects);
    let Export::FunctionExport(beginplay) =
        &mut blueprint.asset_data.exports[beginplay.index as usize - 1]
    else {
        eprintln!("ReceiveBeginPlay couldn't be retrieved");
        std::process::exit(0)
    };
    beginplay.struct_export.script_bytecode = Some(bytecode);
    // let mut insert = back.len() + split + 1;
    // for (i, (_, name)) in funcs.iter().enumerate() {
    //     class.func_map.insert(
    //         name_map.get_mut().add_fname(name),
    //         unreal_asset::types::PackageIndex {
    //             index: (insert + i + 1) as i32,
    //         },
    //     );
    // }
    // insert += funcs.len();
    // for (i, (_, name)) in hooks.iter().enumerate() {
    //     let name = name.trim_start_matches("hook_");
    //     println!("{name} hooked");
    //     class.func_map.insert(
    //         name_map.get_mut().add_fname(name),
    //         unreal_asset::types::PackageIndex {
    //             index: (insert + i + 1) as i32,
    //         },
    //     );
    // }
    // for (i, _) in funcs.into_iter().chain(hooks.into_iter()) {
    //     transplant::transplant(i, &mut orig, &hook)
    // }
    io::save(&mut blueprint, output.unwrap_or(orig_path)).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(0);
    });
}
