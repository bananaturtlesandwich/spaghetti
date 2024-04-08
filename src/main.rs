use clap::Parser;
use unreal_asset::exports::ExportBaseTrait;

mod io;
mod transplant;

#[derive(Parser)]
#[command(about, long_about = None)]
struct Cli {
    #[arg(value_name = "hook")]
    hook: std::path::PathBuf,
    #[arg(value_name = "original")]
    orig: std::path::PathBuf,
    #[arg(value_name = "overwrite original")]
    overwrite: bool,
}

fn main() {
    let Cli {
        hook: hook_path,
        orig: mut orig_path,
        overwrite,
    } = Cli::try_parse().unwrap_or_else(|err| {
        match rfd::FileDialog::new()
            .set_title("select the hook asset")
            .pick_file()
            .zip(
                rfd::FileDialog::new()
                    .set_title("select the original asset")
                    .pick_file(),
            ) {
            Some((hook, orig)) => Cli {
                hook,
                orig,
                overwrite: true,
            },
            None => err.exit(),
        }
    });
    let hook = io::open(
        hook_path,
        unreal_asset::engine_version::EngineVersion::VER_UE5_1,
    )
    .unwrap();
    let mut orig = io::open(
        &orig_path,
        unreal_asset::engine_version::EngineVersion::VER_UE5_1,
    )
    .unwrap();
    let mut name_map = orig.get_name_map();
    // why does it need the import for cast?
    use unreal_asset::Export;
    let funcs: Vec<_> = hook
        .asset_data
        .exports
        .iter()
        .enumerate()
        .filter_map(|(i, ex)| unreal_asset::cast!(Export, FunctionExport, ex).map(|ex| (i, ex)))
        .collect();
    // use index so i can push to exports
    for func in orig
        .asset_data
        .exports
        .iter_mut()
        .filter_map(|ex| unreal_asset::cast!(Export, FunctionExport, ex))
    {
        if !funcs
            .iter()
            .any(|(_, fun)| fun.get_base_export().object_name == func.get_base_export().object_name)
        {
            continue;
        }
        func.get_base_export_mut().object_name = name_map.get_mut().add_fname(
            &func
                .get_base_export()
                .object_name
                .get_content(|name| format!("orig_{name}")),
        );
    }
    for (i, _) in funcs {
        transplant::transplant(i, &mut orig, &hook)
    }
    if !overwrite {
        orig_path.set_file_name(format!(
            "hooked_{:?}",
            orig_path.file_name().unwrap_or_default()
        ))
    }
    io::save(&mut orig, &orig_path).unwrap();
}
