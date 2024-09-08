use std::{
    collections::{HashMap, HashSet},
    io::Write,
};

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
        messages: HashSet<usize>,
    },
}

struct BroadcastNode {
    pub node: String,
    pub id: usize,
    pub messages: HashSet<usize>,
    known: HashMap<String, HashSet<usize>>,
    msg_communicated: HashMap<usize, HashSet<usize>>,
    neighborhood: Vec<String>,
}

impl Node<(), Payload> for BroadcastNode {
    fn from_init(state: (), init: sog::Init) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            node: init.node_id,
            id: 1,
            messages: HashSet::new(),
            known: init
                .node_ids
                .into_iter()
                .map(|nid| (nid, HashSet::new()))
                .collect(),
            msg_communicated: HashMap::new(),
            neighborhood: Vec::new(),
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
                self.messages.insert(message);
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
            Payload::Topology { mut topology } => {
                self.neighborhood = topology
                    .remove(&self.node)
                    .unwrap_or_else(|| panic!("nope"));
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
