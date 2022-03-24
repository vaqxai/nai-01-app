use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::{fmt, env};
use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;

#[derive(PartialEq)]
struct DataObject {
    id: usize,
    classifier: String,
    data: Vec<f32>
}

impl Eq for DataObject {
}

impl Hash for DataObject {
    fn hash<H: Hasher>(&self, state: &mut H){
        self.id.hash(state);
    }
}

impl std::fmt::Display for DataObject {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Data-Object: ID: [{}] CLASS [{}], DATA: [{:?}]", self.id, self.classifier, self.data)
    }

}

impl TryFrom<&Vec<&str>> for DataObject {

    type Error = String;

    fn try_from(source: &Vec<&str>) -> Result<Self, Self::Error> {
        // assume that string-value is always last

        let id = 0;
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

        Ok( DataObject {id, classifier, data} )
    }

}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn load_data(filename: &str) -> Result<Vec<DataObject>, io::Error> {

    let mut res = Vec::<DataObject>::new();

    if let Ok(lines) = read_lines(filename){
        let mut i = 1;
        for line in lines {
            if let Ok(text) = line {
                let columns = text.split(",").collect();
                match DataObject::try_from(&columns) {
                    Ok(mut data_object) => {
                        data_object.id = i; // assign based on line number
                        println!("{}", data_object);
                        res.push(data_object);
                    },
                    Err(errmsg) => println!("Could not parse columns on line {}: {} into DataObject, because: {}", i, text, errmsg)
                }
            }
            i+=1;
        }
    }

    Ok(res)

}

fn calc_dist(a: &DataObject, b: &DataObject) -> Result<f32, &'static str> {

    if a.data.len() != b.data.len(){
        return Err("Objects need to have the same dimensions!"); // err
    }

    let dim = a.data.len();
    let mut sum= 0.0;

    for i in 0..dim {
        sum += f32::powf(a.data[i] - b.data[i], 2.0);
    }

    Ok(f32::sqrt(sum))

}

fn build_dist_table<'a>(a: &DataObject, data: &'a Vec<DataObject>) -> HashMap<&'a DataObject, f32> {

    let mut res = HashMap::<&DataObject, f32>::new();

    for b in data.iter() {
        let x = match calc_dist(a,b) {
            Ok(x) => x,
            Err(msg) => panic!("Failure building distance table: {}", msg)
        };
        res.insert(b, x);
    }

    res

}

fn sort_by_dist<'a>(disttable: &mut HashMap<&'a DataObject, f32>, k: usize) -> Vec<&'a DataObject> {

    let mut res = Vec::new();

    let mut min: f32 = -1.0;
    let mut minobj: Option<&DataObject> = None;

    while res.len() < k {
    // find min
        for obj in disttable.iter() {

            if res.len() == k { return res }

            if min == -1.0 || min > *obj.1{
                min = *obj.1;
                minobj = Some(*obj.0);
            }

        }

        if min == -1.0 {
            panic!("Not enough neighbors to find nearest 3");
        }

        match minobj {
            Some(x) => {
                res.push(x);
                disttable.remove(x);
            },
            None => panic!("Not enough neighbors to find nearest 3")
        }

    }

    res

}

fn assign_knn_class(a: &mut DataObject, data: &Vec<DataObject>, k: usize) {

    let dist_table = sort_by_dist(&mut build_dist_table(a, data), k);
    let mut fr_tab: HashMap<String, usize> = HashMap::new();

    for obj in dist_table {

        let option_res = fr_tab.get(&obj.classifier);

        let v = match option_res {
            Some(&k) => k,
            None => 1
        };

        let class = &obj.classifier;
        fr_tab.insert(class.to_string(), v);
    }

    let mut max = 0;
    let mut max_class: Option<String> = None;

    for entry in fr_tab {
        if entry.1 > max{
            max_class = Some(entry.0);
            max = entry.1;
        }
    }

    match max_class {
        Some(s) => a.classifier = s,
        None => return
    }

}

fn test_from_file(path: &str, train_data: &Vec<DataObject>, k: usize){
    let test_data = match load_data(path) {
        Ok(data) => data,
        Err(e) => panic!("Error! {}", e)
    };

    let total = test_data.len();
    println!("Test data length: {}", total);
    let mut correct = 0;
    let mut i = 1;

    for mut dataobj in test_data {

        let shouldbe = dataobj.classifier.to_string(); // copy the value

        dataobj.classifier = "?".to_string(); // just to be sure the result is genuine

        assign_knn_class(&mut dataobj, &train_data, k);

        if shouldbe == dataobj.classifier {
            println!("Guessed test data line {} ({}) correctly!", i, dataobj);
            correct += 1;
        }

        i += 1;

    }

    println!("Guessed {} out of {} lines correctly. {}% accuracy.", correct, total, correct as f32/total as f32 * 100.0);

}

