use crate::RePhasor;
//use std::option;

#[derive(Copy, Clone)]
pub enum Behavior {
    Step,
    Linear,
    GlissTiny,
    GlissSmall,
    GlissMedium,
    GlissLarge,
    GlissHuge,
}

pub struct Gesture<T> {
    prev: T,
    next: T,
    ratemul: T,
    behavior: Behavior,
    rephasor: RePhasor,
    lphs: T,
    next_behavior: Behavior,
}

pub struct LinearGesture<'a> {
    gest: Gesture<f32>,
    path: Option<&'a Vec<GestureVertex<f32>>>,
    pos: usize,
}

pub struct LinearGestureBuilder {
    gest: Gesture<f32>,
    path: Vec<GestureVertex<f32>>,
    pos: usize,
}

#[derive(Copy, Clone)]
enum GestureEventType {
    None,
    Scalar,
    Rate,
    Behavior,
    Wait,
}

#[derive(Copy, Clone)]
// TODO: converted this form a Union because unions are unsafe.
// There may be a more efficient way to handle this?
struct GestureEventData {
    rate: Option<[u32; 2]>,
    scalar: Option<f32>,
    behavior: Option<Behavior>,
}

#[derive(Copy, Clone)]
pub struct GestureEvent {
    evtype: GestureEventType,
    data: GestureEventData,
}

const EVENT_QUEUE_SIZE: usize = 16;

struct GestureEventQueue {
    queue: [GestureEvent; EVENT_QUEUE_SIZE],
    head: usize,
    tail: usize,
    num_events: usize,
}

pub struct EventfulGesture {
    gest: Gesture<f32>,
    events: GestureEventQueue,
    vtx: GestureVertex<f32>,
    wait: u32,
}

#[derive(Copy, Clone)]
pub struct GestureVertex<T> {
    pub val: T,
    pub num: u32,
    pub den: u32,
    pub bhvr: Behavior,
}

pub trait SignalGenerator {
    fn next_vertex(&mut self) -> GestureVertex<f32>;
    fn compute_rephasor(&mut self, clk: f32) -> f32;
    fn interpolate(&mut self, phs: f32) -> f32;
    fn new_period(&mut self, phs: f32) -> bool;
    fn tick(&mut self, clk: f32) -> f32 {
        let phs = self.compute_rephasor(clk);

        if self.new_period(phs) {
            let vtx = self.next_vertex();
            self.update(&vtx);
        }
        self.interpolate(phs)
    }
    fn update(&mut self, vtx: &GestureVertex<f32>);
    fn preinit(&mut self) {
        let a = self.next_vertex();
        self.update(&a);
    }
}

impl SignalGenerator for Gesture<f32> {
    fn new_period(&mut self, phs: f32) -> bool {
        self.lphs > phs
    }

    fn next_vertex(&mut self) -> GestureVertex<f32> {
        GestureVertex {
            val: 0.0,
            num: 1,
            den: 1,
            bhvr: Behavior::Linear,
        }
    }

    fn compute_rephasor(&mut self, clk: f32) -> f32 {
        self.rephasor.tick(clk)
    }

    fn interpolate(&mut self, phs: f32) -> f32 {
        let a = apply_behavior(phs, &self.behavior);

        let out = (1.0 - a) * self.prev + a * self.next;

        self.lphs = phs;

        out
    }

    fn update(&mut self, vtx: &GestureVertex<f32>) {
        // Set the previous rate multiplier
        // because we want this relationship: A -> A_rm (A_bhvr) -> B
        self.rephasor.set_scale(self.ratemul);

        // with that set, we can cache the upcoming RM
        self.ratemul = vtx.num as f32 / vtx.den as f32;
        self.prev = self.next;
        self.next = vtx.val;
        self.behavior = self.next_behavior;
        self.next_behavior = vtx.bhvr;
    }
}

