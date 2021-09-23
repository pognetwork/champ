

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
    type Receiver;

    async fn send(&self);
    async fn receive() -> Self::Receiver;
}

#[tonic::async_trait]
pub trait UdpSerializable {
    type Receiver;

    async fn send(&self);
    async fn receive() -> Self::Receiver;
}

pub struct Test {
    name: String,
    age: i32,
}

#[tonic::async_trait]
impl TcpSerializable for Test {
    type Receiver = Test;

    async fn send(&self) {
        println!("{}", self.age)
    }

    async fn receive() -> Self::Receiver {
        Test { name: String::from("Hello, World"), age: 42 }
    }
}
