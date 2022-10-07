use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::{io, thread};
use std::time::Duration;

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;

fn main() {
    println!("Starting the client...");

    // Cliente TCP que envia as mensagens para o servidor
    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connet");
    // ** O que é [non]blocking?
    client.set_nonblocking(true).expect("Failed to initiate non-blocking");

    // Transmissor e Receptor de mensagens entre o
    // usuário (fora da thread) e o cliente (dentro da thread)
    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        // Cria um buffer para armazenar as mensagens recebidas pelo servidor
        let mut buff = vec![0; MSG_SIZE];
        // Lê a mensagem recebida pelo servidor
        match client.read_exact(&mut buff) {
            Ok(_) => {
                // Itera sobre a mensagem recebida, [u8], e
                // remove do buffer a parte que não foi utilizada com a mensagem
                let msg = buff
                    .into_iter()
                    .take_while(|&x| x != 0)
                    .collect::<Vec<_>>();
                println!("Message received {:?}", msg);
            },
            Err(err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Connection with server was severed");
                break;
            }
        }

        // [1] Para cada mensagem recebida de fora da thread e que deve ser enviada para o servidor,
        // redimensiona a mensagem e envia
        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("Writing to socket failed");
                println!("Mensage sent: {}", msg); // ** {:?}
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break,
        }

        thread::sleep(Duration::from_millis(100));
    });

    println!("Write a message:");
    loop {
        let mut buff = String::new();
        // Espera mensagem do usuário
        io::stdin().read_line(&mut buff).expect("Reading from stdin failed");
        let msg = buff.trim().to_string();
        // Fecha o cliente se for digitado ':quit', caso contrário,
        // tenta enviar a mensagem para a thread [1]
        if msg == ":quit" || tx.send(msg).is_err() { break }
    }
    println!("Bye!");
}
