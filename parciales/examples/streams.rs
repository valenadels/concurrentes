use futures::stream::{BoxStream, StreamExt};
async fn sum_with_reduce(stream: BoxStream<'_, i32>) -> i32 {
    let sum = stream.fold(0, |acc, item| async move { acc + item }).await;
    sum
}

#[tokio::main]
async fn main() {
    let stream = futures::stream::iter(1..=5);

    let sum = sum_with_reduce(stream.boxed()).await;

    println!("La suma de los números es: {}", sum);
}
