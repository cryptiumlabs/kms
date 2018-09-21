use super::{Ed25519Signature, RemoteErr, Signature, TendermintSignable};
use bytes::BufMut;
use prost::{EncodeError, Message};

#[derive(Clone, PartialEq, Message)]
pub struct Heartbeat {
    #[prost(bytes, tag = "1")]
    pub validator_address: Vec<u8>,
    #[prost(sint64)]
    pub validator_index: i64,
    #[prost(sint64)]
    pub height: i64,
    #[prost(sint64)]
    pub round: i64,
    #[prost(sint64)]
    pub sequence: i64,
    #[prost(message)]
    pub signature: Option<Vec<u8>>,
}

pub const AMINO_NAME: &str = "tendermint/socketpv/SignHeartbeatRequest";

// TODO(ismail): the Request / Reply types should live in a separate module!
// (same for proposal and vote)

#[derive(Clone, PartialEq, Message)]
#[amino_name = "tendermint/socketpv/SignHeartbeatRequest"]
pub struct SignHeartbeatRequest {
    #[prost(message, tag = "1")]
    pub heartbeat: Option<Heartbeat>,
}

#[derive(Clone, PartialEq, Message)]
#[amino_name = "tendermint/socketpv/SignedHeartbeatReply"]
pub struct SignedHeartbeatReply {
    #[prost(message, tag = "1")]
    pub heartbeat: Option<Heartbeat>,
    #[prost(message, tag = "2")]
    pub err: Option<RemoteErr>,
}

impl TendermintSignable for SignHeartbeatRequest {
    // Get the amino encoded bytes; excluding the signature (even if it was set):
    fn sign_bytes<B>(&self, sign_bytes: &mut B) -> Result<bool, EncodeError>
    where
        B: BufMut,
    {
        let mut hbm = self.clone();
        if let Some(ref mut hbm) = hbm.heartbeat {
            hbm.signature = None
        }
        hbm.encode(sign_bytes)?;
        Ok(true)
    }
    fn set_signature(&mut self, sig: &Ed25519Signature) {
        if let Some(ref mut hb) = self.heartbeat {
            hb.signature = Some(sig.clone().into_vec());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;
    use std::error::Error;

    #[test]
    fn test_serializationuns_unsigned() {
        let addr = vec![
            0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4,
            0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35,
        ];
        let heartbeat = Heartbeat {
            validator_address: addr,
            validator_index: 1,
            height: 15,
            round: 10,
            sequence: 30,
            signature: None,
        };
        let mut got = vec![];
        let _have = SignHeartbeatRequest {
            heartbeat: Some(heartbeat),
        }.encode(&mut got);
        let want = vec![
            0x24, 0xbf, 0x58, 0xca, 0xef, 0xa, 0x1e, 0xa, 0x14, 0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86,
            0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4, 0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35,
            0x10, 0x2, 0x18, 0x1e, 0x20, 0x14, 0x28, 0x3c,
        ];

        assert_eq!(got, want)
    }

    #[test]
    fn test_serializationuns_withoutaddr() {
        // identical to above but without validator_adress:
        let heartbeat = Heartbeat {
            validator_address: vec![],
            validator_index: 1,
            height: 15,
            round: 10,
            sequence: 30,
            signature: None,
        };
        let msg = SignHeartbeatRequest {
            heartbeat: Some(heartbeat),
        };

        let mut got = vec![];
        let _have = msg.encode(&mut got);
        let want = vec![
            0xe, 0xbf, 0x58, 0xca, 0xef, 0xa, 0x8, 0x10, 0x2, 0x18, 0x1e, 0x20, 0x14, 0x28, 0x3c,
        ];

        assert_eq!(got, want)
    }

    #[test]
    fn test_deserialization() {
        let addr = vec![
            0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4,
            0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35,
        ];
        let hb = Heartbeat {
            validator_address: addr,
            validator_index: 1,
            height: 15,
            round: 10,
            sequence: 30,
            signature: None,
        };
        let want = SignHeartbeatRequest {
            heartbeat: Some(hb),
        };

        let data = vec![
            0x24, 0xbf, 0x58, 0xca, 0xef, 0xa, 0x1e, 0xa, 0x14, 0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86,
            0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4, 0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35,
            0x10, 0x2, 0x18, 0x1e, 0x20, 0x14, 0x28, 0x3c,
        ];

        match SignHeartbeatRequest::decode(&data) {
            Err(err) => assert!(false, err.description().to_string()),
            Ok(have) => assert_eq!(have, want),
        }
    }
}
