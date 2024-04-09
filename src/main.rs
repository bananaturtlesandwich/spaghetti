use clap::Parser;
use unreal_asset::exports::ExportBaseTrait;

mod cli;
mod io;
mod transplant;

fn main() {
    let cli::Cli {
        hook: hook_path,
        orig: orig_path,
        mut output,
        version,
    } = cli::Cli::parse();
    let ignored = hook_path.is_none() && orig_path.is_none() && output.is_none();
    let hook_path = hook_path.unwrap_or_else(|| {
        rfd::FileDialog::new()
            .set_title("select the hook-containing blueprint")
            .add_filter("unreal asset", &["uasset", "umap"])
            .pick_file()
            .unwrap_or_else(|| std::process::exit(0))
    });
    let orig_path = orig_path.unwrap_or_else(|| {
        rfd::FileDialog::new()
            .set_title("select the original blueprint")
            .add_filter("unreal asset", &["uasset", "umap"])
            .pick_file()
            .unwrap_or_else(|| std::process::exit(0))
    });
    if ignored {
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
    let version = version.0;
    let hook = io::open(hook_path, version).unwrap();
    let mut orig = io::open(&orig_path, version).unwrap();
    let mut name_map = orig.get_name_map().clone_resource();
    // why does it need the import for cast?
    use unreal_asset::Export;
    let mut funcs: Vec<_> = hook
        .asset_data
        .exports
        .iter()
        .enumerate()
        .filter_map(|(i, ex)| {
            unreal_asset::cast!(Export, FunctionExport, ex).and_then(|ex| {
                ex.get_base_export().object_name.get_content(|name| {
                    (!name.starts_with("orig_") && !name.starts_with("ExecuteUbergraph_"))
                        .then(|| (i, name.to_string()))
                })
            })
        })
        .collect();
    let (class, exports) = orig.asset_data.exports.split_at_mut(1);
    let Export::ClassExport(class) = &mut class[0] else {
        std::process::exit(0)
    };
    let mut hooks = Vec::with_capacity(funcs.capacity());
    for (i, orig) in exports
        .iter_mut()
        .enumerate()
        .filter_map(|(i, ex)| unreal_asset::cast!(Export, FunctionExport, ex).map(|ex| (i, ex)))
    {
        let Some(hook) = funcs.iter().position(|(_, name)| {
            orig.get_base_export()
                .object_name
                .get_content(|orig| &format!("hook_{orig}") == name)
        }) else {
            continue;
        };
        hooks.push(funcs.remove(hook));
        orig.get_base_export_mut().object_name = name_map.get_mut().add_fname(
            &orig
                .get_base_export()
                .object_name
                .get_content(|name| format!("orig_{name}")),
        );
        class.func_map.insert(
            orig.get_base_export().object_name.clone(),
            unreal_asset::types::PackageIndex {
                // i + 2 since exports is one export short
                index: (i + 2) as i32,
            },
        )
    }
    // len + 1 since exports is one export short
    let mut insert = exports.len() + 1;
    for (i, (_, name)) in funcs.iter().enumerate() {
        if class.func_map.keys().any(|key| key == name) {
            continue;
        }
        class.func_map.insert(
            name_map.get_mut().add_fname(name),
            unreal_asset::types::PackageIndex {
                index: (insert + i + 1) as i32,
            },
        );
    }
    insert += funcs.len();
    for (i, (_, name)) in hooks.iter().enumerate() {
        let name = name.trim_start_matches("hook_");
        println!("{name} hooked");
        class.func_map.insert(
            name_map.get_mut().add_fname(name),
            unreal_asset::types::PackageIndex {
                index: (insert + i + 1) as i32,
            },
        );
    }
    for (i, _) in funcs.into_iter().chain(hooks.into_iter()) {
        transplant::transplant(i, &mut orig, &hook)
    }
    io::save(&mut orig, output.unwrap_or(orig_path)).unwrap();
}
