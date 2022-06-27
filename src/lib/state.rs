use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug,Copy,Clone,Serialize)]
pub struct State {
    pub client:u16,
    pub available:f64,
    pub held:f64,
    pub total:f64,
    pub locked:bool,
}
impl State{
    pub fn update_total(&mut self){
        self.total = self.available + self.held;
    }
    pub fn deposit_amount(&mut self,amount:f64){
        //println!("Depositing {}",amount);
        self.available += amount;
        self.update_total();
    }
    pub fn withdraw_amount(&mut self,amount:f64){
        if self.available - amount < 0.0{
            return Err("Balance cannot go negative, cancelling transaction");
        }else{
            self.available = self.available - amount;
        }
        self.update_total();
    }
    pub fn dispute(&mut self, tx:u32, previous_transactions:&HashMap<u32,f64>,mut disputed_transactions:&Vec<u32>){
        if disputed_transactions.contains(&tx){
            return Err("This transaction is already disputed!");
        }else{
            let disputed_amount = previous_transactions.get(&tx).unwrap();
            self.available = self.available - disputed_amount;
            self.held = self.held + disputed_amount;
        }
        self.update_total();  
    }
    pub fn resolve(&mut self, tx:u32, previous_transactions:&HashMap<u32,f64>, mut disputed_transactions:&Vec<u32>){
        //Check it is disputed
        if disputed_transactions.contains(&tx){
            let disputed_amount = previous_transactions.get(&tx).unwrap();
            self.available = self.available + disputed_amount;
            self.held = self.held - disputed_amount;
        }else{
            return Err("This transaction is not disputed!");
        }
        self.update_total();
    }
    pub fn chargeback(&mut self, tx:u32, previous_transactions:&HashMap<u32,f64>, mut disputed_transactions:&Vec<u32>){
        if disputed_transactions.contains(&tx){
            let disputed_amount = previous_transactions.get(&tx).unwrap();
            self.held = self.held - disputed_amount;
            self.locked = true;
        }else{
            return Err("This transaction is not disputed!");
        }
        self.update_total();
    }
}
