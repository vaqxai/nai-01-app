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

trait FromColumns {
    fn from_columns(columns: &Vec<&str>) -> DataObject;
}

impl FromColumns for DataObject {
    fn from_columns(columns: &Vec<&str>) -> DataObject {

        // assume that string-value is always last

        let classifier: String;

        match columns.get(columns.len() - 1) {

            Some(result) => classifier = result.to_string(),
            None => panic!("Invalid input column for DataObject")

        }

        let mut data = Vec::<f32>::new();

        for i in 0..columns.len()-1 {

            match columns[i].parse::<f32>() {

                Ok(result) => data.push(result),
                Err(error) => panic!("Fatal error while reading data: {}", error)

            }
        }

        DataObject { classifier, data }
    }
}


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
                let data_object = DataObject::from_columns(&columns);
                println!("{}", data_object );
            }
        }
    }

}

fn main() {
    load_data("../iris/iris/test.txt");
}
