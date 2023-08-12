use rocket::Shutdown;
use rocket::{
    fairing::AdHoc,
    futures::{SinkExt, StreamExt},
    http::Status,
    request::FromRequest,
    tokio::{
        select,
        sync::{Mutex, Notify, RwLock},
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    sync::{
        atomic::{AtomicBool, AtomicUsize},
        Arc,
    },
};

#[derive(Serialize, Deserialize)]

enum Message {
    SessionNotOpen,
    SessionEnded,
    SessionBegun,
    Data(String),
}

#[derive(Default)]
pub struct Sessions {
    sessions: RwLock<HashMap<String, Arc<SessionInfo>>>,
    session_notify: Notify,
}

impl Sessions {
    pub fn new() -> Self {
        Self {
            sessions: Default::default(),
            session_notify: Notify::new(),
        }
    }
}

pub struct SessionInfo {
    notify_consumer: Notify,
    notify_change: Notify,
    notify_session_over: Notify,
    live: AtomicBool,
    data: Mutex<VecDeque<String>>,
    queue_info: RwLock<Vec<usize>>,
    next_queue_id: AtomicUsize,
    queue_updated: Notify,
}

impl Default for SessionInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SessionInfo {
    fn drop(&mut self) {
        println!("dropping session");
    }
}

impl SessionInfo {
    pub fn new() -> Self {
        println!("new session");
        Self {
            notify_consumer: Notify::new(),
            notify_change: Notify::new(),
            notify_session_over: Notify::new(),
            data: Mutex::default(),
            next_queue_id: AtomicUsize::new(0),
            queue_info: Default::default(),
            live: AtomicBool::new(false),
            queue_updated: Notify::new(),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for &'r Sessions {
    type Error = ();

    async fn from_request(
        request: &'r rocket::Request<'_>,
    ) -> rocket::request::Outcome<&'r Sessions, Self::Error> {
        if let Some(some) = request.rocket().state() {
            rocket::outcome::Outcome::Success(some)
        } else {
            rocket::outcome::Outcome::Forward(Status::ServiceUnavailable)
        }
    }
}

#[get("/open_session/<session_id>")]
pub async fn open_new_session(
    websocket: ws::WebSocket,
    sessions: &Sessions,
    session_id: String,
    shutdown: Shutdown,
) -> ws::Channel<'_> {
    // let mut map = HashMap::new();
    // for (key, val) in &*sessions.sessions.read().await{
    //     let val = Vec::from(val.data.lock().await.clone());
    //     map.insert(key.clone(), val);
    // }
    // rocket::
    let mut lock = sessions.sessions.write().await;
    let session = lock.entry(session_id.clone()).or_default().clone();
    if session.live.swap(true, Relaxed) {
        return websocket.channel(move |mut stream| {
            Box::pin(async move {
                stream
                    .close(Some(ws::frame::CloseFrame {
                        code: ws::frame::CloseCode::Error,
                        reason: "SessionId already In use".into(),
                    }))
                    .await
            })
        });
    }
    drop(lock);

    sessions.session_notify.notify_waiters();
    session.notify_session_over.notify_waiters();
    use std::sync::atomic::Ordering::Relaxed;
    websocket.channel(move |mut stream| {
        Box::pin(async move {
            let val = async {
                while session.live.load(std::sync::atomic::Ordering::Relaxed){
                    let shutdown = shutdown.clone();
                    select! {
                        Some(next_recv) = stream.next() => {
                            let next_recv = next_recv?;
                            match next_recv{
                                ws::Message::Text(val) => {
                                    session.data.lock().await.push_back(val);
                                    session.notify_consumer.notify_one();
                                    session.notify_change.notify_waiters();

                                    rocket::tokio::task::yield_now().await;

                                    let data = serde_json::to_string(&*session.data.lock().await).unwrap_or(String::new());
                                    stream.send(ws::Message::Text(data)).await?;
                                },
                                ws::Message::Close(_) => break,
                                _ => {}
                            }
                        }
                        _ = session.notify_change.notified() => {
                            let data = session.data.lock().await;
                            let data = serde_json::to_string(&*data).unwrap_or(String::new());
                            stream.send(ws::Message::Text(data)).await?;
                        }
                        _ = shutdown => {
                            break
                        }
                    }
                }

                Ok(())
            }.await;

            sessions.session_notify.notify_waiters();

            session.live.store(false, Relaxed);
            session.data.lock().await.clear();
            session.notify_change.notify_waiters();
            session.notify_session_over.notify_waiters();
            drop(session);

            let mut lock = sessions.sessions.write().await;
            if let Some(entry) = lock.get(&session_id){
                if Arc::strong_count(entry) == 1{
                    lock.remove(&session_id);
                }
            }
            drop(lock);

            val
        })
    })
}

#[get("/consume_single/<session_id>")]
pub async fn session_single_consumer(
    websocket: ws::WebSocket,
    sessions: &Sessions,
    session_id: String,
    shutdown: Shutdown,
) -> Result<ws::Channel<'_>, &str> {
    #[derive(Deserialize, Serialize)]
    enum Message {
        SessionQueuePos(usize),
        NoSessionQueuePos(usize),
        Consumed(String),
    }
    use std::sync::atomic::Ordering::Relaxed;
    let session = sessions
        .sessions
        .write()
        .await
        .entry(session_id.clone())
        .or_default()
        .clone();

    Ok(websocket.channel(move |mut stream| {
        Box::pin(async move {
            let ret = async move{


                
                
                let mut lock = session.queue_info.write().await;
                if lock.is_empty(){
                    let val = session.data.lock().await.pop_front();
                    session.notify_change.notify_waiters();
                    if let Some(val) = val{
                        drop(lock);
                        let val = serde_json::to_string(&Message::Consumed(val)).unwrap_or(String::new());
                        stream.send(ws::Message::Text(val)).await?;
                        return Ok(())
                    }
                }
                let line_future = session.notify_consumer.notified();
                let queue_id = session.next_queue_id.fetch_add(1, Relaxed);
                lock.push(queue_id);
                drop(lock);
                session.queue_updated.notify_waiters();

            let ret = async {

                macro_rules! queue_pos {
                    () => {
                        session.queue_info.read().await.iter().position(|i| *i == queue_id).unwrap()
                    };
                }

                if session.live.load(Relaxed){
                    let val = serde_json::to_string(&Message::SessionQueuePos(queue_pos!())).unwrap_or(String::new());
                    stream.send(ws::Message::Text(val)).await?;
                }else{
                    let val = serde_json::to_string(&Message::NoSessionQueuePos(queue_pos!())).unwrap_or(String::new());
                    stream.send(ws::Message::Text(val)).await?;
                }

                
                
                rocket::tokio::pin!(line_future);
                loop{
                    let shutdown = shutdown.clone();
                    
                    select!{
                        _ = sessions.session_notify.notified() => {
                            if !session.live.load(Relaxed){
                                let val = serde_json::to_string(&Message::NoSessionQueuePos(queue_pos!())).unwrap_or(String::new());
                                stream.send(ws::Message::Text(val)).await?;
                            }else{
                                let pos = queue_pos!();
                                if pos == 0 {
                                    let val = session.data.lock().await.pop_front();
                                    session.queue_updated.notify_waiters();
                                    if let Some(val) = val{
                                        session.notify_change.notify_waiters();
                                        let val = serde_json::to_string(&Message::Consumed(val)).unwrap_or(String::new());
                                        stream.send(ws::Message::Text(val)).await?;
                                        return Ok(())
                                    }

                                    let val = serde_json::to_string(&Message::SessionQueuePos(pos)).unwrap_or(String::new());
                                    stream.send(ws::Message::Text(val)).await?;
                                }else{
                                    let val = serde_json::to_string(&Message::SessionQueuePos(pos)).unwrap_or(String::new());
                                    stream.send(ws::Message::Text(val)).await?;
                                }
                            }
                        }
                        _ = session.queue_updated.notified() => {
                            if !session.live.load(Relaxed){
                                let val = serde_json::to_string(&Message::NoSessionQueuePos(queue_pos!())).unwrap_or(String::new());
                                stream.send(ws::Message::Text(val)).await?;
                            }else{
                                let pos = queue_pos!();
                                if pos == 0 {
                                    let val = session.data.lock().await.pop_front();
                                    // session.waiting.fetch_sub(1, Relaxed);

                                    if let Some(val) = val{
                                        {
                                            let mut lock = session.queue_info.write().await;
                                            let pos = lock.iter().position(|i| *i == queue_id).unwrap();
                                            lock.remove(pos);
                                        }
                                        session.queue_updated.notify_waiters();

                                        session.notify_change.notify_waiters();
                                        let val = serde_json::to_string(&Message::Consumed(val)).unwrap_or(String::new());
                                        stream.send(ws::Message::Text(val)).await?;
                                        return Ok(())
                                    }

                                    let val = serde_json::to_string(&Message::SessionQueuePos(pos)).unwrap_or(String::new());
                                    stream.send(ws::Message::Text(val)).await?;
                                }else{
                                    let val = serde_json::to_string(&Message::SessionQueuePos(pos)).unwrap_or(String::new());
                                    stream.send(ws::Message::Text(val)).await?;
                                }
                            }
                        }
                        _ = &mut line_future => {
                            if !session.live.load(Relaxed){
                                let val = serde_json::to_string(&Message::NoSessionQueuePos(queue_pos!())).unwrap_or(String::new());
                                stream.send(ws::Message::Text(val)).await?;
                            }else{
                                let val = session.data.lock().await.pop_front();
                                // session.waiting.fetch_sub(1, Relaxed);

                                if let Some(val) = val{
                                    {
                                        let mut lock = session.queue_info.write().await;
                                        if let Some(pos) = lock.iter().position(|i| *i == queue_id){
                                            lock.remove(pos);
                                        }else{
                                            continue;
                                        }
                                    }
                                    session.queue_updated.notify_waiters();
                                    session.notify_change.notify_waiters();
                                    let val = serde_json::to_string(&Message::Consumed(val)).unwrap_or(String::new());
                                    stream.send(ws::Message::Text(val)).await?;
                                    return Ok(())
                                }
                            }
                        }
                        
                        Some(Ok(ws::Message::Close(_)))
                        | Some(Err(_)) = stream.next() => {
                            return Ok(())
                        }
                        _ = shutdown => {
                            return Ok(())
                        }
                    }
                }
            }.await;
            {
                let mut lock = session.queue_info.write().await;
                let pos = lock.iter().position(|i| *i == queue_id);
                if let Some(pos) = pos{
                    lock.remove(pos);
                }
                session.queue_updated.notify_waiters();
            }

            ret
            }.await;

            let mut lock = sessions.sessions.write().await;
            if let Some(entry) = lock.get(&session_id){
                if Arc::strong_count(entry) == 1{
                    lock.remove(&session_id);
                }
            }
            drop(lock);

            ret
        })
    }))
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("test", |rocket| async {
        rocket
            .manage(Sessions::new())
            .mount("/api", routes![open_new_session, session_single_consumer])
    })
}
