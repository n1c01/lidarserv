use std::sync::Arc;
use pasture_core::layout::{PointAttributeDefinition, PointLayout};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::ToSocketAddrs;
use tokio::sync::broadcast::Receiver;
use tokio::sync::Mutex;
use lidarserv_common::geometry::coordinate_system::CoordinateSystem;
use crate::index::write_query::WriteQuery;
use crate::net::LidarServerError;
use crate::net::protocol::connection::Connection;
use crate::net::protocol::messages::PointDataCodec;

struct Inner {
    connection: Connection<OwnedWriteHalf>,
    last_ack: u64,
    ack_after: u64,
}
//todo capture device client anschauen
pub struct UpdateClient {
    pub read: ReadServer,//todo vermutlich nicht gebraucht
    pub write: WriteClient,
}

pub struct ReadServer {
    inner: Arc<Mutex<Inner>>,
}

pub struct WriteClient {
    inner: Arc<Mutex<Inner>>,
    codec: PointDataCodec,
    coordinate_system: CoordinateSystem,
    attributes: Vec<PointAttributeDefinition>,
    point_layout: PointLayout,
}

impl UpdateClient {
    pub async fn connect<A>(_addr: A, shutdown: &mut Receiver<()>) -> Result<Self, LidarServerError>
    where
        A: ToSocketAddrs,
    {
        todo!("create the new connection {:?}",shutdown);
    }
}

impl WriteClient {
    pub async fn write_query(
        &self,
        query: WriteQuery,
    ) -> Result<(), LidarServerError>
    {
        todo!("write the query to the server {:?}", query);

    }

}

impl ReadServer{
    async fn read_query(
        &self,
        query: WriteQuery,
    ) -> Result<(), LidarServerError>
    {
        todo!("read the query from the client {:?}", query);

        //todo!("safe the data to the server");
    }
}