impl Default for Gesture<f32> {
    fn default() -> Self {
        Gesture::<f32>::new()
    }
}

impl Gesture<f32> {
    pub fn new() -> Self {
        Gesture {
            prev: 0.0,
            next: 0.0,
            ratemul: 1.0,
            behavior: Behavior::GlissMedium,
            next_behavior: Behavior::GlissMedium,
            rephasor: RePhasor::new(),
            // triggers update on init
            lphs: 1.0,
        }
    }
}

fn gliss_it(phs: f32, glisspos: f32) -> f32 {
    let mut a;
    if phs < glisspos {
        a = 0.0;
    } else {
        a = phs - glisspos;
        if a < 0.0 {
            a = 0.0;
        }
        a /= 1.0 - glisspos;
        a = a * a * a;
    }
    a
}

fn apply_behavior(phs: f32, bhvr: &Behavior) -> f32 {
    match bhvr {
        Behavior::Step => 0.0,
        Behavior::Linear => phs,
        Behavior::GlissMedium => gliss_it(phs, 0.75),
        Behavior::GlissSmall => gliss_it(phs, 0.85),
        Behavior::GlissLarge => gliss_it(phs, 0.5),
        Behavior::GlissHuge => gliss_it(phs, 0.1),
        Behavior::GlissTiny => gliss_it(phs, 0.9),
    }
}

impl<'a> Default for LinearGesture<'a> {
    fn default() -> Self {
        LinearGesture::new()
    }
}

impl<'a> LinearGesture<'a> {
    pub fn new() -> Self {
        LinearGesture {
            gest: Gesture::new(),
            path: None,
            pos: 0,
        }
    }

    pub fn init(&mut self, path: &'a Vec<GestureVertex<f32>>) {
        self.path = Some(path);
        // get vertex, now next vertex is on deck
        let a = self.next_vertex();
        // this is called before the first tick
        // important values are ratemul and next
        self.update(&a);

        // tick will be called on the first call,
        // which will set this next value to be prev
        // the rate multiplier here will also be held onto
    }
}

impl SignalGenerator for LinearGesture<'_> {
    fn next_vertex(&mut self) -> GestureVertex<f32> {
        match self.path {
            Some(x) => {
                let nxt = x[self.pos];
                self.pos += 1;
                if self.pos >= x.len() {
                    self.pos = 0;
                }
                nxt
            }

            None => GestureVertex {
                val: 0.0,
                num: 1,
                den: 1,
                bhvr: Behavior::Linear,
            },
        }
    }

    fn compute_rephasor(&mut self, clk: f32) -> f32 {
        self.gest.compute_rephasor(clk)
    }

    fn interpolate(&mut self, phs: f32) -> f32 {
        self.gest.interpolate(phs)
    }

    fn new_period(&mut self, phs: f32) -> bool {
        self.gest.new_period(phs)
    }

    fn update(&mut self, vtx: &GestureVertex<f32>) {
        self.gest.update(vtx);
    }
}

impl Default for LinearGestureBuilder {
    fn default() -> Self {
        LinearGestureBuilder::new()
    }
}

impl LinearGestureBuilder {
    pub fn new() -> Self {
        LinearGestureBuilder {
            gest: Gesture::new(),
            path: vec![],
            pos: 0,
        }
    }

    // Appends vertex to the path
    pub fn append(&mut self, vtx: GestureVertex<f32>) {
        self.path.push(vtx);
    }

    // To be called when path is done being populated with events
    pub fn done(&mut self) {
        if !self.path.is_empty() {
            let a = self.next_vertex();
            self.update(&a);
        }
    }
}

impl SignalGenerator for LinearGestureBuilder {
    fn next_vertex(&mut self) -> GestureVertex<f32> {
        let x = &self.path;
        let nxt = x[self.pos];
        self.pos += 1;
        if self.pos >= x.len() {
            // just hold at the end, don't loop back
            self.pos = x.len() - 1;
        }
        nxt
    }

