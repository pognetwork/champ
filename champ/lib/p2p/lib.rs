

pub async fn send_via_tcp<T: TcpSerializable>() {
    //Not Implemented
}

pub async fn receive_via_tcp<T: TcpSerializable>() {
    //Not Implemented
}

pub async fn send_via_udp<T: UdpSerializable>() {
    //Not Implemented
}

pub async fn receive_via_udp<T: UdpSerializable>() {
    //Not Implemented
}

#[tonic::async_trait]
pub trait TcpSerializable {
    async fn send() {}
    async fn receive() {}
}

#[tonic::async_trait]
pub trait UdpSerializable {
    async fn send() {}
    async fn receive() {}
}

struct Test {
    name: String,
    age: i32,
}

#[tonic::async_trait]
impl TcpSerializable for Test {
    async fn send() {
        //do some Async stuff
    }

    async fn receive() {
        //do some Async stuff
    }
}