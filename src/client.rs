use websockets::{WebSocketBuilder, WebSocketError, WebSocket};
use wamp_helpers::messages::{
    Event, 
    Subscribed, 
    Subscribe, 
    WampMessageTrait, 
    Call, 
    MessageResult, 
    Publish, 
    Published, 
    Unsubscribe, 
    Unsubscribed, 
    Register, 
    Registered, 
    Unregister, 
    Unregistered,
    Events as WampEvents, 
    ErrorMessage
};

use crate::{error::Error, callback::{CallbackHandler, Callback, Events}};

pub struct Context {
    messages: CallbackHandler
}

impl Context {
    pub fn send<T: WampMessageTrait>(&mut self, message: T) -> Result<(), Error> {
        self.messages.send(message)
    }

    pub fn subscribe(&mut self, subscription: Subscribe, on_subscribed: Callback<Result<Subscribed, ErrorMessage>>, on_event: Callback<Event>) -> Result<(), Error> {
        self.messages.subscribe(subscription, on_subscribed, on_event)
    }

    pub fn call(&mut self, call: Call, on_result: Callback<Result<MessageResult, ErrorMessage>>) -> Result<(), Error> { 
        self.messages.call(call, on_result)
    }

    pub fn publish(&mut self, publish: Publish, on_published: Callback<Result<Published, ErrorMessage>>) -> Result<(), Error> {
        self.messages.publish(publish, on_published)
    }

    pub fn unsubscribe(&mut self, unsubscribe: Unsubscribe, on_unsubscribed: Callback<Result<Unsubscribed, ErrorMessage>>) -> Result<(), Error> {
    
        self.messages.unsubscribe(unsubscribe, on_unsubscribed)
    }

    pub fn register(&mut self, register: Register, on_registered: Callback<Result<Registered, ErrorMessage>>) -> Result<(), Error> {
        self.messages.register(register, on_registered)
    }

    pub fn unregister(&mut self, unregister: Unregister, on_unregistered: Callback<Result<Unregistered, ErrorMessage>>) -> Result<(), Error> {
        self.messages.unregister(unregister, on_unregistered)
    }

    pub fn on(&mut self, callback: Events) {
        self.messages.on(callback)
    }
}

pub struct WampClient {
    websocket: WebSocket,
    handler: CallbackHandler
}


impl WampClient {

    pub fn on(&mut self, callback: Events) {
        self.handler.callbacks.push(callback);
    }

    pub async fn subscribe(&mut self, subscription: Subscribe, on_subscribed: Callback<Result<Subscribed, ErrorMessage>>, on_event: Callback<Event>) -> Result<(), Error> {
        self.send(subscription.clone()).await?;
        self.handler.subscriptions.push((subscription.request, on_event, None));
        self.handler.on_subscribed.push((subscription.request, on_subscribed));
        Ok(())
    }

    pub async fn call(&mut self, call: Call, on_result: Callback<Result<MessageResult, ErrorMessage>>) -> Result<(), Error> {
        self.send(call.clone()).await?;
        self.handler.call_results.push((call.request, on_result));
        Ok(())
    }

    pub async fn publish(&mut self, publish: Publish, on_published: Callback<Result<Published, ErrorMessage>>) -> Result<(), Error> {
        self.send(publish.clone()).await?;
        self.handler.publish_callbacks.push((publish.request, on_published));
        Ok(())
    }

    pub async fn unsubscribe(&mut self, unsubscribe: Unsubscribe, on_unsubscribed: Callback<Result<Unsubscribed, ErrorMessage>>) -> Result<(), Error> {
        self.handler.unsubscribe(unsubscribe, on_unsubscribed)
    }

    pub async fn register(&mut self, register: Register, on_registered: Callback<Result<Registered, ErrorMessage>>) -> Result<(), Error> {
        self.send(register.clone()).await?;
        self.handler.register_callbacks.push((register.request, on_registered));
        Ok(())
    }

    pub async fn unregister(&mut self, unregister: Unregister, on_unregistered: Callback<Result<Unregistered, ErrorMessage>>) -> Result<(), Error> {
        self.send(unregister.clone()).await?;
        self.handler.unregistered_callbacks.push((unregister.request, on_unregistered));
        Ok(())
    }

    pub async fn connect(url: &str, options: &mut WebSocketBuilder) -> Result<Self, WebSocketError> {
        let websocket = options.connect(url).await?;
        
        Ok(Self { 
            websocket, 
            handler: CallbackHandler::new()
        })
    }