    fn compute_rephasor(&mut self, clk: f32) -> f32 {
        self.gest.compute_rephasor(clk)
    }

    fn interpolate(&mut self, phs: f32) -> f32 {
        self.gest.interpolate(phs)
    }

    fn new_period(&mut self, phs: f32) -> bool {
        self.gest.new_period(phs)
    }

    fn update(&mut self, vtx: &GestureVertex<f32>) {
        self.gest.update(vtx);
    }
}

pub fn behavior_from_integer(bhvr: u16) -> Result<Behavior, u16> {
    match bhvr {
        0 => Ok(Behavior::Step),
        1 => Ok(Behavior::Linear),
        2 => Ok(Behavior::GlissTiny),
        3 => Ok(Behavior::GlissSmall),
        4 => Ok(Behavior::GlissMedium),
        5 => Ok(Behavior::GlissLarge),
        6 => Ok(Behavior::GlissHuge),
        _ => Err(bhvr),
    }
}

#[no_mangle]
pub extern "C" fn vb_gesture_tick(vb: &mut LinearGestureBuilder, clk: f32) -> f32 {
    vb.tick(clk)
}

#[no_mangle]
pub extern "C" fn vb_gesture_done(vb: &mut LinearGestureBuilder) {
    vb.done()
}

#[no_mangle]
pub extern "C" fn vb_gesture_free(vd: &mut LinearGestureBuilder) {
    let ptr = unsafe { Box::from_raw(vd) };
    drop(ptr);
}

#[no_mangle]
pub extern "C" fn vb_gesture_append(
    vb: &mut LinearGestureBuilder,
    val: f32,
    num: u32,
    den: u32,
    bhvr: u16,
) {
    let b = behavior_from_integer(bhvr);

    // TODO: Clippy is having a hard time with this expressoin...
    if b.is_ok() {
        vb.append(GestureVertex {
            val,
            num,
            den,
            bhvr: b.unwrap(),
        });
    }
}

#[no_mangle]
pub extern "C" fn vb_gesture_new() -> Box<LinearGestureBuilder> {
    Box::new(LinearGestureBuilder::new())
}

impl SignalGenerator for EventfulGesture {
    fn next_vertex(&mut self) -> GestureVertex<f32> {
        let events = &mut self.events;
        while events.has_events() && self.wait == 0 {
            let evt = events.dequeue();

            match evt.evtype {
                GestureEventType::Scalar => {
                    self.vtx.val = evt.data.scalar.expect("no scalar found");
                }
                GestureEventType::Behavior => {
                    self.vtx.bhvr = evt.data.behavior.expect("no behavior found");
                }
                GestureEventType::Rate => {
                    let rate = evt.data.rate.expect("no rate found");
                    self.vtx.num = rate[0];
                    self.vtx.den = rate[1];
                }
                GestureEventType::Wait => {
                    // HACK: use rate to store data
                    let wait = evt.data.rate.expect("no wait found");
                    let wait = wait[0];

                    self.wait = wait;
                }
                _ => {}
            }
        }

        if self.wait > 0 {
            self.wait -= 1;
        }
        self.vtx
    }

    fn compute_rephasor(&mut self, clk: f32) -> f32 {
        self.gest.compute_rephasor(clk)
    }

    fn interpolate(&mut self, phs: f32) -> f32 {
        self.gest.interpolate(phs)
    }

    fn new_period(&mut self, phs: f32) -> bool {
        self.gest.new_period(phs)
    }

    fn update(&mut self, vtx: &GestureVertex<f32>) {
        self.gest.update(vtx);
    }
}

impl Default for EventfulGesture {
    fn default() -> Self {
        EventfulGesture {
            gest: Gesture::new(),
            events: GestureEventQueue::new(),
            vtx: GestureVertex {
                val: 0.,
                num: 1,
                den: 1,
                bhvr: Behavior::Linear,
            },
            wait: 0,
        }
    }
}

