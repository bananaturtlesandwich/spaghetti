use clap::Parser;
use unreal_asset::exports::ExportBaseTrait;

mod cli;
mod io;
mod transplant;

fn main() {
    let cli::Cli {
        hook: hook_path,
        orig: orig_path,
        output,
        version,
    } = cli::Cli::parse();
    let hook_path = hook_path.unwrap_or_else(|| {
        rfd::FileDialog::new()
            .set_title("select the hook-containing blueprint")
            .pick_file()
            .unwrap_or_else(|| std::process::exit(0))
    });
    let orig_path = orig_path.unwrap_or_else(|| {
        rfd::FileDialog::new()
            .set_title("select the original blueprint")
            .pick_file()
            .unwrap_or_else(|| std::process::exit(0))
    });
    let version = version.0;
    let hook = io::open(hook_path, version).unwrap();
    let mut orig = io::open(&orig_path, version).unwrap();
    let mut name_map = orig.get_name_map();
    // why does it need the import for cast?
    use unreal_asset::Export;
    let funcs: Vec<_> = hook
        .asset_data
        .exports
        .iter()
        .enumerate()
        .filter_map(|(i, ex)| {
            unreal_asset::cast!(Export, FunctionExport, ex)
                .filter(|ex| {
                    ex.get_base_export()
                        .object_name
                        .get_content(|name| !name.starts_with("orig_"))
                })
                .map(|ex| (i, ex))
        })
        .collect();
    // use index so i can push to exports
    for orig in orig
        .asset_data
        .exports
        .iter_mut()
        .filter_map(|ex| unreal_asset::cast!(Export, FunctionExport, ex))
    {
        if !funcs.iter().any(|(_, func)| {
            func.get_base_export().object_name.get_content(|func| {
                orig.get_base_export().object_name == func.trim_start_matches('_')
            })
        }) {
            continue;
        }
        orig.get_base_export_mut().object_name = name_map.get_mut().add_fname(
            &orig
                .get_base_export()
                .object_name
                .get_content(|name| format!("orig_{name}")),
        );
    }
    for (i, _) in funcs {
        transplant::transplant(i, &mut orig, &hook)
    }
    io::save(&mut orig, output.unwrap_or(orig_path)).unwrap();
}
