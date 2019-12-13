use crate::{buffer::Buffer, ip, packet, protocol, util};
use std::error::Error;
use std::fmt;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Type {
    EchoReply = 0,
    DestUnreach = 3,
    SourceQuench = 4,
    Redirect = 5,
    Echo = 8,
    TimeExceeded = 11,
    ParamProblem = 12,
    Timestamp = 13,
    TimestampReply = 14,
    InfoRequest = 15,
    InfoReply = 16,
}

impl Type {
    pub fn from_u8(n: u8) -> Option<Type> {
        Some(if n == Type::EchoReply as u8 {
            Type::EchoReply
        } else if n == Type::DestUnreach as u8 {
            Type::DestUnreach
        } else if n == Type::SourceQuench as u8 {
            Type::SourceQuench
        } else if n == Type::Redirect as u8 {
            Type::Redirect
        } else if n == Type::Echo as u8 {
            Type::Echo
        } else if n == Type::TimeExceeded as u8 {
            Type::TimeExceeded
        } else if n == Type::ParamProblem as u8 {
            Type::ParamProblem
        } else if n == Type::Timestamp as u8 {
            Type::Timestamp
        } else if n == Type::TimestampReply as u8 {
            Type::TimestampReply
        } else if n == Type::InfoRequest as u8 {
            Type::InfoRequest
        } else if n == Type::InfoReply as u8 {
            Type::InfoReply
        } else {
            return None;
        })
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Type::EchoReply => "Echo Reply",
                Type::DestUnreach => "Destination Unreachable",
                Type::SourceQuench => "Source Quench",
                Type::Redirect => "Redirect",
                Type::Echo => "Echo",
                Type::TimeExceeded => "Time Exceeded",
                Type::ParamProblem => "Parameter Problem",
                Type::Timestamp => "Timestamp",
                Type::TimestampReply => "Timestamp Reply",
                Type::InfoRequest => "Information Request",
                Type::InfoReply => "Information Reply",
            }
        )
    }
}

// for Unreach
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CodeUnreach {
    Net = 0,
    Host = 1,
    Proto = 2,
    Port = 3,
    FragmentNeeded = 4,
    SourceRouteFailed = 5,
}

impl CodeUnreach {
    pub fn from_u8(n: u8) -> Option<CodeUnreach> {
        Some(if n == CodeUnreach::Net as u8 {
            CodeUnreach::Net
        } else if n == CodeUnreach::Host as u8 {
            CodeUnreach::Host
        } else if n == CodeUnreach::Proto as u8 {
            CodeUnreach::Proto
        } else if n == CodeUnreach::Port as u8 {
            CodeUnreach::Port
        } else if n == CodeUnreach::FragmentNeeded as u8 {
            CodeUnreach::FragmentNeeded
        } else if n == CodeUnreach::SourceRouteFailed as u8 {
            CodeUnreach::SourceRouteFailed
        } else {
            return None;
        })
    }
}

impl fmt::Display for CodeUnreach {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CodeUnreach::Net => "Net",
                CodeUnreach::Host => "Host",
                CodeUnreach::Proto => "Proto",
                CodeUnreach::Port => "Port",
                CodeUnreach::FragmentNeeded => "Fragment Needed",
                CodeUnreach::SourceRouteFailed => "Source Route Failed",
            }
        )
    }
}

// for Redirect
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CodeRedirect {
    Net = 0,
    Host = 1,
    TosNet = 2,
    TosHost = 3,
}

impl CodeRedirect {
    pub fn from_u8(n: u8) -> Option<CodeRedirect> {
        Some(if n == CodeRedirect::Net as u8 {
            CodeRedirect::Net
        } else if n == CodeRedirect::Host as u8 {
            CodeRedirect::Host
        } else if n == CodeRedirect::TosNet as u8 {
            CodeRedirect::TosNet
        } else if n == CodeRedirect::TosHost as u8 {
            CodeRedirect::TosHost
        } else {
            return None;
        })
    }
}

impl fmt::Display for CodeRedirect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CodeRedirect::Net => "Net",
                CodeRedirect::Host => "Host",
                CodeRedirect::TosNet => "Tos Net",
                CodeRedirect::TosHost => "Tos Host",
            }
        )
    }
}

// for TimeExceeded
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CodeExceeded {
    Ttl = 0,
    Fragment = 1,
}

impl CodeExceeded {
    pub fn from_u8(n: u8) -> Option<CodeExceeded> {
        Some(if n == CodeExceeded::Ttl as u8 {
            CodeExceeded::Ttl
        } else if n == CodeExceeded::Fragment as u8 {
            CodeExceeded::Fragment
        } else {
            return None;
        })
    }
}