impl EventfulGesture {
    pub fn scalar(&mut self, scalar: f32) {
        self.events.enqueue_scalar(scalar);
    }
    pub fn rate(&mut self, rate: [u32; 2]) {
        self.events.enqueue_rate(rate[0], rate[1]);
    }
    pub fn behavior(&mut self, bhvr: Behavior) {
        self.events.enqueue_behavior(bhvr);
    }
    pub fn wait(&mut self, wait: u32) {
        self.events.enqueue_wait(wait);
    }
}

impl GestureEventQueue {
    pub fn new() -> Self {
        let evt_default = GestureEvent {
            evtype: GestureEventType::None,
            data: GestureEventData {
                scalar: None,
                rate: None,
                behavior: None,
            },
        };
        GestureEventQueue {
            queue: [evt_default; EVENT_QUEUE_SIZE],
            head: 0,
            tail: 0,
            num_events: 0,
        }
    }

    pub fn enqueue_scalar(&mut self, scalar: f32) {
        let evt = self.enqueue();

        evt.evtype = GestureEventType::Scalar;
        evt.data.scalar = Some(scalar);
        evt.data.behavior = None;
        evt.data.rate = None;
    }

    pub fn enqueue_rate(&mut self, num: u32, den: u32) {
        let evt = self.enqueue();

        evt.evtype = GestureEventType::Rate;
        evt.data.scalar = None;
        evt.data.behavior = None;
        evt.data.rate = Some([num, den]);
    }

    pub fn enqueue_behavior(&mut self, bhvr: Behavior) {
        let evt = self.enqueue();

        evt.evtype = GestureEventType::Behavior;
        evt.data.scalar = None;
        evt.data.behavior = Some(bhvr);
        evt.data.rate = None;
    }

    pub fn enqueue_wait(&mut self, wait: u32) {
        let evt = self.enqueue();

        evt.evtype = GestureEventType::Wait;
        evt.data.scalar = None;
        // HACK: re-use rate to store wait value
        evt.data.rate = Some([wait, 0]);
    }

    /// Enques event and returns reference to it
    pub fn enqueue(&mut self) -> &mut GestureEvent {
        if self.num_events >= EVENT_QUEUE_SIZE {
            panic!("Event overflow")
        }
        let evt = &mut self.queue[self.tail];

        // initialize values to be nothing

        evt.evtype = GestureEventType::None;
        evt.data.scalar = None;
        evt.data.behavior = None;
        evt.data.behavior = None;
        evt.data.rate = None;

        self.tail += 1;
        self.tail %= EVENT_QUEUE_SIZE;
        self.num_events += 1;

        evt
    }

    pub fn dequeue(&mut self) -> &GestureEvent {
        if self.num_events == 0 {
            panic!("event underflow")
        }

        let evt = &self.queue[self.head];

        self.head += 1;
        self.head %= EVENT_QUEUE_SIZE;
        self.num_events -= 1;

        evt
    }

