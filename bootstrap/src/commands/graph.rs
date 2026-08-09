use dkg::DkgService;
use shared_kernel::prelude::*;
use uuid::Uuid;

pub struct GraphCommand;

impl Command for GraphCommand {
    fn name(&self) -> &'static str {
        "graph"
    }

    fn description(&self) -> &'static str {
        "Manage the knowledge graph (DKG). Usage: graph node <label> | graph edge <from> <to> <relation> | graph list | graph neighbors <id>"
    }

    fn execute(
        &self,
        ctx: &CommandContext,
        args: &[&str],
    ) -> String {
        let Some(dkg) = ctx.services.resolve::<DkgService>() else {
            return "DKG service not available.".to_string();
        };

        match args.first().copied() {
            Some("node") => {
                if args.len() < 2 {
                    return "Usage: graph node <label>".to_string();
                }

                let id = Uuid::new_v4();

                match dkg.add_node(id, args[1]) {
                    Ok(()) => format!("Node '{}' created with id {id}", args[1]),
                    Err(err) => format!("Error: {err}"),
                }
            }

            Some("edge") => {
                if args.len() < 4 {
                    return "Usage: graph edge <from> <to> <relation>".to_string();
                }

                let (Ok(from), Ok(to)) = (
                    Uuid::parse_str(args[1]),
                    Uuid::parse_str(args[2]),
                ) else {
                    return "Invalid node id(s). Use ids returned by 'graph node'.".to_string();
                };

                match dkg.add_edge(from, to, args[3]) {
                    Ok(id) => format!("Edge '{}' created with id {id}", args[3]),
                    Err(err) => format!("Error: {err}"),
                }
            }

            Some("list") => {
                let nodes = dkg.list_nodes();
                let edges = dkg.list_edges();

                let mut lines = vec![format!("Nodes ({}):", nodes.len())];

                lines.extend(
                    nodes.iter().map(|n| format!("  {} | {}", n.id, n.label)),
                );

                lines.push(format!("Edges ({}):", edges.len()));

                lines.extend(
                    edges
                        .iter()
                        .map(|e| format!("  {} -[{}]-> {}", e.from, e.relation, e.to)),
                );

                lines.join("\n")
            }

            Some("neighbors") => {
                if args.len() < 2 {
                    return "Usage: graph neighbors <id>".to_string();
                }

                let Ok(id) = Uuid::parse_str(args[1]) else {
                    return "Invalid node id.".to_string();
                };

                let neighbors = dkg.neighbors(id);

                if neighbors.is_empty() {
                    "No outgoing relationships.".to_string()
                } else {
                    neighbors
                        .iter()
                        .map(|(edge, node)| {
                            let target = node
                                .as_ref()
                                .map(|n| n.label.as_str())
                                .unwrap_or("<unknown node>");

                            format!("-[{}]-> {} ({})", edge.relation, target, edge.to)
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            }

            _ => {
                "Usage: graph node <label> | graph edge <from> <to> <relation> | graph list | graph neighbors <id>"
                    .to_string()
            }
        }
    }
}