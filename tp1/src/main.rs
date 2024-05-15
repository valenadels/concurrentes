use stack_exchange_processor::error::Error;
use stack_exchange_processor::error::Error::*;
use stack_exchange_processor::processor::process_data;

/// Number of arguments that the user must provide to the program
const N_ARGUMENTS: usize = 2;
/// Index of the number of threads in the arguments
const THREADS_IDX: usize = 1;
/// Data directory
const DATA_DIR: &str = "./data";

/// Main function that reads the program arguments and starts the processing
fn main() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();
    match process_data(parse_args(args)?, DATA_DIR) {
        Ok(sites) => {
            println!("{}", sites);
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Parse the program arguments
/// User must provide the number of threads to run the processor
/// Returns the number of threads to run the processor
/// If the number of arguments or threads are invalid, returns an error
fn parse_args(args: Vec<String>) -> Result<usize, Error> {
    if args.len() != N_ARGUMENTS {
        return Err(ParsingError(
            "Invalid number of arguments, please provide the number of threads to run the processor"
                .to_string(),
        ));
    }

    let n_threads = args[THREADS_IDX]
        .parse::<usize>()
        .map_err(|_| ParsingError("Invalid number of threads".to_string()))?;

    if n_threads == 0 {
        return Err(ParsingError(
            "Number of threads must be greater than 0".to_string(),
        ));
    }

    Ok(n_threads)
}
