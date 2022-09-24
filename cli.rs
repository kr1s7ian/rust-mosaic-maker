use std::{env, convert::Infallible, num::ParseIntError};

use crate::minecraftify::{minecrafify_img, self, Blocks, convert_threaded_png};

pub fn print_usage() {
    println!("Usage: INPUTFILE OUTPUTFILE THREADS");
    println!("Example: input.png output.png 8");
}

struct Ctx {
    file: String,
    out: String,
    threads: usize,
}

impl Ctx {
    pub fn parse_args() -> Result<Self, ParseIntError> {
        let args: Vec<String> = env::args().skip(1).collect();
        if args.len() != 3 {
            print_usage();
            std::process::exit(-1);
        }

        let file = args[0].clone();
        let out = args[1].clone();

        let threads: usize = args[2].parse()?;

        Ok(Self { file, out, threads })
    }
}

pub async fn run () {
    let blocks = match Blocks::load("blocks/").await {
        Ok(v) => {
            v
        },
        Err(_) => {
            println!("No blocks folder found, create a blocks folder and place inside all minecraft blocks you want your mosaic to be composed by");
            std::process::exit(-1);
        },
    };
    let ctx = match Ctx::parse_args() {
        Ok(v) => {
            v
        },
        Err(_) => {
            print_usage();
            std::process::exit(-1);
        }
    };

    match convert_threaded_png(&ctx.file, &ctx.out, blocks, ctx.threads) {
        Ok(_) => {
            println!("Saving to {}", ctx.out);
        },
        Err(_) => {
            println!("Could not locate input file '{}'", ctx.file);
            std::process::exit(-1);
        }
    }
}
