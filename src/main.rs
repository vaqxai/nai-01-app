use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn load_data(filename: &str){

    if let Ok(lines) = read_lines(filename){
        for line in lines {
            if let Ok(text) = line {
                let columns: Vec<&str> = text.split(",").collect();
                println!("{:?}", columns );
            }
        }
    }

}

fn main() {
    load_data("../iris/iris/test.txt");
}
