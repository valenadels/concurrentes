fn main() {
    //versión SINCRONICA
    use std::{net, thread};
    let listener = net::TcpListener::bind(address)?;
    for socket_result in listener.incoming() {
        let socket = socket_result?;
        let groups = chat_group_table.clone();
        thread::spawn(|| {
            log_error(serve(socket, groups)); //por cada conexion crea un thread
        });
    }

    //versión ASINCRÓNICA -> usar todo de la libreria async_std
    use async_std::task;
    let listener = net::TcpListener::bind(address).await?; //como es async, se invoca con await
    let mut new_connections = listener.incoming();
    while let Some(socket_result) = new_connections.next().await {
        //iterador async
        let socket = socket_result?;
        let groups = chat_group_table.clone();
        task::spawn(async {
            log_error(serve(socket, groups).await);
        });
    }
}
