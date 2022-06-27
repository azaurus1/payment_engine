use std::error::Error;
use std::ffi::OsString;
use std::io;
use std::process;
use std::collections::HashMap;
use std::env;
use std::fs::File;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize)]
struct Record {
    r#type: String,
    client: u16,
    tx: u32,
    #[serde(deserialize_with = "csv::invalid_option")]
    amount: Option<f64>,
}
#[derive(Debug,Copy,Clone,Serialize)]
struct State {
    client:u16,
    available:f64,
    held:f64,
    total:f64,
    locked:bool,
}
impl State{
    fn update_total(&mut self){
        self.total = self.available + self.held;
    }
    fn deposit_amount(&mut self,amount:f64){
        //println!("Depositing {}",amount);
        self.available += amount;
        self.update_total();
    }
    fn withdraw_amount(&mut self,amount:f64){
        if self.available - amount < 0.0{
            //println!("Balance cannot go negative, cancelling transaction");
        }else{
            //println!("Withdrawing {}",amount);
            self.available = self.available - amount;
        }
        self.update_total();
    }
    fn dispute(&mut self, tx:u32, previous_transactions:&HashMap<u32,f64>,mut disputed_transactions:&Vec<u32>){
        if disputed_transactions.contains(&tx){
            //println!("This transaction is already disputed!");
        }else{
            //Find amount in tx
            //println!("Dispute TX: {}",tx);
            let disputed_amount = previous_transactions.get(&tx).unwrap();
            //println!("Disputing tx: {}, Disputed Amount = {}",tx,disputed_amount);
            self.available = self.available - disputed_amount;
            self.held = self.held + disputed_amount;
        }
        self.update_total();  
    }
    fn resolve(&mut self, tx:u32, previous_transactions:&HashMap<u32,f64>, mut disputed_transactions:&Vec<u32>){
        //Check it is disputed
        if disputed_transactions.contains(&tx){
            let disputed_amount = previous_transactions.get(&tx).unwrap();
            self.available = self.available + disputed_amount;
            self.held = self.held - disputed_amount;
            //println!("Resolved transaction: {} for amount: {}",tx,disputed_amount);
        }else{
            //println!("Transaction is not disputed.");
        }
        self.update_total();
    }
    fn chargeback(&mut self, tx:u32, previous_transactions:&HashMap<u32,f64>, mut disputed_transactions:&Vec<u32>){
        if disputed_transactions.contains(&tx){
            let disputed_amount = previous_transactions.get(&tx).unwrap();
            self.held = self.held - disputed_amount;
            self.locked = true;
            //println!("Chargedback transaction: {} for amount: {}",tx,disputed_amount);
        }else{
            //println!("Transaction is not disputed.");
        }
        self.update_total();
    }
}

fn get_file_path() -> Result<OsString,Box<dyn Error>>{
    match env::args_os().nth(1){
        None => Err(From::from("expecting 1 argument, got none")),
        Some(file_path) => Ok(file_path),
    }
}

fn run() -> Result<(), Box<dyn Error>>{ 
    let mut current_state = HashMap::<u16,State>::new();
    let file_path = get_file_path()?;
    let file = File::open(file_path).unwrap();
    let mut rdr = csv::Reader::from_reader(file);
    let mut key:u16;
    let mut last_tx:u32 = 0;
    let mut disputed_transactions = Vec::<u32>::new();
    // last 500 non-dispute transactions stored in hashmap
    let mut previous_transactions = HashMap::<u32, f64>::with_capacity(500);
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
                available:0.0,
                held:0.0,
                total:0.0,
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
                _ => () //println!("Invalid type")
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
                _ =>() //println!("Invalid type")
            }
        }  
    }
    //println!("{:?}",current_state.values());
    let mut wtr= csv::Writer::from_writer(io::stdout());
    for (client,state) in &current_state {
        wtr.serialize(&state);
    }
    wtr.flush();
    
    Ok(())
    
}

fn main() {
    if let Err(err) = run() {
        println!("{}", err);
        process::exit(1);
    }
}