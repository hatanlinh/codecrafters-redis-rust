use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum RespData {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Vec<u8>),
    Array(Vec<RespData>),
}

impl RespData {
    pub fn serialize(&self) -> Vec<u8> {
        let mut result = Vec::new();
        match self {
            RespData::SimpleString(str) => {
                result.push(b'+');
                result.extend_from_slice(str.as_bytes());
                result.extend_from_slice(b"\r\n");
            }
            RespData::Error(msg) => {
                result.push(b'-');
                result.extend_from_slice(msg.as_bytes());
                result.extend_from_slice(b"\r\n");
            }
            RespData::Integer(n) => {
                result.push(b':');
                result.extend_from_slice(n.to_string().as_bytes());
                result.extend_from_slice(b"\r\n");
            }
            RespData::BulkString(data) => {
                result.push(b'$');
                result.extend_from_slice(data.len().to_string().as_bytes());
                result.extend_from_slice(b"\r\n");
                result.extend_from_slice(data.as_slice());
                result.extend_from_slice(b"\r\n");
            }
            RespData::Array(arr) => {
                result.push(b'*');
                result.extend_from_slice(arr.len().to_string().as_bytes());
                result.extend_from_slice(b"\r\n");
                for elem in arr {
                    result.extend_from_slice(elem.serialize().as_slice());
                }
            }
        }
        result
    }
}

#[derive(Error, Debug)]
enum ParseError {
    #[error("Current data is incomplete")]
    Incomplete,

    #[error("Invalid data")]
    Invalid,
}

pub struct RespParser {
    buffer: Vec<u8>,
    cursor: usize,
}

impl RespParser {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            cursor: 0,
        }
    }

    pub fn feed(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    pub fn parse(&mut self) -> Result<Option<RespData>> {
        if self.cursor >= self.buffer.len() {
            return Ok(None);
        }

        match self.parse_value() {
            Ok(val) => Ok(Some(val)),
            Err(ParseError::Incomplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn parse_value(&mut self) -> Result<RespData, ParseError> {
        if self.cursor >= self.buffer.len() {
            return Err(ParseError::Incomplete);
        }

        match self.buffer[self.cursor] {
            b'+' => self.parse_simple_string(),
            b'-' => self.parse_error(),
            b':' => self.parse_integer(),
            b'$' => self.parse_bulk_string(),
            b'*' => self.parse_array(),
            _ => Err(ParseError::Invalid),
        }
    }

    fn parse_simple_string(&mut self) -> Result<RespData, ParseError> {
        self.cursor += 1;
        let str = self.read_line()?;
        Ok(RespData::SimpleString(str))
    }

    fn parse_error(&mut self) -> Result<RespData, ParseError> {
        self.cursor += 1;
        let msg = self.read_line()?;
        Ok(RespData::Error(msg))
    }

    fn parse_integer(&mut self) -> Result<RespData, ParseError> {
        self.cursor += 1;
        let line = self.read_line()?;
        let n = line.parse::<i64>().map_err(|_| ParseError::Invalid)?;
        Ok(RespData::Integer(n))
    }

    fn parse_bulk_string(&mut self) -> Result<RespData, ParseError> {
        self.cursor += 1;
        let line = self.read_line()?;
        let len: usize = line.parse().map_err(|_| ParseError::Invalid)?;

        if len == usize::MAX {
            // Null bulk string: $-1\r\n
            return Ok(RespData::BulkString(Vec::new()));
        }

        // Check if we have enough data
        if self.cursor + len + 2 > self.buffer.len() {
            return Err(ParseError::Incomplete);
        }

        let data = self.buffer[self.cursor..self.cursor + len].to_vec();
        self.cursor += len + 2; // data + \r\n

        Ok(RespData::BulkString(data))
    }

    fn parse_array(&mut self) -> Result<RespData, ParseError> {
        self.cursor += 1;
        let line = self.read_line()?;
        let len: usize = line.parse().map_err(|_| ParseError::Invalid)?;

        if len == usize::MAX {
            // Null array: *-1\r\n
            return Ok(RespData::Array(Vec::new()));
        }

        let mut elements = Vec::with_capacity(len);
        for _ in 0..len {
            let elem = self.parse_value()?;
            elements.push(elem)
        }

        Ok(RespData::Array(elements))
    }

    fn read_line(&mut self) -> Result<String, ParseError> {
        if let Some(pos) = self.find_crlf(self.cursor) {
            let line = String::from_utf8(self.buffer[self.cursor..pos].to_vec())
                .map_err(|_| ParseError::Invalid)?;
            self.cursor = pos + 2; // line + \r\n
            Ok(line)
        } else {
            Err(ParseError::Incomplete)
        }
    }

    fn find_crlf(&self, start: usize) -> Option<usize> {
        for i in start..self.buffer.len().saturating_sub(1) {
            if self.buffer[i] == b'\r' && self.buffer[i + 1] == b'\n' {
                return Some(i);
            }
        }
        None
    }
}
