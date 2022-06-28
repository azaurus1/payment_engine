# payment_engine

run with "cargo run -- transactions.csv > accounts.csv"

Notes:

Handles:
* Deposits
* Withdrawals
* Disputes
* Resolutions
* Chargebacks
  
Tested against provided sample data in the transactions.csv in this repo

Errors in the csv are handled via serde, empty fields are deserialised with csv::invalid_option.

It is possible to change the reader from the file to stdin, which would allow for the data to be streamed from TCP streams.

What could be improved or some issues:
* Using the ATM assumption, only a certain amount of previous transactions would be kept and this is currently not implemented in the code, ideally a queue of x capacity, and every new item at capacity will pop off the oldest transaction.
* Unit-testing, there are no unit tests implemented, the run function could be improved so that unit tests with different missing or incorrect record fields can be tested with "cargo test" 


