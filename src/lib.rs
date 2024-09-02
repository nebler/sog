use anyhow::Context;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::StdoutLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<Payload> {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: Body<Payload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body<Payload> {
    #[serde(rename = "msg_id")]
    pub id: Option<usize>,
    pub in_reply_to: Option<usize>,
    #[serde(flatten)]
    pub payload: Payload,
}

pub trait Node<S, Payload> {
    fn from_init(state: S, init: Init) -> anyhow::Result<Self>
    where
        Self: Sized;
    fn step(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Init {
    node_id: String,
    node_ids: Vec<String>,
}

pub trait Payload: Sized {
    fn extract_init(input: Self) -> Option<Init>;
    fn gen_init_ok() -> Self;
}

pub fn main_loop<S, N, P>(init_state: S) -> anyhow::Result<()>
where
    P: Payload + DeserializeOwned,
    N: Node<S, P>,
{
    let stdin = std::io::stdin().lock();
    let mut inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message<P>>();
    let mut stdout = std::io::stdout().lock();
    let init = inputs
        .next()
        .expect("no input message received")
        .context("init message could be deserailized")?;
    let init = P::extract_init(init.body.payload).expect("first message should be init");
    let mut node: N = Node::from_init(init_state, init).context("node inialization failed")?;
    for input in inputs {
        let input = input.context("Maelstrom input from STDIN could not be deserialized")?;
        node.step(input, &mut stdout)
            .context("Node step funciton failed")?;
    }

    Ok(())
}
