mod record;
mod state;

use std::error::Error;
use std::ffi::OsString;
use std::io;
use std::collections::HashMap;
use std::env;
use std::fs::File;

use crate::record::Record;
use crate::state::State;



pub fn get_file_path() -> Result<OsString,Box<dyn Error>>{
    match env::args_os().nth(1){
        None => Err(From::from("expecting 1 argument, got none")),
        Some(file_path) => Ok(file_path),
    }
}

pub fn run() -> Result<(), Box<dyn Error>>{ 
    let mut current_state = HashMap::<u16,State>::new();
    let file_path = get_file_path()?;
    let file = File::open(file_path).unwrap();
    let mut rdr = csv::Reader::from_reader(file);
    let mut key:u16;
    let mut last_tx:u32 = 0;
    let mut disputed_transactions = Vec::<u32>::new();
    // last 500 non-dispute transactions stored in hashmap
    let mut previous_transactions = HashMap::<u32, f64>::new();
    for result in rdr.deserialize(){
        let record: Record = result?;
        key = record.client;
        if (record.r#type == "deposit") || (record.r#type=="withdrawal"){
            if record.tx < last_tx{
                //println!("Can't have a non-dispute related tx older than most recent tx {} {}",record.tx,record.r#type);
                continue;
            }
            previous_transactions.insert(record.tx, record.amount.unwrap());
        }
        
        last_tx = record.tx;
        if !(current_state.contains_key(&key)){
            let mut state = State {
                client:record.client,
                available:0.0000,
                held:0.0000,
                total:0.0000,
                locked:false
            };
            match record.r#type.as_str() {
                "deposit" => state.deposit_amount(record.amount.unwrap()),
                "withdrawal" => state.withdraw_amount(record.amount.unwrap()),
                "dispute" => {
                    state.dispute(record.tx,&previous_transactions,&disputed_transactions);
                    disputed_transactions.push(record.tx);
                },
                "resolve" => state.resolve(record.tx, &previous_transactions, &disputed_transactions),
                "chargeback" => state.chargeback(record.tx, &previous_transactions, &disputed_transactions),
                _ => () 
            }
            current_state.insert(key, state);    
        }else{
            let state = current_state.get_mut(&key).unwrap();
            match record.r#type.as_str() {
                "deposit" => state.deposit_amount(record.amount.unwrap()),
                "withdrawal" => state.withdraw_amount(record.amount.unwrap()),
                "dispute" => {
                    state.dispute(record.tx,&previous_transactions,&disputed_transactions);
                    disputed_transactions.push(record.tx);
                },
                "resolve" => state.resolve(record.tx, &previous_transactions, &disputed_transactions),
                "chargeback" => state.chargeback(record.tx, &previous_transactions, &disputed_transactions),
                _ => ()
            }
        }  
    };
    let mut wtr= csv::Writer::from_writer(io::stdout());
    for (client,state) in &current_state {
        wtr.serialize(&state);
    }
    wtr.flush();
    
    Ok(())
    
}