impl fmt::Display for CodeExceeded {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CodeExceeded::Ttl => "Ttl",
                CodeExceeded::Fragment => "Fragment",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Code {
    Unreach(CodeUnreach),
    Redirect(CodeRedirect),
    Exceeded(CodeExceeded),
    Others(u8),
}

impl Code {
    fn from_u8(n: u8, type_: Type) -> Option<Code> {
        Some(match type_ {
            Type::DestUnreach => Code::Unreach(CodeUnreach::from_u8(n)?),
            Type::Redirect => Code::Redirect(CodeRedirect::from_u8(n)?),
            Type::TimeExceeded => Code::Exceeded(CodeExceeded::from_u8(n)?),
            _ => Code::Others(n),
        })
    }

    fn to_u8(&self) -> u8 {
        match self {
            Code::Unreach(code) => *code as u8,
            Code::Redirect(code) => *code as u8,
            Code::Exceeded(code) => *code as u8,
            Code::Others(n) => *n,
        }
    }
}

#[derive(Debug, Clone)]
struct IcmpFrame {
    pub type_: Type,
    pub code: Code,
    pub values: u32,
    pub sum: u16,
    pub payload: Buffer,
}

impl IcmpFrame {
    pub fn dump(&self) {
        eprintln!("type: {}", self.type_);
        eprintln!("code: {:?}", self.code);
        eprintln!("sum: {}", self.sum);
        eprintln!("{}", self.payload);
    }
}

impl packet::Packet<IcmpFrame> for IcmpFrame {
    fn from_buffer(mut buf: Buffer) -> Result<Self, Box<dyn Error>> {
        let n = buf.pop_u8("type")?;
        let type_ = Type::from_u8(n).ok_or(util::RuntimeError::new(format!(
            "{} can not be ICMP type.",
            n
        )))?;
        let n = buf.pop_u8("code")?;
        let code = Code::from_u8(n, type_).ok_or(util::RuntimeError::new(format!(
            "{} can not be ICMP code under {}",
            n, type_
        )))?;
        let sum = buf.pop_u16("sum")?;
        let values = buf.pop_u32("values")?;

        Ok(IcmpFrame {
            type_: type_,
            code: code,
            sum: sum,
            values: values,
            payload: buf,
        })
    }
    fn to_buffer(self) -> Buffer {
        let mut buffer = Buffer::new(64 + self.payload.0.len());
        buffer.push_u8(self.type_ as u8);
        buffer.push_u8(self.code.to_u8());
        buffer.push_u32(self.values);
        buffer.push_u16(self.sum);
        buffer.append(self.payload);
        buffer
    }
}

pub fn rx(
    packet: Buffer,
    src: &ip::Addr,
    _dst: &ip::Addr,
    interface: &ip::interface::Interface,
) -> Result<(), Box<dyn Error>> {
    use packet::Packet;
    let frame = IcmpFrame::from_buffer(packet)?;
    eprintln!(">>> icmp rx <<<");
    frame.dump();
    if frame.type_ == Type::Echo {
        self::tx(
            interface,
            Type::EchoReply,
            frame.code,
            frame.values,
            frame.payload,
            src,
        )?;
    }
    Ok(())
}

pub fn tx(
    interface: &ip::interface::Interface,
    type_: Type,
    code: Code,
    values: u32,
    payload: Buffer,
    dst: &ip::Addr,
) -> Result<(), Box<dyn Error>> {
    let frame: IcmpFrame = IcmpFrame {
        type_: type_,
        code: code,
        values: values,
        sum: util::calc_checksum(payload.clone(), 0),
        payload: payload,
    };
    use packet::Packet;
    let buf = frame.to_buffer();
    interface.tx(protocol::ProtocolType::Icmp, buf, dst)
}

pub fn length(dgram: &ip::dgram::Dgram) -> usize {
    (((dgram.version_header_length & 0x0f) << 2) + 8) as usize
}

pub struct IcmpProtocol {}

impl IcmpProtocol {
    pub fn new() -> Arc<dyn protocol::Protocol + Send + Sync> {
        Arc::new(IcmpProtocol {})
    }
}

impl protocol::Protocol for IcmpProtocol {
    fn type_(&self) -> protocol::ProtocolType {
        protocol::ProtocolType::Icmp
    }
    fn handler(
        &self,
        payload: Buffer,
        src: ip::Addr,
        dst: ip::Addr,
        interface: &ip::interface::Interface,
    ) -> Result<(), Box<dyn Error>> {
        self::rx(payload, &src, &dst, interface)
    }
}
