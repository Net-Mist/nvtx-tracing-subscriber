use tracing::{span, Event, Id, Metadata, Subscriber};
// use tracing_subscriber::layer::Context;

use nvtx::{range_pop, range_push, mark};
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
