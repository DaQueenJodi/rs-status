pub mod config;

use std::{
    env,
    process::Command,
    ptr,
    sync::{Arc, RwLock},
    thread::{self, Thread},
    time::Duration, fs::{self, File}, io::Write,
};

use x11::xlib::{Display, XCloseDisplay, XDefaultRootWindow, XFlush, XOpenDisplay, XStoreName};

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = config::read_config();
    let seperator = config.seperator;
    let bar = Bar::new(Seperator(seperator), thread::current());
    let bar = Arc::new(RwLock::new(bar));

    let display = unsafe { XOpenDisplay(ptr::null()) };
    let display = MyXDisplay { inner: display };
    let display = Arc::new(RwLock::new(display));

    let mut index = 0;
    for module in config.modules {
        let command = Execute::new(module.command, module.interval);
        let bar = bar.clone();
        bar.write().unwrap().output.push(String::new());
        thread::spawn(move || command_handler(bar, command, index));
        index += 1;
    }

    let display2 = display.clone();
    let bar2 = bar.clone();
    let update_thread;
    if args.contains(&"wayland".to_string()) {
        update_thread = thread::spawn(move || {
            wayland_update_handler(bar2);
        });
    } else {
        update_thread = thread::spawn(move || {
            update_hander(bar2, display2);
        });
    }

    bar.clone().write().unwrap().updater_handle = update_thread.thread().clone();

    update_thread.join().unwrap();
    unsafe {
        XCloseDisplay(display.read().unwrap().inner);
    }
}
struct Bar {
    seperator: Seperator,
    output: Vec<String>,
    updater_handle: Thread,
}

struct Seperator(String);

#[derive(Clone)]
struct Execute {
    command: String,
    args: Vec<String>,
    interval: Duration,
}

impl Execute {
    fn run(&self) -> String {
        let stdout = Command::new(self.command.clone())
            .args(self.args.clone())
            .output()
            .unwrap()
            .stdout;
        String::from_utf8(stdout).unwrap().trim().to_string()
    }
    fn new(command: String, interval: u64) -> Execute {
        let command = shellwords::split(&command).unwrap();
        Execute {
            command: command[0].clone(),
            args: command[1..].into(),
            interval: Duration::from_millis(interval),
        }
    }
}

impl Bar {
    fn new(seperator: Seperator, thread: Thread) -> Bar {
        Bar {
            seperator,
            output: Vec::new(),
            updater_handle: thread,
        }
    }
    fn get_status(&self) -> String {
        let mut s = String::new();
        for i in 0..self.output.len() {
            s += &self.output[i];
            if i < self.output.len() - 1 {
                // if last element, dont add a seperator
                s += &self.seperator.0;
            }
        }
        s
    }
    fn update(&self) {
        self.updater_handle.unpark();
    }
}

fn command_handler(bar: Arc<RwLock<Bar>>, command: Execute, index: usize) {
    loop {
        bar.write().unwrap().output[index] = command.run();
        bar.read().unwrap().update();
        if command.interval.is_zero() {
            break;
        }
        thread::sleep(command.interval);
    }
}

fn wayland_update_handler(bar: Arc<RwLock<Bar>>) {
    let path = env::var("XDG_RUNTIME_DIR").unwrap() + "/somebar-0";
    let mut file = File::create(path).unwrap();
    loop {
        thread::park();
        let output = bar.read().unwrap().get_status();
        write!(file, "status {}\n", output).unwrap();
        file.flush().unwrap();
        
    }
}

fn update_hander(bar: Arc<RwLock<Bar>>, display: Arc<RwLock<MyXDisplay>>) {
    let display = display.read().unwrap().inner;
    loop {
        thread::park();
        let output = bar.read().unwrap().get_status();
        let c_output = output.as_ptr() as *const i8;
        unsafe {
            XStoreName(display, XDefaultRootWindow(display), c_output);
            XFlush(display);
        }
    }
}

struct MyXDisplay {
    inner: *mut Display,
}

unsafe impl Send for MyXDisplay {}
unsafe impl Sync for MyXDisplay {}
