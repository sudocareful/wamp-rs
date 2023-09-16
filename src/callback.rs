use wamp_helpers::messages::{Welcome, Challenge, Abort, Goodbye, ErrorMessage, Interrupt, Yield, Event, Subscribed, MessageResult, Published, Unsubscribed, Registered, Unregistered, Subscribe, WampMessageTrait, Call, Publish, Unsubscribe, Register, Unregister};

use crate::{client::Context, error::Error};



pub enum Events {
    Welcome(Callback<Welcome>),
    Challenge(Callback<Challenge>),
    Abort(Callback<Abort>),
    Goodbye(Callback<Goodbye>),
    Error(Callback<ErrorMessage>),
    Interrupt(Callback<Interrupt>),
    Yield(Callback<Yield>)
}

pub(crate) type Callback<T> = Box<dyn Fn(Context, T) -> Context>;

pub struct CallbackHandler {
    pub(crate) callbacks: Vec<Events>,
    pub(crate) subscriptions: Vec<(u64, Callback<Event>, Option<u64>)>,
    pub(crate) on_subscribed: Vec<(u64, Callback<Result<Subscribed, ErrorMessage>>)>,
    pub(crate) call_results: Vec<(u64, Callback<Result<MessageResult, ErrorMessage>>)>,
    pub(crate) publish_callbacks: Vec<(u64, Callback<Result<Published, ErrorMessage>>)>,
    pub(crate) unsubscribe_callbacks: Vec<(u64, Callback<Result<Unsubscribed, ErrorMessage>>)>,
    pub(crate) register_callbacks: Vec<(u64, Callback<Result<Registered, ErrorMessage>>)>,
    pub(crate) unregistered_callbacks: Vec<(u64, Callback<Result<Unregistered, ErrorMessage>>)>,
    pub(crate) message_queue: Vec<String>
}

impl CallbackHandler {
    pub(crate) fn merge(&mut self, handler: CallbackHandler) {
        self.callbacks.extend(handler.callbacks);
        self.subscriptions.extend(handler.subscriptions);
        self.on_subscribed.extend(handler.on_subscribed);
        self.call_results.extend(handler.call_results);
        self.publish_callbacks.extend(handler.publish_callbacks);
        self.unsubscribe_callbacks.extend(handler.unsubscribe_callbacks);
        self.register_callbacks.extend(handler.register_callbacks);
        self.unregistered_callbacks.extend(handler.unregistered_callbacks);
        self.message_queue.extend(handler.message_queue);
    }

    pub(crate) fn new() -> CallbackHandler {
        CallbackHandler { 
            callbacks: vec![], 
            subscriptions: vec![],
            on_subscribed: vec![],
            call_results: vec![], 
            publish_callbacks: vec![], 
            unsubscribe_callbacks: vec![], 
            register_callbacks: vec![], 
            unregistered_callbacks: vec![],
            message_queue: vec![]
        }
    }

    pub fn send<T: WampMessageTrait>(&mut self, message: T) -> Result<(), Error> {
        self.message_queue.push(message.to_json().map_err(|e| Error::JsonError(e))?.to_string());
        Ok(())
    }

    pub fn subscribe(&mut self, subscription: Subscribe, on_subscribed: Callback<Result<Subscribed, ErrorMessage>>, on_event: Callback<Event>) -> Result<(), Error> {
        self.send(subscription.clone())?;
        self.subscriptions.push((subscription.request, on_event, None));
        self.on_subscribed.push((subscription.request, on_subscribed));
        Ok(())
    }

    pub fn call(&mut self, call: Call, on_result: Callback<Result<MessageResult, ErrorMessage>>) -> Result<(), Error> {
        self.send(call.clone())?;
        self.call_results.push((call.request, on_result));        
        Ok(())
    }

    pub fn publish(&mut self, publish: Publish, on_published: Callback<Result<Published, ErrorMessage>>) -> Result<(), Error> {
        self.send(publish.clone())?;
        self.publish_callbacks.push((publish.request, on_published));
        Ok(())
    }

    pub fn unsubscribe(&mut self, unsubscribe: Unsubscribe, on_unsubscribed: Callback<Result<Unsubscribed, ErrorMessage>>) -> Result<(), Error> {
        self.subscriptions.retain(|i| {
            if let Some(num) = i.2 {
                if num == unsubscribe.subscription {
                    return false
                }
            }
            true
        });
        self.send(unsubscribe.clone())?;
        self.unsubscribe_callbacks.push((unsubscribe.request, on_unsubscribed));
        Ok(())
    }

    pub fn register(&mut self, register: Register, on_registered: Callback<Result<Registered, ErrorMessage>>) -> Result<(), Error> {
        self.send(register.clone())?;
        self.register_callbacks.push((register.request, on_registered));
        Ok(())
    }

    pub fn unregister(&mut self, unregister: Unregister, on_unregistered: Callback<Result<Unregistered, ErrorMessage>>) -> Result<(), Error> {
        self.send(unregister.clone())?;
        self.unregistered_callbacks.push((unregister.request, on_unregistered));
        Ok(())
    }

    pub fn on(&mut self, callback: Events) {
        self.callbacks.push(callback);
    }
}