extern crate chrono;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate tokio_core;

use std::env::args;
use std::process::exit;

use chrono::{DateTime, Utc};
use futures::future::{Future, err};
use futures::stream::{Stream, once};
use hyper::{Method, Uri};
use hyper::client::{Client, Connect, Request};
use hyper::header::{Authorization, Bearer, ContentType};
use tokio_core::reactor::Core;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GitterUser {
    id: String,
    username: String,
    display_name: String,
    url: String,
    avatar_url: String,
    avatar_url_small: String,
    avatar_url_medium: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GitterRoom {
    id: String,
    name: String,
    one_to_one: bool,
    user: Option<GitterUser>,
    url: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GitterMessage {
    id: String,
    text: String,
    html: String,
    #[serde(rename = "sent")]
    sent_at: Option<DateTime<Utc>>,
    edited_at: Option<DateTime<Utc>>,
    #[serde(rename = "fromUser")]
    user: Option<GitterUser>,
    url: Option<String>,
}

#[derive(Serialize, Debug)]
struct SlackMessage {
    text: String,
    username: Option<String>,
    icon_url: Option<String>,
    mrkdwn: bool,
}

fn make_request(uri: Uri, access_token: &str) -> Request {
    let auth = Authorization(Bearer {
        token: access_token.to_owned()
    });
    let mut req = Request::new(Method::Get, uri);
    req.headers_mut().set(auth);
    return req;
}

#[derive(Debug)]
enum GitterError {
    ApiError(String),
    HttpError(::hyper::Error),
}

fn list_rooms<C: Connect>(
    client: &Client<C>,
    access_token: &str,
) -> Box<Future<Item=Vec<GitterRoom>, Error=GitterError>> {
    let url = "https://api.gitter.im/v1/rooms".parse::<Uri>().unwrap();
    let request = make_request(url, access_token);
    let f = client.request(request).map_err(
        GitterError::HttpError
    ).and_then(|response| {
        if ! response.status().is_success() {
            return Err(
                GitterError::ApiError(
                    format!("Gitter responded with {}", response.status())
                )
            );
        }
        Ok(response.body().concat2().map_err(GitterError::HttpError))
    }).flatten().and_then(|p| match ::serde_json::from_slice(&p) {
        Ok(v) => Ok(v),
        Err(_) => Err(
            GitterError::ApiError(String::from("failed to deserialize"))
        ),
    });
    Box::new(f)
}

fn stream_messages<C: Connect>(
    client: &Client<C>,
    access_token: &str,
    room: &GitterRoom
) -> Box<Stream<Item=GitterMessage, Error=GitterError>> {
    let url_str = format!("https://stream.gitter.im/v1/rooms/{}/chatMessages",
                          room.id);
    let url = match url_str.parse::<Uri>() {
        Ok(v) => v,
        Err(_) => {
            let e = GitterError::ApiError(
                format!("invalid room id: {:?}", room.id)
            );
            return Box::new(once(Err(e)));
        }
    };
    let request = make_request(url, access_token);
    let f = client.request(request).map_err(
        GitterError::HttpError
    ).and_then(|response| {
        if ! response.status().is_success() {
            return Err(
                GitterError::ApiError(
                    format!("Gitter responded with {}", response.status())
                )
            );
        }
        let mut buffer: Vec<u8> = Vec::new();
        let f = response.body().map_err(
            GitterError::HttpError
        ).filter_map(move |chunk| {
            let mut bf = &mut buffer;
            bf.extend(chunk.iter().cloned());
            match ::serde_json::from_slice(&bf) {
                Err(_) => None,
                Ok(message) => {
                    bf.clear();
                    Some(message)
                },
            }
        });
        Ok(f)
    }).flatten_stream();
    Box::new(f)
}

#[derive(Debug)]
enum SlackError {
    SerializeError(::serde_json::Error),
    HttpError(::hyper::Error),
}

fn post_message<C: Connect>(
    client: &Client<C>,
    slack_webhook_url: &Uri,
    message: &SlackMessage,
) -> Box<Future<Item=(), Error=SlackError>> {
    let mut request = Request::new(Method::Post, (*slack_webhook_url).clone());
    request.headers_mut().set(ContentType::json());
    let payload = match ::serde_json::to_vec(message) {
        Err(e) => return Box::new(err(SlackError::SerializeError(e))),
        Ok(v) => v,
    };
    request.set_body(payload);
    let f = client.request(request).map_err(SlackError::HttpError).and_then(|_|
        Ok(())
    );
    Box::new(f)
}

fn main() -> () {
    let (access_token, room_name, slack_webhook_url) = {
        let mut argv = args();
        match (argv.next(), argv.next(), argv.next(), argv.next()) {
            (_, Some(token), Some(room), Some(u)) => {
                match u.parse::<Uri>() {
                    Ok(uri) => (token, room, uri),
                    _ => {
                        eprintln!("Invalid URL: {}", u);
                        exit(1);
                    }
                }
            },
            (prog, _, _, _) => {
                eprintln!(
                    "usage: {} GITTER_ACCESS_TOKEN GITTER_ROOM \
SLACK_WEBHOOK_URL",
                    prog.unwrap_or(String::from("gitter-slack")),
                );
                exit(1)
            }
        }
    };
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let client = Client::configure()
        .connector(::hyper_tls::HttpsConnector::new(4, &handle).unwrap())
        .build(&handle);

    // Rooms
    let rooms_f = list_rooms(&client, access_token.as_str());
    let rooms = match core.run(rooms_f) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Unexpected error happened during listing rooms.");
            eprintln!("{:?}", e);
            exit(1)
        }
    };
    let room = match rooms.iter().find(|r| r.name == room_name) {
        Some(v) => v,
        _ => {
            eprintln!("Failed to find a room named {}.", room_name);
            exit(1)
        },
    };
    println!("Room ID: {}", room.id);

    // Stream
    loop {
        let message_stream = stream_messages(
            &client,
            access_token.as_str(),
            room
        );
        let post_stream = message_stream.and_then(|message| {
            println!("Message {}: {}", message.id, message.text);
            let (username, icon_url) = match message.user {
                Some(u) => {
                    let name = format!(
                        "\u{01f4f6} \u{2014} {} @{}",
                        u.display_name,
                        u.username,
                    );
                    (Some(name), Some(u.avatar_url))
                },
                None => (None, None),
            };
            let url = message.url.unwrap_or(
                format!("https://gitter.im/{}?at={}", room.url, message.id)
            );
            let text = format!("{} <{}|\u{2834}>", message.text, url);
            let slack_msg = SlackMessage {
                text: text,
                username: username,
                icon_url: icon_url,
                mrkdwn: true,
            };
            post_message(
                &client,
                &slack_webhook_url,
                &slack_msg
            ).map_err(|error| match error {
                SlackError::HttpError(e) => GitterError::HttpError(e),
                e => {
                    eprintln!("Unexpected error happened during posting \
messages to Slack.");
                    eprintln!("{:?}", e);
                    exit(1)
                }
            })
        }).for_each(|_| Ok(()));
        let result = core.run(post_stream);
        match result {
            Err(e) => {
                eprintln!(
                    "Unexpected error happened during streaming messages."
                );
                eprintln!("{:?}", e);
                exit(1)
            },
            Ok(_) => continue,
        }
    }
}
