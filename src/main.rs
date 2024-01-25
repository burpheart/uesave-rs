use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{stdin, stdout, BufRead, BufReader, BufWriter, Cursor, Write};
use std::process::exit;

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};

use uesave::{Save, StructType, Types};

#[derive(Parser, Debug)]
struct ActionToJson {
    #[arg(short, long, default_value = "-")]
    input: String,

    #[arg(short, long, default_value = "-")]
    output: String,

    /// Save files do not contain enough context to parse structs inside MapProperty or SetProperty.
    /// uesave will attempt to guess, but if it is incorrect the save will fail to parse and the
    /// type must be manually specified.
    ///
    /// Examples:
    ///   -t .UnlockedItemSkins.Skins=Guid
    ///   -t .EnemiesKilled.Key=Guid
    ///   -t .EnemiesKilled.Value=Struct
    #[arg(short, long, value_parser = parse_type)]
    r#type: Vec<(String, StructType)>,
}

#[derive(Parser, Debug)]
struct ActionFromJson {
    #[arg(short, long, default_value = "-")]
    input: String,

    #[arg(short, long, default_value = "-")]
    output: String,
}

#[derive(Parser, Debug)]
struct ActionEdit {
    #[arg(required = true, index = 1)]
    path: String,

    /// Save files do not contain enough context to parse structs inside MapProperty or SetProperty.
    /// uesave will attempt to guess, but if it is incorrect the save will fail to parse and the
    /// type must be manually specified.
    ///
    /// Examples:
    ///   -t .UnlockedItemSkins.Skins=Guid
    ///   -t .EnemiesKilled.Key=Guid
    ///   -t .EnemiesKilled.Value=Struct
    #[arg(short, long, value_parser = parse_type)]
    r#type: Vec<(String, StructType)>,
}

#[derive(Parser, Debug)]
struct ActionTestResave {
    #[arg(required = true, index = 1)]
    path: String,

    /// If resave fails, write input.sav and output.sav to working directory for debugging
    #[arg(short, long)]
    debug: bool,

    /// Save files do not contain enough context to parse structs inside MapProperty or SetProperty.
    /// uesave will attempt to guess, but if it is incorrect the save will fail to parse and the
    /// type must be manually specified.
    ///
    /// Examples:
    ///   -t .UnlockedItemSkins.Skins=Guid
    ///   -t .EnemiesKilled.Key=Guid
    ///   -t .EnemiesKilled.Value=Struct
    #[arg(short, long, value_parser = parse_type)]
    r#type: Vec<(String, StructType)>,
}

#[derive(Subcommand, Debug)]
enum Action {
    /// Convert binary save to plain text JSON
    ToJson(ActionToJson),
    /// Convert JSON back to binary save
    FromJson(ActionFromJson),
    /// Launch editor to edit a save file as JSON in place
    Edit(ActionEdit),
    /// Test resave
    TestResave(ActionTestResave),
}

#[derive(Parser, Debug)]
#[command(author, version)]
struct Args {
    #[command(subcommand)]
    action: Action,
}

fn parse_type(t: &str) -> Result<(String, StructType)> {
    if let Some((l, r)) = t.rsplit_once('=') {
        Ok((l.to_owned(), r.into()))
    } else {
        Err(anyhow!("Malformed type"))
    }
}

pub fn main() -> Result<()> {
    
    let args: Vec<String> = env::args().collect();


        //let file_result = File::open("C:\\Users\\cnc\\Downloads\\uesave-x86_64-pc-windows-msvc\\8B9E670B4FDCBD037E1323A482D41CFC\\Level.sav.gvas");
        if &args[1] == "raw2json"{
                let file_result = File::open(&args[2]);
    if let Ok(mut file) = file_result {
            let tset = uesave::Context::run(&mut file, uesave::read_property)?.unwrap();;
            print!("{}", serde_json::to_string_pretty(&&tset.1)?);
        
    } else {
        
        println!("Failed to open file");
         exit(1);
        
    }
            exit(0);
        }else if &args[1] == "json2raw"{
                let file_result = File::open(&args[2]);
    if let Ok(mut file) = file_result {
                let save: uesave::Property = serde_json::from_reader(&mut file)?;
                let mut file = File::create(&args[3])?;
                uesave::Context::run(&mut file, |writer| {
                    uesave::write_property((&"SaveParameter".to_string(), &save), writer)
                })?;
   } else {
        
        println!("Failed to open file");
         exit(1);
        
    }
            
    }
       



         

    
    
    let args = Args::parse();

    match args.action {
        Action::ToJson(action) => {
            let mut types = Types::new();
            for (path, t) in action.r#type {
                types.add(path, t);
            }

            let save = Save::read_with_types(&mut input(&action.input)?, &types)?;
            serde_json::to_writer_pretty(output(&action.output)?, &save)?;
        }
        Action::FromJson(io) => {
            let save: Save = serde_json::from_reader(&mut input(&io.input)?)?;
            save.write(&mut output(&io.output)?)?;
        }
        Action::TestResave(action) => {
            let mut types = Types::new();
            for (path, t) in action.r#type {
                types.add(path, t);
            }

            let mut input = std::io::Cursor::new(fs::read(action.path)?);
            let mut output = std::io::Cursor::new(vec![]);
            Save::read_with_types(&mut input, &types)?.write(&mut output)?;
            let (input, output) = (input.into_inner(), output.into_inner());
            if input != output {
                if action.debug {
                    fs::write("input.sav", input)?;
                    fs::write("output.sav", output)?;
                }
                return Err(anyhow!("Resave did not match"));
            }
            println!("Resave successful");
        }
        Action::Edit(action) => {
            let mut types = Types::new();
            for (path, t) in action.r#type {
                types.add(path, t);
            }

            let save = Save::read_with_types(&mut Cursor::new(fs::read(&action.path)?), &types)?;
            let modified_save: Save = serde_json::from_slice(&edit::edit_bytes_with_builder(
                serde_json::to_vec_pretty(&save)?,
                tempfile::Builder::new().suffix(".json"),
            )?)?;

            if save == modified_save {
                println!("File unchanged, doing nothing.");
            } else {
                println!("File modified, writing new save.");
                modified_save.write(&mut BufWriter::new(
                    OpenOptions::new()
                        .create(true)
                        .truncate(true)
                        .write(true)
                        .open(action.path)?,
                ))?;
            }
        }
    }
    Ok(())
}

fn input<'a>(path: &str) -> Result<Box<dyn BufRead + 'a>> {
    Ok(match path {
        "-" => Box::new(BufReader::new(stdin().lock())),
        p => Box::new(BufReader::new(File::open(p)?)),
    })
}

fn output<'a>(path: &str) -> Result<Box<dyn Write + 'a>> {
    Ok(match path {
        "-" => Box::new(BufWriter::new(stdout().lock())),
        p => Box::new(BufWriter::new(
            OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(p)?,
        )),
    })
}
