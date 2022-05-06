use std::fs;

pub fn ls(path: &str) {
    match fs::read_dir(path) {
        Err(why) => println!("! found err: {:?}", why.kind()),
        Ok(paths) => {
            for path in paths {
                println!("> {:?}", path.unwrap().path());
            }
        }
    }
}

pub fn sleep() {}
