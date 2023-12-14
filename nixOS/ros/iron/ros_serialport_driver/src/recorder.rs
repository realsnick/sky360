struct Recorder {
    path: string,
}

impl Recorder {
    pub fn create(path: &str) -> Result<Recorder, String> {
    //TODO: how to use DmaStream or DmaFile
    //TODO: ensure dir exists 

        return Ok(Recorder {
            path: path.to_string(),
        });
    }

    pub fn createBuffer(size: usize) -> Result<mut Buffer<u8>,String> {
        let buffer: Vec<u8> = vec![0; buffer_size];

        return Ok(buffer);
    }

    pub fn flush() -> Result<(), String> {
       return Ok(());
    }
}
