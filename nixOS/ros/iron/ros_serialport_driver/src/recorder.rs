//TODO: mmf recorder
use chrono::Utc;

use log::{info, trace };

 pub struct Recorder {
    buffer: Option<Vec<u8>>,
    path: String,
}

impl Recorder {
    pub fn new(path: String) -> Result<Recorder, String> {
    //TODO: ensure dir  exists 
    info!("Initialized at directory: {}", path);
    //TODO: how to use DmaStream or DmaFile
        return Ok(Recorder {
            path: path.to_owned(),
            buffer: None
        });
    }
    pub fn create_buffer(&mut self, size: usize) -> Result<&mut [u8], String> {
        // TODO: improve performance?????
        let filename = format!("{}/{}", self.path, Recorder::generate_name());
        
        // TODO: change in memory to generate filename and create the buffer to file directly
        let buffer = vec![0;size];
        assert!(size == buffer.len(), "Failed creating buffer with size: {} at: {}", size, filename);

        self.buffer = Some(buffer);
        trace!("new buffer: size:{} path:{}",size, filename);
        return Ok(self.buffer.as_mut().unwrap())
    }

    //TODO: the buffer filenme is based on time that should be discarded... the importance is the delta form when buffer was created and now
    //with that said, the timing here should be ignored as system time (ROS2) will be in the protocol written to the buffer, TBD
    // Generate name - year/month/day/hour_minute_second-miliseconds
    pub fn generate_name() -> String {
        return Utc::now().format("%Y/%m/%d/%H_%M_%S-%f_raw.dat").to_string();
    }
}
