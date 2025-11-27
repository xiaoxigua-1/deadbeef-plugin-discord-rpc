use urlencoding::encode;

use crate::{
    deadbeef::DB_functions_t,
    error::{Error, Result},
};

impl DB_functions_t {
    pub fn http_get(&self, url: &str) -> Result<String> {
        let client = self.fopen(&url)?;
        if client.is_null() {
            return Err(Error::HttpGetFailed(url.to_string()));
        }
        let length = self.fgetlength(client)?;
        if length < 0 {
            self.fclose(client)?;

            return Err(Error::HttpGetFailed(url.to_string()));
        }
        let mut buffer: Vec<u8> = vec![0; length as usize];

        self.trace(format!("HTTP GET: {} ({} bytes)", url, length));
        self.fread(buffer.as_mut_ptr() as *mut _, 1, length as usize, client)?;
        self.fclose(client)?;

        Ok(String::from_utf8_lossy(&buffer).to_string())
    }
}
