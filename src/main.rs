
use postgres::{Client, Error, NoTls};

extern crate nalgebra as na;

mod fact;
mod dp;
mod lr;
mod nnlayer;
mod nn;



type VecVec64 = Vec<Vec<Option<f64>>>;


use rand::thread_rng;
use rand::seq::SliceRandom;




fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}



fn query_col_string(query : &str, idx : usize, data : &mut Vec<Vec<String>>) -> Result<(), Error> {
    let url = "postgresql://postgres:postgres@localhost:5432/postgres";
    let mut conn = Client::connect(url, NoTls).unwrap();

    for row in conn.query(query, &[])? {
        let mut v : Vec<String> = Vec::new();
        v.push(row.get(idx));
        data.push(v);
    }
    Ok(())
}




fn query_vec(query : &str, data : &mut Vec<Vec<f64>>) -> Result<(), Error> {
    let url = "postgresql://postgres:postgres@localhost:5432/postgres";
    let mut conn = Client::connect(url, NoTls).unwrap();

    for row in conn.query(query, &[])? {
        let mut v : Vec<f64> = Vec::new();
        for i in 0..row.len() {
            v.push(row.get(i));
        }
        data.push(v);
    }
    Ok(())
}






fn query_vec_range(query : &str, i1 : usize, i2 : usize, data : &mut Vec<Vec<f64>>) -> Result<(), Error> {
    let url = "postgresql://postgres:postgres@localhost:5432/postgres";
    let mut conn = Client::connect(url, NoTls).unwrap();

    let mut i2i = i2;

    for row in conn.query(query, &[])? {
        if i2i > row.len() {
            i2i = row.len();
        }
        let mut v : Vec<f64> = Vec::new();
        for i in i1..i2i {
            v.push(row.get(i));
        }
        data.push(v);
    }
    Ok(())
}



fn split_dataset(data    : Vec<Vec<f64>>, yid     : usize,
                 x_train : &mut Vec<Vec<f64>>, y_train : &mut Vec<f64>,
                 x_cv    : &mut Vec<Vec<f64>>, y_cv    : &mut Vec<f64>,
                 x_test  : &mut Vec<Vec<f64>>, y_test  : &mut Vec<f64>) {

    let mut _size_train = 0.60;
    let mut _size_cv = 0.20;
    let mut _size_test = 0.20;

    let mut i : i64 = 0;
    let split_train : i64 = (data.len() as f64 * _size_train) as i64;
    let split_cv : i64 = (data.len() as f64 * (_size_train + _size_cv)) as i64;
    for n in data {
        if i < split_train {
            x_train.push(n[0..yid].to_vec());
            y_train.push(n[yid].clone());
        } else if i < split_cv {
            x_cv.push(n[0..yid].to_vec());
            y_cv.push(n[yid].clone());
        } else {
            x_test.push(n[0..yid].to_vec());
            y_test.push(n[yid].clone());
        }
        i = i + 1;
    }
}








fn logistic_regression(db : &str, class : bool) {

    let mut query = String::new();
    query.push_str("SELECT * from ");
    query.push_str(db);


    let mut data: Vec<Vec<f64>> = vec![];

    if db == "nki" {

        let res = query_vec_range(&query, 2, 20, &mut data);
        println!("{:?}", res);

    } else if db == "sn" {

        let res = query_vec(&query, &mut data);
        println!("{:?}", res);

    }

    println!("{}", db);
    println!("{}", data[0][0]);



    let _epsilon = 1.0;
    let _noise_scale = 1.0;
    let _data_norm = 1000.0;

    let epochs = 5;
    let batch = 1000;
    let epochs_dp = 5;
    let batch_dp = 1000;
    let nfeat = 20;
    let nclass = 10;

    
    // Shuffle input
    data.shuffle(&mut thread_rng());

    // Split dataset into train, cross validation and test
    let mut x_train : Vec<Vec<f64>> = vec![];
    let mut y_train : Vec<f64> = vec![];
    let mut x_cv : Vec<Vec<f64>> = vec![];
    let mut y_cv : Vec<f64> = vec![];
    let mut x_test : Vec<Vec<f64>> = vec![];
    let mut y_test : Vec<f64> = vec![];

    split_dataset(data, 20, &mut x_train, &mut y_train, &mut x_cv, &mut y_cv, &mut x_test, &mut y_test);

    let mut lr = lr::LogisticRegression::new(epochs, batch, nfeat, nclass, 0.01, 0.001, true, false);

    lr.fit(x_train.clone(), y_train.clone(), epochs as usize, batch as usize);
    lr.test(x_train.clone(), y_train.clone());
    let loss1 = lr.get_loss();

    lr.reset();

    lr.enable_dp(true, _epsilon, _noise_scale, _data_norm);
    lr.fit(x_train.clone(), y_train.clone(), epochs_dp as usize, batch_dp as usize);
    lr.test(x_train.clone(), y_train.clone());
    let loss2 = lr.get_loss();

    println!("{}", loss1);
    println!("{}", loss2);

}



