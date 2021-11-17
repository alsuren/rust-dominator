use std::borrow::Cow;
use std::mem::ManuallyDrop;

use wasm_bindgen::{UnwrapThrowExt, intern};
use discard::Discard;
use web_sys::{EventTarget, Event};

use crate::dom::EventOptions;
use crate::traits::StaticEvent;


pub(crate) struct EventListener {
    listener: Option<gloo_events::EventListener>,
}

// TODO should these inline ?
impl EventListener {
    #[inline]
    pub(crate) fn new<F>(elem: EventTarget, name: &'static str, options: &EventOptions, callback: F) -> Self where F: FnMut(&Event) + 'static {
        let name: &'static str = intern(name);
        let listener = gloo_events::EventListener::new_with_options(&elem, Cow::Borrowed(name), options.into(), callback);
        Self { listener: Some(listener) }
    }

    #[inline]
    pub(crate) fn once<F>(elem: EventTarget, name: &'static str, callback: F) -> Self where F: FnOnce(&Event) + 'static {
        let name: &'static str = intern(name);
        let listener = gloo_events::EventListener::once(&elem, Cow::Borrowed(name), callback);
        Self { listener: Some(listener) }
    }
}

// Our default drop impl stays registered to the event, but gloo_events::EventListener
// requires you to call ::forget() to stay registered.
impl Drop for EventListener {
    #[inline]
    fn drop(&mut self) {
        self.listener.take().map(|l| l.forget());
    }
}

impl Discard for EventListener {
    #[inline]
    fn discard(mut self) {
        let _ = self.listener.take().unwrap_throw();
    }
}


#[inline]
pub(crate) fn on<E, F>(element: EventTarget, options: &EventOptions, mut callback: F) -> EventListener
    where E: StaticEvent,
          F: FnMut(E) + 'static {
    EventListener::new(element, E::EVENT_TYPE, options, move |e| {
        callback(E::unchecked_from_event(e.clone()));
    })
}


// TODO move this into the discard crate
// TODO verify that this is correct and doesn't leak memory or cause memory safety
pub(crate) struct ValueDiscard<A>(ManuallyDrop<A>);

impl<A> ValueDiscard<A> {
    #[inline]
    pub(crate) fn new(value: A) -> Self {
        ValueDiscard(ManuallyDrop::new(value))
    }
}

impl<A> Discard for ValueDiscard<A> {
    #[inline]
    fn discard(self) {
        // TODO verify that this works
        ManuallyDrop::into_inner(self.0);
    }
}


// TODO move this into the discard crate
// TODO replace this with an impl for FnOnce() ?
pub(crate) struct FnDiscard<A>(A);

impl<A> FnDiscard<A> where A: FnOnce() {
    #[inline]
    pub(crate) fn new(f: A) -> Self {
        FnDiscard(f)
    }
}

impl<A> Discard for FnDiscard<A> where A: FnOnce() {
    #[inline]
    fn discard(self) {
        self.0();
    }
}
