use wexing;

fn main()
{
    let listener = wexing::net::TcpListener::bind("0.0.0.0:3333").unwrap();

    wexing::block_on(async move
    {
        loop
        {
            match listener.accept().await
            {
                Ok((mut stream, _addr)) =>
                {
                    let mut buf = String::new();
                    let _ = stream.read_to_string(&mut buf).await.unwrap();
                    let _ = stream.write(buf.as_bytes()).await;
                },
                Err(e) => { println!("{}", e) },
            }
        }
    });
}