fn neural_network(db : &str, topology : Vec<usize>) {

    let mut query = String::new();
    query.push_str("SELECT * from ");
    query.push_str(db);

    let mut data: Vec<Vec<f64>> = vec![];
    query_vec(&query, &mut data);


    let _epsilon = 1.0;
    let _noise_scale = 1.0;
    let _data_norm = 1000.0;

    let epochs = 5;
    let batch = 1000;
    let epochs_dp = 5;
    let batch_dp = 1000;
    let nfeat = 20;
    let nclass = 10;

    
    // Shuffle input
    data.shuffle(&mut thread_rng());

    // Split dataset into train, cross validation and test
    let mut x_train : Vec<Vec<f64>> = vec![];
    let mut y_train : Vec<f64> = vec![];
    let mut x_cv : Vec<Vec<f64>> = vec![];
    let mut y_cv : Vec<f64> = vec![];
    let mut x_test : Vec<Vec<f64>> = vec![];
    let mut y_test : Vec<f64> = vec![];

    split_dataset(data, 20, &mut x_train, &mut y_train, &mut x_cv, &mut y_cv, &mut x_test, &mut y_test);


    let size_li = x_train[0].len();
    let size_l1 = 50;
    let size_l2 = 25;
    let size_lo = 10;

    let mut _topology : Vec<usize> = vec![x_train[0].len()];
    for i in topology.clone() {
        _topology.push(i);
    }
    
    let mut _facts : Vec<String> = vec![];
    for i in 0..topology.len()-1 {
        facts.push("relu".to_string());
    }
    facts.push("softmax".to_string());


    let learning_rate = 0.005;

    let mut nna = nn::NN::new(_topology,
                              _facts, 
                              learning_rate, 
                              false);

    nna.enable_dp(true, 0.01, 1.0);

    nna.train(x_train, y_train, 1, 1, 1, 1);
    nna.test(x_test, y_test);
}







fn main() -> Result<(), Error> {

    let opt : usize = 3;

    if opt == 1{

        println!("Logistic Regression - Classifier - Tabular Data - SmartNoise/random");
        logistic_regression("sn", true);

    } else if opt == 2 {

        println!("Neural Network Classifier - Tabular Data - SmartNoise/random");
        neural_network("sn", vec![25,25,10]);

    } else if opt == 3 {

        println!("Logistic Regression - Continuous - Tabular Data - Kaggle/nki");
        logistic_regression("nki", false);

    } else if opt == 4 {

        println!("Neural Network Continuous - Tabular Data - Kaggle/nki");
        neural_network("nki", vec![25,25,10]);

    } else if opt == 5 {

        println!("Neural Network - Classifier - Image data");

    }


    /*
    let _epsilon = 3.0;
    let _noise_scale = 0.01;
    let _data_norm = 7.89;

    let epochs = 1000;
    let batch = 50;
    let nfeat = 4;
    let nclass = 3;

    // Read from Postgres database, same as spi does from within PGX
    // Data into VecVec<Option<f64>>>

    let url = "postgresql://postgres:postgres@localhost:5432/postgres";
    let mut conn = Client::connect(url, NoTls).unwrap();

    let mut sn : VecVec64 = vec![];

    for row in conn.query("SELECT * from iris", &[])? {
        sn.push(vec![row.get(0),
                     row.get(1),
                     row.get(2),
                     row.get(3),
                     row.get(4)]);
    }
    
    
    // Shuffle input
    sn.shuffle(&mut thread_rng());

    // Split dataset into train, cross validation and test
    let mut x_train : VecVec64 = vec![];
    let mut y_train : Vec<Option<f64>> = vec![];
    let mut x_cv : VecVec64 = vec![];
    let mut y_cv : Vec<Option<f64>> = vec![];
    let mut x_test : VecVec64 = vec![];
    let mut y_test : Vec<Option<f64>> = vec![];

    let mut size_cv = 0.00;
    let mut size_test = 0.00;
    let mut size_train = 1.0 - size_cv - size_test;

    let mut i : i64 = 0;
    let split_train : i64 = (sn.len() as f64 * size_train) as i64;
    let split_cv : i64 = (sn.len() as f64 * (size_train + size_cv)) as i64;
    for n in sn {
        if i < split_train {
            x_train.push(n[0..nfeat].to_vec());
            y_train.push(n[nfeat].clone());
        } else if i < split_cv {
            x_cv.push(n[0..nfeat].to_vec());
            y_cv.push(n[nfeat].clone());
        } else {
            x_test.push(n[0..nfeat].to_vec());
            y_test.push(n[nfeat].clone());
        }
        i = i + 1;
    }



    let mut lr = lr::LogisticRegression::new(epochs, batch, nfeat, nclass, 0.01, 0.001, false);

    lr.fit(x_train.clone(), y_train.clone(), epochs as usize, batch as usize);
    lr.test(x_train.clone(), y_train.clone());
    let loss1 = lr.get_loss();

    lr.reset();

    lr.enable_dp(true, _epsilon, _noise_scale, _data_norm);
    lr.fit(x_train.clone(), y_train.clone(), epochs_dp as usize, batch_dp as usize);
    lr.test(x_train.clone(), y_train.clone());
    let loss2 = lr.get_loss();
    */

    /*
    let mut lr = lr::LogisticRegression::new(epochs, batch, nfeat, nclass, 0.1, 0.01, false);
    lr.enable_dp(true, 1.0, 0.01, 1.0);

    lr.fit(x_train.clone(), y_train.clone());
    lr.test(x_train.clone(), y_train.clone());
    */






    Ok(())
}   

