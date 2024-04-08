use clap::Parser;

#[derive(Parser)]
#[command(about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "source")]
    src: std::path::PathBuf,
    #[arg(short, long, value_name = "destination")]
    dest: std::path::PathBuf,
}

fn main() {
    let Cli { src, dest } = Cli::try_parse().unwrap_or_else(|err| {
        match rfd::FileDialog::new()
            .set_title("select the source asset")
            .pick_file()
            .zip(
                rfd::FileDialog::new()
                    .set_title("select the destination asset")
                    .pick_file(),
            ) {
            Some((src, dest)) => Cli { src, dest },
            None => err.exit(),
        }
    });
}
