#[warn(clippy::all)]
use super::repoview::*;
use log::warn;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct RepoViewNode {
    pub repository: String,
    pub author: String,
    pub date_created: String,
    pub location: String,
    pub company: String,
    pub organizations: Vec<String>,
}

impl RepoViewNode {
    // Make generic...eventually.
    // Maybe take an Iterator?
    pub fn parse_nodes(data: &Vec<repo_view::ResponseData>) -> Vec<RepoViewNode> {
        let mut parsed: Vec<RepoViewNode> = Vec::new();

        /*for unparsed in data.iter() {
            match unparsed.repository {
                Some(ref repo) => repo.pull_requests.edges.as_ref().and_then(|pr_edges| {
                    pr_edges.iter().map(|pr_edge| {
                        pr_edge.unwrap().node.and_then(|pr_node| {
                            pr_node.participants.edges.iter().for_each(|part_edge| {
                                part_edge.node.and_then(|user| {
                                    parsed.push(RepoViewNode {
                                        repository: repo.name_with_owner.clone(),
                                        author: user.login.clone(),
                                        date_created: "".to_owned(),
                                        location: user.location.unwrap_or_default().clone(),
                                        company: user.company.unwrap_or_default().clone(),
                                        organizations: user
                                            .organizations
                                            .nodes
                                            .unwrap_or_default()
                                            .clone(),
                                    })
                                })
                            })
                        })
                    })
                }),
                None => warn!("Empty data found in parsing. Data: {:#?}", unparsed),
            }
        }*/

        // Let's be honest. Query languages are the worst.

        // This is the ugliest code.
        for unparsed in data.iter() {
            match unparsed.repository {
                Some(ref repo) => {
                    if let Some(ref pr_edges) = repo.pull_requests.edges {
                        for pr_edge_thing in pr_edges {
                            if let Some(pr_edge_thing_edge) = pr_edge_thing {
                                if let Some(ref pr_edge_node) = pr_edge_thing_edge.node {
                                    if let Some(ref part_edges) = pr_edge_node.participants.edges {
                                        for gimme_node_damn in part_edges {
                                            if let Some(what_is_this) = gimme_node_damn {
                                                if let Some(ref finally_node) = what_is_this.node {
                                                    parsed.push(RepoViewNode {
                                                        repository: repo.name_with_owner.clone(),
                                                        author: finally_node.login.clone(),
                                                        date_created: "".to_owned(),
                                                        location: finally_node
                                                            .location
                                                            .as_ref()
                                                            .unwrap_or_default()
                                                            .clone(),
                                                        company: finally_node
                                                            .company
                                                            .unwrap_or_default()
                                                            .clone(),
                                                        organizations: finally_node
                                                            .organizations
                                                            .nodes
                                                            .unwrap()
                                                            .iter()
                                                            .map(|junk| junk.unwrap().login)
                                                            .collect(),
                                                    })
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                None => warn!("Empty data found in parsing. Data: {:#?}", unparsed),
            }
        }

        parsed
    }
}
