
//TODO: mmf recorder

use log::{trace, info};

pub struct Recorder {
    buffer: Option<Vec<u8>>,
    path: String,
}

impl Recorder {
    pub fn new(path: String) -> Result<Recorder, String> {
    //TODO: ensure dir exists 
    info!("Initialized at directory: {}", path);
    //TODO: how to use DmaStream or DmaFile
        return Ok(Recorder {
            path: path.to_owned(),
            buffer: None
        });
    }
    pub fn create_buffer(&mut self, size: usize) -> Result<&mut [u8], String> {

        //TODO: combine path with year/month/day
        let filename = format!("{}/{}", self.path, Recorder::generate_filename());
        
        // TODO: change in memory to generate filename and create the buffer to file directly
        let buffer = vec![0;size];
        assert!(size == buffer.len(), "Failed creating buffer with size: {} at: {}", size, filename);

        self.buffer = Some(buffer);
        trace!("new buffer: size:{} path:{}/",size, filename);
        return Ok(self.buffer.as_mut().unwrap())
    }

    // pub fn flush() -> Result<(), String> {
    //    return Ok(());
    // }

    fn generate_filename() -> String {
        return "???".into();
    }
}
