#![allow(non_snake_case)]
use std::env;
use std::fs;
use std::process;
use std::io;
use std::io::BufRead;
use std::io::Write;

fn main() 
{
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1)
    });

    println!("Creating {} python render script(s)", config.nScripts);
    println!("Reading animation data from //batchSriptData/{}", config.animFile);
    println!("Reading render script body text from //renderScriptBodies/{}", config.scriptFile);

    let currentDir = env::current_dir().unwrap();
    let mut animFilePath = currentDir.clone();
    animFilePath.push("batchSriptData");
    animFilePath.push(config.animFile.clone());
    // println!("{}", animFilePath.display());

    // Open animation text file, and read only uncommented lines
    let file = fs::File::open(animFilePath).unwrap();
    let f = io::BufReader::new(file);

    let mut animations = Vec::new();
    for line in f.lines() {
        let lineString = line.unwrap();
        if !lineString.contains(";")
        {
            // println!("{}", lineString);
            animations.push(lineString);
        }
    }

    // Check if the folder exists
    let mut extractPath = currentDir.clone();
    extractPath.push("renderGeneratedScripts");
    let dirExists: bool = extractPath.is_dir();
    if !dirExists
    {
        fs::create_dir_all(&extractPath).unwrap();
    }


    //Figure out how to divide the animations based on nScripts.
    if config.nScripts == 1 {
        createScript(&animations, &currentDir, &config, 0);
    }
    else
    {
        let nAnimations = animations.len();
        let quotient = nAnimations as u32 / config.nScripts as u32;
        let remainder = nAnimations as u32 % config.nScripts as u32;
        let n = config.nScripts as u32;

        for j in 0..n {
            let a = (j*quotient) as usize;
            let b = (j*quotient+quotient) as usize;
            if remainder != 0 && j == n-1 {
                let sliceRange = a..;
                let animationSlice = &animations[sliceRange];
                createScript(animationSlice, &currentDir, &config, j as u8);
            }
            else {
                let sliceRange = a..b;
                let animationSlice = &animations[sliceRange];
                createScript(animationSlice, &currentDir, &config, j as u8);
            }
        }
    }
}


struct Config {
    nScripts: u8,
    animFile: String,
    scriptFile: String,
}
impl Config {
    fn new(args: &[String]) -> Result<Config, String> {
        if args.len() < 4 {
            let errString = String::from("Not enough arguments!\nArguments must be amount of scripts to create, animation data filename and filename for python script body\nEg. ") + &args[0] + ".exe \"1\" \"noWeaponAnims.txt\" \"batchrender-noweapons.py\"";
            return Err(errString);
        }

        let nScripts = args[1].parse::<u8>().unwrap();
        let animFile = args[2].clone();
        let scriptFile = args[3].clone();

        Ok(Config {nScripts, animFile, scriptFile})
    }
}


fn createScript(animations: &[String], currentDir: &std::path::PathBuf, config: &Config, nScript: u8)
{
    // Create header text for python script
    let mut header = String::from("import os\nimport sys\nscriptpath = \"");
    header.push_str(&currentDir.display().to_string());
    let mut header = header.replace("\\", "/");
    header.push_str("/");
    header.push_str("\"\nsys.path.append(os.path.abspath(scriptpath))\nimport helpers\nimport bpy\n");
    header.push_str("\n# Animation name in blender & end frame\nanimationArray = [\n");

    // read python script body text
    let mut scriptFilePath = currentDir.clone();
    scriptFilePath.push("renderScriptBodies");
    scriptFilePath.push(config.scriptFile.clone());
    let scriptBody = fs::read_to_string(scriptFilePath)
        .expect("Something went wrong reading the file");

    // Add animations to the script
    let mut animArray = String::new();
    for anim in animations {
        // println!("{}", anim);
        animArray.push_str(&anim);
        animArray.push('\n');
    }
    animArray.push_str("]\n\n");
    
    let generatedScript = header + &animArray + &scriptBody;


    // Save script to file
    let scriptFilename = &config.scriptFile;
    let scriptFilename = scriptFilename.replace("unified.py", &format!("{}", &config.animFile));
    let scriptFilename = scriptFilename.replace(".txt", &format!("{}.py", nScript));

    let scriptFile = format!("renderGeneratedScripts\\{}", scriptFilename);
    let mut output = fs::File::create(scriptFile).unwrap();
    write!(output, "{}", generatedScript).unwrap();
}