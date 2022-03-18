use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::fmt;

struct DataObject {
    classifier: String,
    data: Vec<f32>
}

impl std::fmt::Display for DataObject {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Data-Object: CLASS [{}], DATA: [{:?}]", self.classifier, self.data)
    }

}

impl TryFrom<&Vec<&str>> for DataObject {

    type Error = String;

    fn try_from(source: &Vec<&str>) -> Result<Self, Self::Error> {
        // assume that string-value is always last

        let classifier: String;

        match source.get(source.len() - 1) {

            Some(result) => classifier = result.to_string(),
            None => return Err("Invalid input column for DataObject".to_string())

        }

        let mut data = Vec::<f32>::new();

        for i in 0..source.len()-1 {

            match source[i].parse::<f32>() {

                Ok(result) => data.push(result),
                Err(error) => return Err(format!("Error while reading data: {}.", error))

            }
        }

        Ok( DataObject {classifier, data} )
    }

}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn load_data(filename: &str){

    if let Ok(lines) = read_lines(filename){
        let mut i = 1;
        for line in lines {
            if let Ok(text) = line {
                let columns: Vec<&str> = text.split(",").collect();
                match DataObject::try_from(&columns) {
                    Ok(data_object) => println!("{}", data_object),
                    Err(errmsg) => println!("Could not parse columns on line {}: {} into DataObject, because: {}", i, text, errmsg)
                }
            }
            i+=1;
        }
    }

}

fn main() {
    load_data("../iris/iris/train.txt");
}
