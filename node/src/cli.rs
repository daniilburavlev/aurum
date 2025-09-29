use crate::node::Node;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = "Aurum blockchain node")]
pub struct NodeCli {
    #[arg(long, value_name = "port")]
    port: Option<i32>,
    #[arg(long, value_name = "wallet")]
    wallet: Option<String>,
    #[arg(long, value_name = "nodes")]
    nodes: Option<Vec<String>>,
    #[command(subcommand)]
    command: Option<NodeCmd>,
}

#[derive(Subcommand)]
pub enum NodeCmd {
    Init {
        #[arg(long, value_name = "path")]
        path: String,
    },
}

async fn start_node(port: Option<i32>, wallet: Option<String>, nodes: Option<Vec<String>>) {
    let node = Node::new(port, wallet, nodes).await;
    node.start().await;
}

fn init_node(path: &str) {
    Node::init(path);
}

pub async fn start_cli() {
    let cli = NodeCli::parse();
    if let Some(cmd) = cli.command {
        match cmd {
            NodeCmd::Init { path } => init_node(&path),
        }
    } else {
        start_node(cli.port, cli.wallet, cli.nodes).await;
    }
}
