use std::io::Read;
use std::net::{TcpListener, TcpStream};
use clap::Parser;
use thermal_parser::command::Command;
use thermal_parser::context::Context;
use thermal_renderer::image_renderer::ImageRenderer;
use thermal_renderer::renderer::CommandRenderer;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct Cli {
    /// Path to save rendered prints
    save_path: String,

    /// Address to listen on i.e. localhost:9100
    #[arg(short, long)]
    address: String,
}

fn handle_client(cli: &Cli, mut stream: TcpStream) -> std::io::Result<()> {
    let mut buf: Vec<u8> = vec![];
    let amt = stream.read_to_end(&mut buf)?;

    println!("Received request of size {amt}");
    if buf.len() > 0 {
        let mut renderer = ImageRenderer::new(cli.save_path.clone());

        let mut context = Context::new();

        //Create a closure to handle parsed commands
        let on_new_command = move |cmd: Command| {
            //Pass the commands through to the renderer
            renderer.process_command(&mut context, &cmd);
        };

        //Create a new parser with the esc/pos command set
        let mut command_parser = thermal_parser::new_esc_pos_parser(Box::from(on_new_command));

        //Parse the bytes (Commands will be sent to the closure above)
        command_parser.parse_bytes(&buf);
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    let listener = TcpListener::bind(cli.address.clone())?;

    let local = listener.local_addr()?;
    println!("Listening on {local}");

    for stream in listener.incoming() {
        handle_client(&cli, stream?)?;
    }

    Ok(())
}
