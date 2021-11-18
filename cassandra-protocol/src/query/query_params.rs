use derive_more::Constructor;
use std::convert::TryFrom;
use std::io::Cursor;

use crate::consistency::Consistency;
use crate::frame::Serialize;
use crate::query::query_flags::QueryFlags;
use crate::query::query_values::QueryValues;
use crate::types::value::Value;
use crate::types::{CBytes, CIntShort};
use crate::Error;

/// Parameters of Query for query operation.
#[derive(Debug, Default, Clone)]
pub struct QueryParams {
    /// Cassandra consistency level.
    pub consistency: Consistency,
    /// Were values provided with names
    pub with_names: bool,
    /// Array of values.
    pub values: Option<QueryValues>,
    /// Page size.
    pub page_size: Option<i32>,
    /// Array of bytes which represents paging state.
    pub paging_state: Option<CBytes>,
    /// Serial `Consistency`.
    pub serial_consistency: Option<Consistency>,
    /// Timestamp.
    pub timestamp: Option<i64>,
    /// Is the query idempotent.
    pub is_idempotent: bool,
    /// Query keyspace. If not using a global one, setting it explicitly might help the load
    /// balancer use more appropriate nodes. Note: prepared statements with keyspace information
    /// take precedence over this field.
    pub keyspace: Option<String>,
    /// The token to use for token-aware routing. A load balancer may use this information to
    /// determine which nodes to contact. Takes precedence over `routing_key`.
    pub token: Option<Murmur3Token>,
    /// The partition key to use for token-aware routing. A load balancer may use this information
    /// to determine which nodes to contact. Alternative to `token`. Note: prepared statements
    /// with bound primary key values take precedence over this field.
    pub routing_key: Option<Vec<Value>>,
}

impl QueryParams {
    fn flags(&self) -> QueryFlags {
        let mut flags = QueryFlags::empty();

        if self.values.is_some() {
            flags.insert(QueryFlags::VALUE);
        }

        if self.with_names {
            flags.insert(QueryFlags::WITH_NAMES_FOR_VALUES);
        }

        if self.page_size.is_some() {
            flags.insert(QueryFlags::PAGE_SIZE);
        }

        if self.paging_state.is_some() {
            flags.insert(QueryFlags::WITH_PAGING_STATE);
        }

        if self.serial_consistency.is_some() {
            flags.insert(QueryFlags::WITH_SERIAL_CONSISTENCY);
        }

        if self.timestamp.is_some() {
            flags.insert(QueryFlags::WITH_DEFAULT_TIMESTAMP);
        }

        flags
    }
}

impl Serialize for QueryParams {
    fn serialize(&self, cursor: &mut Cursor<&mut Vec<u8>>) {
        let consistency: CIntShort = self.consistency.into();
        consistency.serialize(cursor);

        let flag_bits = self.flags().bits();
        flag_bits.serialize(cursor);

        if let Some(values) = &self.values {
            let len = values.len() as CIntShort;
            len.serialize(cursor);
            values.serialize(cursor);
        }

        if let Some(page_size) = self.page_size {
            page_size.serialize(cursor);
        }

        if let Some(paging_state) = &self.paging_state {
            paging_state.serialize(cursor);
        }

        if let Some(serial_consistency) = self.serial_consistency {
            let serial_consistency: CIntShort = serial_consistency.into();
            serial_consistency.serialize(cursor);
        }

        if let Some(timestamp) = self.timestamp {
            timestamp.serialize(cursor);
        }
    }
}

/// A token on the ring. Only Murmur3 tokens are supported for now.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default, Debug, Hash, Constructor)]
pub struct Murmur3Token {
    pub value: i64,
}

impl TryFrom<String> for Murmur3Token {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value
            .parse()
            .map_err(|error| format!("Error parsing token: {}", error).into())
            .map(Murmur3Token::new)
    }
}
