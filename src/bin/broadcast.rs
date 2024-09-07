use std::{collections::HashMap, io::Write};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use sog::{main_loop, Body, Message, Node};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Broadcast {
        message: usize,
    },
    Read,
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    BroadcastOk,
    TopologyOk,
    ReadOk {
        messages: Vec<usize>,
    },
}

struct BroadcastNode {
    pub node: String,
    pub id: usize,
    pub messages: Vec<usize>,
}

impl Node<(), Payload> for BroadcastNode {
    fn from_init(state: (), init: sog::Init) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(BroadcastNode {
            node: init.node_id,
            id: 1,
            messages: Vec::new(),
        })
    }

    fn step(
        &mut self,
        input: sog::Message<Payload>,
        output: &mut std::io::StdoutLock,
    ) -> anyhow::Result<()> {
        let mut reply = input.into_reply(Some(&mut self.id));
        match reply.body.payload {
            Payload::Broadcast { message } => {
                reply.body.payload = Payload::BroadcastOk;
                self.messages.push(message);
                serde_json::to_writer(&mut *output, &reply)
                    .context("Could not write broadcastok")?;
                output.write_all(b"\n").context("write trailing newline")?;
                self.id += 1;
            }
            Payload::Read {} => {
                reply.body.payload = Payload::ReadOk {
                    messages: self.messages.clone(),
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("Could not write broadcastok")?;
                output.write_all(b"\n").context("write trailing newline")?;
                self.id += 1;
            }
            Payload::Topology { .. } => {
                reply.body.payload = Payload::TopologyOk {};
                serde_json::to_writer(&mut *output, &reply)
                    .context("Could not write broadcastok")?;
                output.write_all(b"\n").context("write trailing newline")?;
                self.id += 1;
            }
            Payload::BroadcastOk => {}
            Payload::ReadOk { messages } => {}
            Payload::TopologyOk => todo!(),
        }
        Ok(())
    }
}
fn main() -> anyhow::Result<()> {
    main_loop::<_, BroadcastNode, _>(())
}