    pub fn has_events(&self) -> bool {
        self.num_events > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_behavior_from_integer() {
        // Out of bounds error
        let result = behavior_from_integer(9999).is_err();
        assert!(result);
    }

    #[test]
    fn test_event_queue() {
        let mut queue = GestureEventQueue::new();
        queue.enqueue_scalar(123.0);
        queue.enqueue_scalar(456.0);
        assert_eq!(queue.num_events, 2);

        let evt1 = queue.dequeue();

        let result = matches!(evt1.evtype, GestureEventType::Scalar);
        assert!(result);

        assert!(evt1.data.scalar.is_some());

        if let Some(x) = evt1.data.scalar {
            assert_eq!(x, 123.0);
        }

        assert_eq!(queue.num_events, 1);
        let evt2 = queue.dequeue();
        assert!(evt2.data.scalar.is_some());

        let result = matches!(evt2.evtype, GestureEventType::Scalar);
        assert!(result);

        if let Some(x) = evt2.data.scalar {
            assert_eq!(x, 456.0);
        }

        assert_eq!(queue.num_events, 0);
    }

    #[test]
    fn test_eventful_gesture() {
        let mut evtgst = EventfulGesture::default();
        let mut phs = 0.;
        let inc = 0.1;
        evtgst.scalar(60.);
        evtgst.rate([2, 3]);
        evtgst.behavior(Behavior::Linear);

        evtgst.preinit();
        let x = evtgst.tick(phs);
        let vtx = evtgst.vtx;

        assert_eq!(vtx.val, 60.0, "vertex was not set");
        assert_eq!(x, 60.0, "tick did not produce expected result");

        assert!(matches!(vtx.bhvr, Behavior::Linear), "wrong behavior found");
        assert_eq!(vtx.num, 2, "invalid rate: numerator");
        assert_eq!(vtx.den, 3, "invalid rate: denominator");
        // gesture states should match, since the internal
        // vertex has not yet been updated

        let prev = evtgst.gest.prev;
        let next = evtgst.gest.next;
        assert_eq!(prev, next, "states do not match");

        phs += inc;

        evtgst.scalar(65.);

        evtgst.tick(phs);
        phs += inc;

        let prev = evtgst.gest.prev;
        let next = evtgst.gest.next;
        assert_eq!(prev, next, "states do not match after first tick");

        let mut running = true;
        let mut lphs = evtgst.gest.lphs;
        let mut count = 0;

        // Wait until next period, then make sure
        // gesture is updated
        while running {
            evtgst.tick(phs);
            phs += inc;
            if phs > 1.0 {
                phs -= 1.0;
            }
            if lphs > evtgst.gest.lphs {
                // new period found check and see if event updated
                running = false;
            }
            lphs = evtgst.gest.lphs;
            count += 1;

            assert!(count < 20, "probably an unbounded loop");
        }

        assert_eq!(count, 14, "wrong sample count");

        let prev = evtgst.gest.prev;
        let next = evtgst.gest.next;
        assert_eq!(prev, 60.0, "wrong state value: prev");
        assert_eq!(next, 65.0, "wrong state value: next");
    }

    #[test]
    fn test_wait_event() {
        let mut evtgst = EventfulGesture::default();
        let mut phs = 0.;
        let inc = 0.1;

        evtgst.scalar(60.);
        evtgst.rate([1, 1]);
        evtgst.behavior(Behavior::Linear);

        evtgst.preinit();

        // Start of Period A
        println!("start of Period A");
        let mut count = 0;
        let mut running = true;
        let mut lphs = -1.;

        while running {
            evtgst.tick(phs);
            phs += inc;
            if phs > 1.0 {
                phs -= 1.0;
            }
            if lphs >= 0. && lphs > evtgst.gest.lphs {
                // new period found check and see if event updated
                running = false;
            }
            lphs = evtgst.gest.lphs;

            count += 1;

            if count == 1 {
                evtgst.wait(1);
                evtgst.scalar(65.);
            }

            assert!(count < 20, "probably an unbounded loop");
        }

        // Start of Period B
        println!("start of Period B");
        let mut count = 0;
        let mut running = true;

        // make sure next scalar didn't set to be 65
        assert_ne!(
            evtgst.gest.next, 65.0,
            "Wait did not wait a period as it ought to."
        );

        while running {
            evtgst.tick(phs);
            phs += inc;
            if phs > 1.0 {
                phs -= 1.0;
            }
            if lphs >= 0. && lphs > evtgst.gest.lphs {
                // new period found check and see if event updated
                running = false;
            }
            lphs = evtgst.gest.lphs;
            count += 1;

            assert!(count < 20, "probably an unbounded loop");
        }

        // Start of Period C
        assert_eq!(
            evtgst.gest.next, 65.0,
            "Expected scalar to have been set by now."
        );
    }
}
