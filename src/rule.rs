use conversion::{ToFfi, CopyToFfi};
use ffi;

use libc;

use std::mem;
use std::net::Ipv4Addr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Builder)]
pub struct FilterRule {
    action: RuleAction,
    #[builder(default)]
    direction: Direction,
    #[builder(default)]
    quick: bool,
    #[builder(default)]
    proto: Proto,
    #[builder(default)]
    af: AddrFamily,
    #[builder(default="Ipv4Addr::new(0, 0, 0, 0)")]
    from: Ipv4Addr,
    #[builder(default="Ipv4Addr::new(0, 0, 0, 0)")]
    to: Ipv4Addr,
}

impl FilterRule {
    // TODO(linus): Very ugly hack for now :(
    fn set_addr(addr: Ipv4Addr, pf_addr: &mut ffi::pfvar::pf_rule_addr) {
        unsafe {
            pf_addr.addr.type_ = ffi::pfvar::PF_ADDR_ADDRMASK as u8;
            pf_addr.addr
                .v
                .a
                .as_mut()
                .addr
                .pfa
                .v4
                .as_mut()
                .s_addr = addr.to_ffi();
            pf_addr.addr
                .v
                .a
                .as_mut()
                .mask
                .pfa
                .v4
                .as_mut()
                .s_addr = 0xffffffffu32;
        }
    }
}

impl CopyToFfi<ffi::pfvar::pf_rule> for FilterRule {
    fn copy_to(&self, pf_rule: &mut ffi::pfvar::pf_rule) -> ::Result<()> {
        pf_rule.action = self.action.to_ffi();
        pf_rule.direction = self.direction.to_ffi();
        pf_rule.quick = self.quick.to_ffi();
        pf_rule.af = self.af.to_ffi();
        pf_rule.proto = self.proto.to_ffi();
        Self::set_addr(self.from, &mut pf_rule.src);
        Self::set_addr(self.to, &mut pf_rule.dst);
        Ok(())
    }
}


/// Enum describing what should happen to a packet that matches a rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleAction {
    Pass,
    Drop,
}

impl ToFfi<u8> for RuleAction {
    fn to_ffi(&self) -> u8 {
        match *self {
            RuleAction::Pass => ffi::pfvar::PF_PASS as u8,
            RuleAction::Drop => ffi::pfvar::PF_DROP as u8,
        }
    }
}


/// Enum describing matching of rule towards packet flow direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Any,
    In,
    Out,
}

impl Default for Direction {
    fn default() -> Self {
        Direction::Any
    }
}

impl ToFfi<u8> for Direction {
    fn to_ffi(&self) -> u8 {
        match *self {
            Direction::Any => ffi::pfvar::PF_INOUT as u8,
            Direction::In => ffi::pfvar::PF_IN as u8,
            Direction::Out => ffi::pfvar::PF_OUT as u8,
        }
    }
}


// TODO(linus): Many more protocols to add. But this will do for now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Proto {
    Any,
    Tcp,
}

impl Default for Proto {
    fn default() -> Self {
        Proto::Any
    }
}

impl ToFfi<u8> for Proto {
    fn to_ffi(&self) -> u8 {
        match *self {
            Proto::Any => libc::IPPROTO_IP as u8,
            Proto::Tcp => libc::IPPROTO_TCP as u8,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AddrFamily {
    Any,
    Ipv4,
    Ipv6,
}

impl Default for AddrFamily {
    fn default() -> Self {
        AddrFamily::Any
    }
}

impl ToFfi<u8> for AddrFamily {
    fn to_ffi(&self) -> u8 {
        match *self {
            AddrFamily::Any => ffi::pfvar::PF_UNSPEC as u8,
            AddrFamily::Ipv4 => ffi::pfvar::PF_INET as u8,
            AddrFamily::Ipv6 => ffi::pfvar::PF_INET6 as u8,
        }
    }
}

// Port range representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Port {
    Any,
    One(u16, PortUnaryModifier),
    Range(u16, u16, PortRangeModifier),
}

impl CopyToFfi<ffi::pfvar::pf_port_range> for Port {
    fn copy_to(&self, pf_port_range: &mut ffi::pfvar::pf_port_range) -> ::Result<()> {
        match *self {
            Port::Any => {
                pf_port_range.op = ffi::pfvar::PF_OP_NONE as u8;
                pf_port_range.port[0] = 0;
                pf_port_range.port[1] = 0;
            }
            Port::One(port, modifier) => {
                pf_port_range.op = modifier.to_ffi();
                // convert port range to network byte order
                pf_port_range.port[0] = port.to_be();
                pf_port_range.port[1] = 0;
            }
            Port::Range(start_port, end_port, modifier) => {
                ensure!(start_port <= end_port,
                        ::ErrorKind::InvalidArgument("Lower port is greater than upper port."));
                pf_port_range.op = modifier.to_ffi();
                // convert port range to network byte order
                pf_port_range.port[0] = start_port.to_be();
                pf_port_range.port[1] = end_port.to_be();
            }
        }
        Ok(())
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortUnaryModifier {
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterOrEqual,
    LessOrEqual,
}

impl ToFfi<u8> for PortUnaryModifier {
    fn to_ffi(&self) -> u8 {
        match *self {
            PortUnaryModifier::Equal => ffi::pfvar::PF_OP_EQ as u8,
            PortUnaryModifier::NotEqual => ffi::pfvar::PF_OP_NE as u8,
            PortUnaryModifier::Greater => ffi::pfvar::PF_OP_GT as u8,
            PortUnaryModifier::Less => ffi::pfvar::PF_OP_LT as u8,
            PortUnaryModifier::GreaterOrEqual => ffi::pfvar::PF_OP_GE as u8,
            PortUnaryModifier::LessOrEqual => ffi::pfvar::PF_OP_LE as u8,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortRangeModifier {
    Exclusive,
    Inclusive,
    Except,
}

impl ToFfi<u8> for PortRangeModifier {
    fn to_ffi(&self) -> u8 {
        match *self {
            PortRangeModifier::Exclusive => ffi::pfvar::PF_OP_IRG as u8,
            PortRangeModifier::Inclusive => ffi::pfvar::PF_OP_RRG as u8,
            PortRangeModifier::Except => ffi::pfvar::PF_OP_XRG as u8,
        }
    }
}


// Implementations to convert types that are not ours into their FFI representation

impl ToFfi<u32> for Ipv4Addr {
    fn to_ffi(&self) -> u32 {
        unsafe { mem::transmute(self.octets()) }
    }
}

impl ToFfi<u8> for bool {
    fn to_ffi(&self) -> u8 {
        if *self { 1 } else { 0 }
    }
}

/// Safely copy a Rust string into a raw buffer. Returning an error if `src` could not be
/// copied to the buffer.

impl<T: AsRef<str>> CopyToFfi<[i8]> for T {
    fn copy_to(&self, dst: &mut [i8]) -> ::Result<()> {
        let src_i8: &[i8] = unsafe { mem::transmute(self.as_ref().as_bytes()) };

        ensure!(src_i8.len() < dst.len(),
                ::ErrorKind::InvalidArgument("String does not fit destination"));
        ensure!(!src_i8.contains(&0),
                ::ErrorKind::InvalidArgument("String has null byte"));

        dst[..src_i8.len()].copy_from_slice(src_i8);
        // Terminate ffi string with null byte
        dst[src_i8.len()] = 0;
        Ok(())
    }
}