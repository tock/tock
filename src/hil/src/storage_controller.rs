/// Error codes are used to inform the Client if the command completed successfully
/// or whether there was an error and what type of error it was.
pub enum Error {
    CommandComplete,    /* Command Complete */
    LockE,              /* Lock Error (i.e. tried writing to locked page) */
    ProgE,              /* Program Error (i.e. incorrectly issued flash commands */
    LockProgE,          /* Lock and Program Error */
    ECC,                /* Error Correcting Code Error */
}

pub trait StorageController<'a>{
    fn storage_ready(&self) -> bool;
    fn set_client(&self, client: &'a Client); 
}

pub trait Client {
    //  Called upon a completed call
    fn command_complete(&self, err: Error);     
}
