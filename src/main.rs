/*Uses the mqtt.rs file to establish connection to the mosquitto broker using the podcontroller # identification
  Topics are automatically subscribed to by each sensor based on pin # and publishes pin change messages
  Server-side, these messages will be received by the client subscribed to each podcontroller and call AMR missions*/

#[macro_use]
use rppal::gpio::{Event, Gpio, Trigger};
use std::sync::mpsc::channel;
use std::{clone, thread};
use std::time::Duration;
use paho_mqtt;
mod gpiostatus;
mod mqtt;
use gpiostatus::print_status;
use mqtt::establish_broker;
use mqtt::publish_message;


struct MonitoredPin {
  pin_id: u8,
  input_pin: rppal::gpio::InputPin
}

struct Message {
  pin_id: u8,
  // trigger: rppal::gpio::Trigger
  event: Event
}

struct ClientProperties{
  pod: String,
  pensor: String
}

#[tokio::main]
async fn main() -> rppal::gpio::Result<()> {
  _ = print_status();
  println!("Watching GPIO for changes...");
  // Create a new GPIO object and get pins
  let gpio = Gpio::new()?;

  // signal debounce time to filter flicker
  let debounce_time = Some(Duration::from_millis(200));

  let mut monitored_pins: Vec<MonitoredPin> = Vec::new();
  
  // Set up a channel to receive GPIO events
  let (tx, rx) = channel();

  //Declare pod number as identifier.
  //TODO Eventually this will be decided by the user on the frontend
  let pod = "pod000";

  for n in 0..27{
    // channel
    let sender = tx.clone();
    // pin
    let mut pin= gpio.get(n)?.into_input_pulldown();

    // handler
    pin.set_async_interrupt(Trigger::Both, debounce_time, move |event|{
      println!("in handler {}", n);
      let msg = Message{
        pin_id: n,
        // trigger: event.trigger
        event: event
      };
      sender.send(msg).unwrap();
    });

    // save
    let mp = MonitoredPin{
      pin_id : n,
      input_pin : pin
    };
    monitored_pins.push(mp);
  }

  //Establish async connection to MQTT client
  println!("Connecting pod to MQTT...");
  let broker = mqtt::establish_broker(pod);

    // Loop to handle pin state changes
    loop {
        // Receive the signal sent from the interrupt
        if let Ok(msg) = rx.recv() {
            println!("rcv'ed message");
            match msg.event.trigger {
              Trigger::RisingEdge => {

                //create an outgoing message to send to the broker
                let high_msg_out = format!("Pin {} is high", msg.pin_id);
                let sensor = format!("{}", msg.pin_id);
                //publish message to broker with pod and sensor number
                let broker_client = broker.clone();
                let podval = pod.clone().to_string();
                let send = mqtt::publish_message(high_msg_out, podval, sensor, broker_client);
                
              },
              Trigger::FallingEdge => {
                let low_msg_out = format!("Pin {} is low", msg.pin_id);
                //Publish MQTT Message to "test"
                
              }
              Trigger::Disabled => todo!(),
              Trigger::Both => todo!(),
              }
          }
          
        // Optional sleep to avoid busy-waiting
        thread::sleep(Duration::from_millis(10));
    }
}  
  // Remove the interrupt handler when done
  // This won't be reached in this infinite loop, but it's good practice in real-world cases
  // pin.clear_async_interrupt()?;

  // Ok result for the main function
  // Ok(())
