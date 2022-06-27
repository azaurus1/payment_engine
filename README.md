# payment_engine

run with "cargo run -- transactions.csv > accounts.csv"

Notes:

Handles:
*Deposits
*Withdrawals
*Disputes
*Resolutions
*Chargebacks
  
Tested against provided sample data in the transactions.csv in this repo

Errors in the csv are handled via serde, empty fields are deserialised with csv::invalid_option.


