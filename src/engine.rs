use crate::resp::RespData;
use crate::storage::Storage;

pub struct Engine {
    storage: Storage,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            storage: Storage::new(),
        }
    }

    pub fn handle_ping(&self) -> RespData {
        RespData::SimpleString(String::from("PONG"))
    }

    pub fn handle_echo(&self, args: &[RespData]) -> RespData {
        if args.len() < 1 {
            return RespData::Error(String::from(
                "Err wrong number of arguments for 'echo' command",
            ));
        }

        if matches!(args[0], RespData::BulkString(_)) {
            return args[0].clone();
        }

        RespData::Error(String::from(
            "ERR incorrect type of message data for 'echo'",
        ))
    }

    pub fn handle_set(&mut self, args: &[RespData]) -> RespData {
        if args.len() < 2 {
            return RespData::Error(String::from(
                "Err wrong number of arguments for 'set' command",
            ));
        }

        let mut expire_in_ms: Option<u64> = None;
        if let RespData::BulkString(key) = &args[0]
            && let RespData::BulkString(value) = &args[1]
        {
            if args.len() > 3
                && let RespData::BulkString(opt_name) = &args[2]
                && let RespData::BulkString(opt_val) = &args[3]
            {
                match opt_name.as_slice() {
                    opt if opt.eq_ignore_ascii_case(b"PX") => {
                        expire_in_ms = std::str::from_utf8(opt_val)
                            .ok()
                            .and_then(|s| s.parse::<u64>().ok());
                    }
                    _ => (),
                }
            }

            self.storage.set(key.clone(), value.clone(), expire_in_ms);
            return RespData::SimpleString(String::from("OK"));
        }

        RespData::Error(String::from("ERR incorrect type of arguments for 'set'"))
    }

    pub fn handle_get(&mut self, args: &[RespData]) -> RespData {
        if args.len() < 1 {
            return RespData::Error(String::from(
                "Err wrong number of arguments for 'get' command",
            ));
        }

        if let RespData::BulkString(key) = &args[0] {
            if let Some(value) = self.storage.get(key) {
                return RespData::BulkString(value.clone());
            }
            return RespData::BulkString(Vec::new());
        }

        RespData::Error(String::from("ERR incorrect type of arguments for 'get'"))
    }
}
