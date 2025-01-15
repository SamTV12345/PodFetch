use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use actix_web::web::Query;
use actix_ws::Message;
use futures_util::StreamExt;
use sha256::Sha256Digest;

#[get("/socket.io/")]
pub async fn handle_socket_io(req: HttpRequest, body: web::Payload) ->
Result<HttpResponse, Error> {
    handle_request(&req, body)
}

#[derive(Serialize)]
pub struct Connect {
    data: ConnectPayload,
    r#type: i32,
}

#[derive(Serialize)]
struct ConnectPayload {
    sid: String,
    upgrades: Vec<String>,
    ping_interval: i32,
    ping_timeout: i32,
}

pub fn handle_request(req: &HttpRequest, body: web::Payload) ->
Result<HttpResponse, Error> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    let connect = ConnectPayload{
        sid: "rmgyyNpGHRoE_f5bRoA".to_string(),
        upgrades: vec![],
        ping_interval: 25000,
        ping_timeout: 20000
    };

    let conn = Connect {
        data: connect,
        r#type: 0
    };

    let resp = serde_json::to_string(&conn).unwrap();


    actix_web::rt::spawn(async move {
        session.text(resp).await.unwrap();
        while let rs = msg_stream.next().await {
            println!("Msg: {:?}", rs);
            match rs.unwrap().unwrap(){
                Message::Ping(bytes) => {
                    println!("Got ping");
                    if session.pong(&bytes).await.is_err() {
                        return;
                    }
                },
                Message::Binary(bytes)=>{
                    println!("Got binary");

                },
                Message::Text(msg) => {
                    println!("Got text");
                    //let parser_state = ParserState::default();
                    //let parser = Parser::decode_str(&parser_state, );
                    //println!("{:?}",parser);
                },
                Message::Continuation(c) => {
                    println!("Got continuation");
                }
                Message::Pong(p) => {
                    println!("Got pong");
                }
                Message::Close(c) => {
                    println!("Got close {:?}",c);
                }
                Message::Nop => {
                    println!("Got nop");
                }
            }
        }
        println!("Closing session");
        let _ = session.close(None).await;
    });
    Ok(response)
}