use debo_engine::{
    face::{Expression, Face},
    protocol::{parse_message, FaceCommand, MAX_MESSAGE_BYTES},
    renderer::Renderer,
    LOGICAL_HEIGHT, LOGICAL_WIDTH,
};
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc::{self, SyncSender, TrySendError},
    thread,
    time::{Duration, Instant},
};

const SCALE: usize = 5;
const TITLE: &str = "DEBO — custom software face engine";
const CONTROL_ADDRESS: &str = "127.0.0.1:45821";
const COMMAND_QUEUE_CAPACITY: usize = 64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let width = LOGICAL_WIDTH * SCALE;
    let height = LOGICAL_HEIGHT * SCALE;
    let mut window = Window::new(TITLE, width, height, WindowOptions { resize: false, ..WindowOptions::default() })?;
    window.limit_update_rate(Some(Duration::from_micros(16_667)));

    let (command_tx, command_rx) = mpsc::sync_channel(COMMAND_QUEUE_CAPACITY);
    start_control_server(command_tx);
    println!("DEBO face control listening on {CONTROL_ADDRESS} (JSON Lines, protocol v1)");

    let mut renderer = Renderer::new(LOGICAL_WIDTH, LOGICAL_HEIGHT, 0x080D16);
    let mut output = vec![0_u32; width * height];
    let mut face = Face::default();
    let mut last = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let dt = now.duration_since(last).as_secs_f32();
        last = now;

        // Commands are applied on the render thread; sockets and JSON parsing never block it.
        for command in command_rx.try_iter() {
            apply_command(&mut face, command);
        }

        if window.is_key_pressed(Key::Key1, KeyRepeat::No) { face.set_expression(Expression::Neutral); }
        if window.is_key_pressed(Key::Key2, KeyRepeat::No) { face.set_expression(Expression::Curious); }
        if window.is_key_pressed(Key::Key3, KeyRepeat::No) { face.set_expression(Expression::Happy); }
        if window.is_key_pressed(Key::Key4, KeyRepeat::No) { face.set_expression(Expression::Surprised); }
        if window.is_key_pressed(Key::Key5, KeyRepeat::No) { face.set_expression(Expression::Sleepy); }
        if window.is_key_pressed(Key::Space, KeyRepeat::No) { face.trigger_blink(); }
        if window.is_key_pressed(Key::Left, KeyRepeat::No) { face.gaze(-1.0, face.gaze_y); }
        if window.is_key_pressed(Key::Right, KeyRepeat::No) { face.gaze(1.0, face.gaze_y); }
        if window.is_key_pressed(Key::Up, KeyRepeat::No) { face.gaze(face.gaze_x, -1.0); }
        if window.is_key_pressed(Key::Down, KeyRepeat::No) { face.gaze(face.gaze_x, 1.0); }
        if window.is_key_pressed(Key::C, KeyRepeat::No) { face.gaze(0.0, 0.0); }
        if window.is_key_pressed(Key::Tab, KeyRepeat::No) { face.set_expression(face.expression.next()); }
        if window.is_key_pressed(Key::E, KeyRepeat::No) {
            let preset = face.cycle_eye_color();
            println!("Eye colour: {} (#{:06X})", preset.name, preset.color.packed());
        }

        face.tick(dt);
        face.render(&mut renderer);
        output.copy_from_slice(&renderer.upscale(SCALE));
        window.update_with_buffer(&output, width, height)?;
    }
    Ok(())
}

fn apply_command(face: &mut Face, command: FaceCommand) {
    match command {
        FaceCommand::SetExpression(expression) => face.set_expression(expression),
        FaceCommand::SetEyeColor(name) => { let _ = face.set_named_eye_color(&name); }
        FaceCommand::Gaze { x, y } => face.gaze(x, y),
        FaceCommand::Blink => face.trigger_blink(),
        FaceCommand::CentreGaze => face.gaze(0.0, 0.0),
    }
}

fn start_control_server(tx: SyncSender<FaceCommand>) {
    thread::spawn(move || {
        let listener = match TcpListener::bind(CONTROL_ADDRESS) {
            Ok(listener) => listener,
            Err(error) => {
                eprintln!("Could not start DEBO control server on {CONTROL_ADDRESS}: {error}");
                return;
            }
        };
        if let Err(error) = listener.set_nonblocking(true) {
            eprintln!("Could not configure DEBO control server: {error}");
            return;
        }
        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    let client_tx = tx.clone();
                    thread::spawn(move || handle_client(stream, client_tx));
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => thread::sleep(Duration::from_millis(10)),
                Err(error) => {
                    eprintln!("DEBO control accept error: {error}");
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }
    });
}

fn handle_client(mut stream: TcpStream, tx: SyncSender<FaceCommand>) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
    loop {
        let line = match read_bounded_line(&mut stream) {
            Ok(Some(line)) => line,
            Ok(None) => return,
            Err(reason) => {
                let _ = write_response(&mut stream, false, reason);
                return;
            }
        };
        match parse_message(&line) {
            Ok(command) => match tx.try_send(command) {
                Ok(()) => { if write_response(&mut stream, true, "queued").is_err() { return; } }
                Err(TrySendError::Full(_)) => { if write_response(&mut stream, false, "command_queue_full").is_err() { return; } }
                Err(TrySendError::Disconnected(_)) => { let _ = write_response(&mut stream, false, "face_unavailable"); return; }
            },
            Err(error) => {
                if write_response(&mut stream, false, error.0).is_err() { return; }
            }
        }
    }
}

/// Read one newline-terminated message without ever buffering more than the protocol limit.
fn read_bounded_line(stream: &mut TcpStream) -> Result<Option<String>, &'static str> {
    let mut bytes = Vec::with_capacity(128);
    let mut byte = [0_u8; 1];
    loop {
        match stream.read(&mut byte) {
            Ok(0) if bytes.is_empty() => return Ok(None),
            Ok(0) => break,
            Ok(_) if byte[0] == b'\n' => break,
            Ok(_) => {
                if bytes.len() >= MAX_MESSAGE_BYTES { return Err("message_too_large"); }
                if byte[0] != b'\r' { bytes.push(byte[0]); }
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock || error.kind() == std::io::ErrorKind::TimedOut => return Err("read_timeout"),
            Err(_) => return Err("read_error"),
        }
    }
    String::from_utf8(bytes).map(Some).map_err(|_| "invalid_utf8")
}

fn write_response(stream: &mut TcpStream, accepted: bool, status: &str) -> std::io::Result<()> {
    let response = serde_json::json!({ "version": 1, "accepted": accepted, "status": status });
    writeln!(stream, "{response}")
}
