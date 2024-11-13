use tracing::{span, Event, Id, Metadata, Subscriber};
use tracing_subscriber::layer::Context;
// use tracing_subscriber::prelude::*;
use tracing_subscriber::Layer;

use nvtx::{mark, range_pop, range_push};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex,
};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

struct Span {
    id: Id,
    name: String,
}

pub struct NvtxSubscriber {
    spans: Mutex<Vec<Span>>,
}

impl NvtxSubscriber {
    pub fn new() -> Self {
        Self {
            spans: Mutex::new(Vec::new()),
        }
    }
}

impl Subscriber for NvtxSubscriber {
    fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
        // Enable all spans and events
        true
    }

    fn new_span(&self, span: &span::Attributes<'_>) -> Id {
        let name = span.metadata().name();

        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let id = Id::from_u64(id as u64);

        let mut lock = self.spans.lock().unwrap();
        lock.push(Span {
            id: id.clone(),
            name: name.to_string(),
        });
        id
    }

    fn record(&self, _span: &Id, _values: &span::Record<'_>) {
        // No-op
    }

    fn record_follows_from(&self, _span: &Id, _follows: &Id) {
        // No-op
    }

    fn event(&self, event: &Event<'_>) {
        mark!("{:?}", event);
    }

    fn enter(&self, span: &Id) {
        let lock = self.spans.lock().unwrap();
        let span = lock.iter().find(|s| s.id == *span).unwrap();
        let name = &span.name;
        range_push!("{}", name);
    }

    fn exit(&self, _span: &Id) {
        range_pop!();
    }
}

pub struct NvtxLayer {
    spans: Mutex<Vec<Span>>,
}

impl NvtxLayer {
    pub fn new() -> Self {
        Self {
            spans: Mutex::new(Vec::new()),
        }
    }
}

impl<S: Subscriber> Layer<S> for NvtxLayer {
    fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
        let _ = (id, ctx);

        let name = attrs.metadata().name();

        let mut lock = self.spans.lock().unwrap();
        lock.push(Span {
            id: id.clone(),
            name: name.to_string(),
        });
    }

    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        mark!("{:?}", event);
    }

    fn on_enter(&self, id: &Id, _ctx: Context<'_, S>) {
        let lock = self.spans.lock().unwrap();
        let span = lock.iter().find(|s| s.id == *id).unwrap();
        let name = &span.name;
        range_push!("{}", name);
    }

    fn on_exit(&self, _id: &Id, _ctx: Context<'_, S>) {
        range_pop!();
    }
}