    pub async fn handle_event(&mut self, event: WampEvents, context: Context) -> Context {
        match event {
            WampEvents::Welcome(welcome) => {
                for callback in &self.handler.callbacks {
                    if let Events::Welcome(cb) = callback {
                        return cb(context, welcome.clone());
                    }
                };
            },
            WampEvents::Abort(abort) => {
                for callback in &self.handler.callbacks {
                    if let Events::Abort(cb) = callback {
                        return cb(context, abort.clone());
                    }
                };
            },
            WampEvents::Challenge(challenge) => {
                for callback in &self.handler.callbacks {
                    if let Events::Challenge(cb) = callback {
                        return cb(context, challenge);
                    }
                }
            },
            WampEvents::Goodbye(goodbye) => {
                for callback in &self.handler.callbacks {
                    if let Events::Goodbye(cb) = callback {
                        return cb(context, goodbye);
                    }
                }
            },
            WampEvents::ErrorMessage(error) => {

                match error.request_type {

                    Subscribe::ID => {
                        for callback in &self.handler.on_subscribed {
                            if callback.0 == error.request {    
                                return callback.1(context , Err(error.clone()));
                            }
                        }
                    }

                    Call::ID => {
                        for callback in &self.handler.call_results {
                            if callback.0 == error.request {
                                return callback.1(context, Err(error));
                            }
                        }
                    }

                    Publish::ID => {
                        for callback in &self.handler.call_results {
                            if callback.0 == error.request {
                                return callback.1(context, Err(error));
                            }
                        }
                    }

                    Unsubscribe::ID => {
                        for callback in &self.handler.unsubscribe_callbacks {
                            if callback.0 == error.request {
                                return  callback.1(context, Err(error));
                            }
                        }
                    }

                    Register::ID => {
                        for callback in &self.handler.register_callbacks {
                            if callback.0 == error.request {
                                return callback.1(context, Err(error));
                            }
                        }
                    }

                    Unregister::ID => {
                        for callback in &self.handler.unregistered_callbacks {
                            if callback.0 == error.request {
                                return callback.1(context, Err(error));
                            }
                        }
                    }

                    _ => {
                        for callback in &self.handler.callbacks {
                            if let Events::Error(cb) = callback {
                                return cb(context, error.clone());
                            }
                        }
                    }
                }
            },
            WampEvents::Published(published) => {
                for callback in &self.handler.publish_callbacks {
                    if published.request == callback.0 {
                        return callback.1(context, Ok(published));
                    }
                }
            },
            WampEvents::Subscribed(subscribed) => {
                for callback in &mut self.handler.subscriptions {
                    callback.2 = Some(subscribed.subscription);
                }

                for callback in &self.handler.on_subscribed {
                    if subscribed.request == callback.0 {
                        return callback.1(context, Ok(subscribed.clone()));
                    }
                };
            },
            WampEvents::Unsubscribed(unsubscribed) => {
                for callback in &self.handler.unsubscribe_callbacks {
                    if unsubscribed.request == callback.0 {
                        return callback.1(context, Ok(unsubscribed));
                    }
                }
            },
            WampEvents::Event(event) => {
                for callback in &self.handler.subscriptions {
                    if let Some(subscription) = callback.2 {
                        if event.subscription == subscription {
                            return callback.1(context, event);
                        }
                    }
                }
            },
            WampEvents::MessageResult(result) => {
                for callback in &self.handler.call_results {
                    if callback.0 == result.request {
                        return callback.1(context, Ok(result));
                    }
                }
            },
            WampEvents::Registered(registered) => {
                for callback in &self.handler.register_callbacks {
                    if callback.0 == registered.request {
                        return callback.1(context, Ok(registered));
                    }
                }
            },
            WampEvents::Unregistered(unregistered) => {
                for callback in &self.handler.unregistered_callbacks {
                    if callback.0 == unregistered.request {
                        return callback.1(context, Ok(unregistered));
                    }
                }
            },
            WampEvents::Interrupt(interrupt) => {
                for callback in &self.handler.callbacks {
                    if let Events::Interrupt(cb) = callback {
                        return cb(context, interrupt);
                    }
                }
            },
            WampEvents::Yield(yield_frame) => {
                for callback in &self.handler.callbacks {
                    if let Events::Yield(cb) = callback {
                        return cb(context, yield_frame);
                    }    
                }
            },
            _ => {
                println!("Wamp Server frame received... Not handling.");
            }
        } 
        context
    }

    pub async fn loop_messages(&mut self){
        loop {
            match self.websocket.receive().await {
                Ok(f) => {
                    if let Some(message) = f.as_text() {
                        let message = message.0;
                        let event = WampEvents::parse_message(message).unwrap();
                        let original_context = Context { messages: CallbackHandler::new() };
                        let context = self.handle_event(event.clone(), original_context).await;
                        self.handler.merge(context.messages);
                        let mut to_send = vec![];
                        self.handler.message_queue.retain(|item| {
                            to_send.push(item.clone());
                            false
                        });

                        match event {
                            WampEvents::Unregistered(unregistered) => {
                                self.handler.unregistered_callbacks.retain(|i| {
                                    if i.0 == unregistered.request {
                                        return false
                                    }
                                    true
                                });
                            },
                            WampEvents::Unsubscribed(unsubscribed ) => {
                                self.handler.unsubscribe_callbacks.retain(|i| {
                                    if i.0 == unsubscribed.request {
                                        return false
                                    }
                                    true
                                });
                            },
                            WampEvents::Subscribed(subscribed) => {
                                self.handler.on_subscribed.retain(|i| {
                                    if i.0 == subscribed.request {
                                        return false
                                    }
                                    true
                                })
                            },
                            WampEvents::MessageResult(result) => {
                                self.handler.call_results.retain(|i| {
                                    if i.0 == result.request {
                                        return false
                                    }
                                    true
                                })
                            },
                            WampEvents::Registered(registered) => {
                                self.handler.register_callbacks.retain(|i| {
                                    if i.0 == registered.request {
                                        return false
                                    }
                                    true
                                })
                            },
                            WampEvents::Published(published) => {
                                self.handler.publish_callbacks.retain(|i| {
                                    if i.0 == published.request {
                                        return false
                                    }
                                    true
                                })
                            }
                            _ => { }
                        };

                        for message in to_send {
                            let _ = self.websocket.send_text(message.to_string()).await;
                        }
                    }
                },
                Err(_e) => {
                },
            };   
        }
    }

    pub async fn send<M: WampMessageTrait>(&mut self, message: M) -> Result<(), Error> {
        Ok(self.websocket.send_text(
        message
                .to_json()
                .map_err(|e| Error::JsonError(e))?
                .to_string()
            )
            .await
            .map_err(|e| Error::WsError(e))?)
    }


}
