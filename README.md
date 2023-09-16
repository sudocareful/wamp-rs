# wamp-rs
Wamp Client library. Not stable, subject to changes at any time. It makes use of the [wamp-helpers](https://github.com/ibotva/wamp-parser-rs.git) library.

# Usage
```rust

#[tokio::main]
async fn main() {
    let mut binding = WebSocket::builder();

    let options = binding
        .add_subprotocol("wamp.json");
    let mut client = WampClient::connect("wss://chat.co/", options).await.unwrap();

    client.send(Hello { 
        realm: "co.fun.chat.ifunny".to_string(), 
        details: json::object! {
            roles: {
                subscriber: {},
                caller: {},
                callee: {},
                publisher: {}
            },
            authmethods: ["ticket"]
        }
    }).await.unwrap();


    client.on(Events::Challenge(Box::new(|mut ctx, _challenge| {
        let _ = ctx.send(Authenticate {
            signature: dotenv!("BEARER").to_string(),
            details: json::object! {}
        }).unwrap();
        ctx
    })));

    client.on(Events::Welcome(Box::new(|mut ctx, welcome| {
        let auth_id = welcome.details["authid"].as_str().unwrap().to_string();
        let nickname = welcome.details["attributes"]["nick"].as_str();
        if let Some(nick) = nickname {
            println!("Logged in as {nick}");
        }

        let _ = ctx.subscribe(
            Subscribe {
                request: inc(),
                options: json::object! {},
                topic: format!("co.fun.chat.user.{auth_id}.chats")
            },
            Box::new(|ctx, subscribed| {
                // Listening for chats
                println!("{:#?}", subscribed);
                println!("Listening for chats...");
                ctx
            }),
            Box::new(|ctx, _event| {
                println!("Received chat...");
                ctx
            })
        );


        ctx
    })));

    client.loop_messages().await;
}
```