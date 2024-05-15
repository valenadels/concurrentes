use crate::error::Error;

/// This function creates a Rayon thread pool with the given number of threads
/// If the thread pool can't be created, it returns an error
/// Otherwise, it returns the thread pool
///
/// # Arguments
/// num_threads: usize - The number of threads to create the pool
pub fn create_pool(num_threads: usize) -> Result<rayon::ThreadPool, Error> {
    match rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
    {
        Err(e) => Err(Error::ThreadPoolError(e.to_string())),
        Ok(pool) => Ok(pool),
    }
}
