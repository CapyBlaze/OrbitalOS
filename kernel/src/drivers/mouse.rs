use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use futures_util::task::AtomicWaker;
use spin::Mutex;
use x86_64::instructions::port::Port;


static PACKET_QUEUE: OnceCell<ArrayQueue<[u8; 3]>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();
static PACKET_STATE: Mutex<MousePacketState> = Mutex::new(MousePacketState::new());


pub fn init() {
    initialize_controller();

    let _ = PACKET_QUEUE.try_init_once(|| ArrayQueue::new(1024));
}

fn initialize_controller() {
    unsafe {
        let mut command = Port::new(0x64);
        let mut data = Port::new(0x60);

        wait_input_empty(&mut command);
        command.write(0xAD_u8);
        wait_input_empty(&mut command);
        command.write(0xA7_u8);

        flush_output(&mut command, &mut data);

        wait_input_empty(&mut command);
        command.write(0x20_u8);
        wait_output_full(&mut command);
        let mut status: u8 = data.read();
        status = (status | 0x03) & !0x20;
        wait_input_empty(&mut command);
        command.write(0x60_u8);
        wait_input_empty(&mut command);
        data.write(status);

        wait_input_empty(&mut command);
        command.write(0xA8_u8);

        wait_input_empty(&mut command);
        command.write(0xAE_u8);

        mouse_write(0xF6);
        let _ = mouse_expect_ack();

        mouse_write(0xF4);
        let _ = mouse_expect_ack();
    }
}

unsafe fn wait_input_empty(command: &mut Port<u8>) {
    loop {
        let status: u8 = command.read();
        if status & 0x02 == 0 {
            break;
        }
    }
}

unsafe fn wait_output_full(command: &mut Port<u8>) {
    loop {
        let status: u8 = command.read();
        if status & 0x01 != 0 {
            break;
        }
    }
}

unsafe fn flush_output(command: &mut Port<u8>, data: &mut Port<u8>) {
    while {
        let status: u8 = command.read();
        status & 0x01 != 0
    } {
        let _ = data.read();
    }
}

fn mouse_write(byte: u8) {
    unsafe {
        let mut command = Port::new(0x64);
        let mut data = Port::new(0x60);

        wait_input_empty(&mut command);
        command.write(0xD4_u8);
        wait_input_empty(&mut command);
        data.write(byte);
    }
}

fn mouse_expect_ack() -> u8 {
    unsafe {
        let mut data = Port::new(0x60);
        let ack: u8 = data.read();
        ack
    }
}

fn mouse_read() -> u8 {
    unsafe {
        let mut data = Port::new(0x60);
        data.read()
    }
}

pub fn handle_interrupt() {
    let byte = mouse_read();

    let mut state = PACKET_STATE.lock();
    if let Some(packet) = state.push(byte) {
        if let Ok(queue) = PACKET_QUEUE.try_get() {
            let _ = queue.push(packet);
            WAKER.wake();
        }
    }
}

pub struct MousePacketStream;

impl MousePacketStream {
    pub fn new() -> Self {
        Self
    }
}

impl futures_util::stream::Stream for MousePacketStream {
    type Item = [u8; 3];

    fn poll_next(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Option<[u8; 3]>> {
        let queue = match PACKET_QUEUE.try_get() {
            Ok(queue) => queue,
            Err(_) => {
                WAKER.register(cx.waker());
                return core::task::Poll::Pending;
            }
        };

        if let Some(packet) = queue.pop() {
            return core::task::Poll::Ready(Some(packet));
        }

        WAKER.register(cx.waker());

        match queue.pop() {
            Some(packet) => {
                WAKER.take();
                core::task::Poll::Ready(Some(packet))
            }
            None => core::task::Poll::Pending,
        }
    }
}

struct MousePacketState {
    packet: [u8; 3],
    index: usize,
}

impl MousePacketState {
    const fn new() -> Self {
        Self {
            packet: [0; 3],
            index: 0,
        }
    }

    fn push(&mut self, byte: u8) -> Option<[u8; 3]> {
        if self.index == 0 && byte & 0x08 == 0 {
            return None;
        }

        self.packet[self.index] = byte;
        self.index += 1;

        if self.index == 3 {
            self.index = 0;
            Some(self.packet)
        } else {
            None
        }
    }
}