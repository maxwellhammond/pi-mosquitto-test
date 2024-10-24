#[macro_use]
use std::{env, io, process, time::Duration};
extern crate paho_mqtt as mqtt;
use paho_mqtt::{properties, topic, TopicFilter};
/////////////////////////////////////////////////////////////////////////////


pub fn establish_broker(pod: &str) -> mqtt::AsyncClient {
    /*This function is used to establish the initial connection to the broker.
    A topic for the pod (pod/pod#) is automatically subscribed to for the pod controller to listen to all received messages from sensors. 
    Sensors/pins will subscribe to pod/pod#/sensor#*/

    //Host is statically assigned to the localhost on port 4832 for now.
    //TODO In the future, this will be assigned by the user on the frontend based on the address where the broker is running.
	let host = "localhost:4832";
	let chat_topic = format!("pod/{}", pod);
    let client_id = format!("podcontroller-{}", pod);
	
	const QOS: i32 = 1;
    const NO_LOCAL: bool = true;
	
	// The LWT is broadcast to the group if our connection is lost
    // But wait 30sec for reconnect before broadcasting it.
	let lwt_props = mqtt::properties! {
        mqtt::PropertyCode::WillDelayInterval => 10,
    };

    let lwt = mqtt::MessageBuilder::new()
        .topic(&chat_topic)
        .payload(format!("<<< {} disconnected >>>", pod))
        .qos(QOS)
        .properties(lwt_props)
        .finalize();
		
	// Create a client to the specified host, no persistence
	let create_opts = mqtt::CreateOptionsBuilder::new()
	.server_uri(host)
	.client_id(client_id)
	.persistence(None)
	.finalize();

    let cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|err| {
        eprintln!("Error creating the client: {}", err);
        process::exit(1);
    });
	
	// Session will exist for a day (86,400 sec) between connections.
    let props = mqtt::properties! {
        mqtt::PropertyCode::SessionExpiryInterval => 86400,
    };
	
    // Connect with a persistent sesstion

    // Connect with MQTT v5 and a persistent server session (no clean start).
    // For a persistent v5 session, we must set the Session Expiry Interval
    // on the server. Here we set that requests will persist for a day
    // (86,400sec) if the service disconnects or restarts.
    let conn_opts = mqtt::ConnectOptionsBuilder::new_v5()
        .keep_alive_interval(Duration::from_secs(20))
        .clean_start(false)
        .properties(props)
        .will_message(lwt)
        .finalize();

    // Set a closure to be called when the client loses the connection.
    // It will simply end the session.
    cli.set_connection_lost_callback(|_cli| {
        println!("*** Connection lost ***");
        process::exit(2);
    });

    // Attach a closure to the client to receive callbacks on incoming
    // messages. Just print them to the console.
    cli.set_message_callback(|_cli, msg| {
        if let Some(msg) = msg {
            println!("{}", msg.payload_str());
        }
    });
	
	let topic = mqtt::Topic::new(&cli, chat_topic, QOS);

    // Connect and wait for it to complete or fail

    if let Err(err) = cli.connect(conn_opts).wait() {
        eprintln!("Unable to connect: {}", err);
        process::exit(1);
    }

    // Subscribe to the pods topic channel
    println!("Joining the group '{}'...", pod);
    topic.subscribe_with_options(NO_LOCAL, None).wait().unwrap();

    // Broadcast that the new pod is connected
    topic
        .publish(format!("<<< {} is online >>>", pod))
        .wait()
        .unwrap();

	//Exit the function with the client details	
    return cli
}

pub fn publish_message (message: String, pod: String, sensor: String, broker: mqtt::AsyncClient) -> mqtt::Result<()>{
     
    //let mut message = String::new();
    let chat_user = format!("{}/{}", pod, sensor);
    let chat_msg = format!("{}: {}", chat_user, message);
    let chat_topic = format!("pod/{}/sensor{}", pod, sensor);
    let topic_filter = chat_topic.clone();

    //Check if sensors topic exists already or if it needs to be subscribed to
    let filter = mqtt::TopicFilter::new(topic_filter);
    if mqtt::TopicFilter::is_match(&self, filter) {
        
    } else {
        let topic = mqtt::Topic::new(&broker, chat_topic, 1);
        topic.subscribe_with_options(true, None).wait().unwrap();
    }
    
    if let Err(err) = topic.publish(chat_msg).wait() {
        eprintln!("Error: {}", err);
    }

    Ok(())
}