fn user_dataobj_test(train_data_path: &str, args: &Vec<String>, mbnd: usize, ubnd: usize){

    let mut columns = Vec::<&str>::new();
    columns.extend(args[mbnd..ubnd].iter().map(String::as_str));
    columns.push("?");

    let mut dataobject = match DataObject::try_from(&columns) {
        Ok(dataobject) => dataobject,
        Err(e) => {
            println!("Could not create a DataObject from the supplied argument vectors: {:?}, because: {}", columns, e);
            return;
        }
    };

    let data = match load_data(train_data_path) {
        Ok(data) => data,
        Err(e) => panic!("Error! {}", e)
    };

    assign_knn_class(&mut dataobject, &data, 3);

    println!("Assigned a class to the following data: {}", dataobject);

}

fn main() {

    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        println!("KNN Rust Implementation");
        println!("Please refer to the following help article:");
        println!("Run this program with arguments to use it.");
        println!("");
        println!("By default, it will use iris train data, located in iris/train.txt");
        println!("If you specify one argument, it will be interpreted as test-data path.");
        println!("E.g: nai-01-app.exe train.txt");
        println!("");
        println!("If you specify two arguments, they will be interpreted as train data + test data.");
        println!("Keep in mind though, that both data-sets need to have the same amount of dimensions.");
        println!("The data files should be formatted like this: x.x,y.y,z.z,...,class-string");
        println!("E.g. nai-01-app.exe iris/train.txt iris/test.txt");
        println!("");
        println!("If you speicfy 'specimen' as the first arg, the rest will be assumed to be floating-point variables. the class will be assumed from the train data from the second-argument file, e.g.");
        println!("nai-01-app.exe specimen iris/train.txt 4.7 3.2 1.6 0.2");
        println!("If you specify 'specimend' as the first arg, the rest will be assumed to be floating-point variables. the class will be assumed from the default train data.");
        println!("nai-01-app.exe specimend 4.7 3.2 1.6 0.2");
    }

    // test specified by user
    if args.len() == 2 {
        println!("Test data file from user...");

        let test_data_path = match args.get(1) {
            Some(path) => path,
            None => panic!("Error! Could not find data-path!")
        };

        if !Path::new(test_data_path).exists() {
            println!("Test data file does not exist/path is incorrect: {}", test_data_path);
            return;
        }

        let data = match load_data("iris/train.txt") {
            Ok(data) => data,
            Err(e) => panic!("Error! {}", e)
        };

        test_from_file(test_data_path, &data, 3);

        return;

    }

    // train+test specified by user
    if args.len() > 2 && args[1] != "specimen" && args[1] != "specimend" {
        println!("Test+Train data file from user...");

        let train_data_path = match args.get(1) {
            Some(path) => path,
            None => panic!("Error! Could not find data-path!")
        };

        if !Path::new(train_data_path).exists() {
            println!("Train data file does not exist/path is incorrect: {}", train_data_path);
            return;
        }

        let test_data_path = match args.get(2) {
            Some(path) => path,
            None => panic!("Error! Could not find data-path!")
        };

        if !Path::new(test_data_path).exists() {
            println!("Test data file does not exist/path is incorrect: {}", test_data_path);
            return;
        }

        let data = match load_data(train_data_path) {
            Ok(data) => data,
            Err(e) => panic!("Error! {}", e)
        };

        test_from_file(test_data_path, &data, 3);

    }

    if args.len() > 2 && args[1] == "specimend" {

        if args.len() != 6 {
            println!("Invalid amount of dimensions for test data object. Needed 4, supplied {}", args.len() - 2);
            return;
        }

        user_dataobj_test("iris/train.txt", &args, 2, 6);
     
    }

    if args.len() > 2 && args[1] == "specimen" {

        user_dataobj_test(&args[2], &args, 3, args.len()); 

    }

    let mut _example_obj = DataObject {
        id: 0,
        classifier: "?".to_string(),
        data: vec![4.7,3.2,1.6,0.2],
    };

}
