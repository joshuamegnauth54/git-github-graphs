use super::repoview::*;
//use crate::error::{Error, Result};
use log::warn;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct RepoViewNode {
    pub repository: String,
    pub author: String,
    pub date_created: String,
    pub pull_req_title: String,
    pub location: String,
    pub company: String,
    pub organizations: Vec<String>,
}

impl RepoViewNode {
    // Pulls out each organization from the array of organizations listed by the user.
    fn organizations_to_vec(
        orgs: &repo_view::RepoViewRepositoryPullRequestsEdgesNodeParticipantsEdgesNodeOrganizations,
    ) -> Option<Vec<String>> {
        orgs.nodes.as_ref().map(|nodes_iter| {
            nodes_iter
                .iter()
                .map(|node_org| {
                    node_org
                        .as_ref()
                        .map(|org| org.login.clone())
                        .unwrap_or_else(|| String::from("NA"))
                })
                .collect()
        })
    }

    // My attempt to break up the rightward drift in destructuring and parsing the JSON output from
    // my GraphQL query. The following function builds RepoViewNodes from
    // RepoViewRepositoryPullRequestsEdgesNodeParticipantsEdges. Sorry, I just wanted to type that
    // again.
    // Participants refers to posters on the specific pull request. So, we take in repository and
    // DateTime String slices as those don't change per poster.
    fn participants_to_nodes(
        participants: &[Option<
            repo_view::RepoViewRepositoryPullRequestsEdgesNodeParticipantsEdges,
        >],
        repo: &str,
        created_at: &str,
        title: &str,
    ) -> Option<Vec<RepoViewNode>> {
        participants
            .iter()
            .map(|part_edges_opt| {
                // Individual ParticipantsEdges
                part_edges_opt.as_ref().and_then(|part_edges| {
                    // ParticipantsEdgesNode
                    // Constructs individual RepoViewNodes that are collected into a Vector.
                    part_edges.node.as_ref().map(|user| {
                        RepoViewNode {
                            repository: repo.to_owned(),
                            author: user.login.to_owned(),
                            date_created: created_at.to_owned(),
                            pull_req_title: title.to_owned(),
                            // Users don't have to specify a location/company/organizations so they
                            // must be handled reasonably.
                            location: user
                                .location
                                .as_ref()
                                .map_or_else(|| String::from("NA"), |location| location.clone()),
                            company: user
                                .company
                                .as_ref()
                                .map_or_else(|| String::from("NA"), |company| company.clone()),
                            organizations: RepoViewNode::organizations_to_vec(&user.organizations)
                                .unwrap_or_else(Vec::new),
                        } //End of RepoViewNode construction
                    }) // End of ParticipantsEdgesNode
                }) // End of individual ParticipantsEdges
            })
            .collect() // End of ParticipantsEdges
    }

    // Traverses the pull requests to build Vectors of RepoViewNodes. The Vectors are constructed
    // from Vectors of Vectors of RepoViewNodes.
    fn pull_reqs_vec(
        pr_edges_vec: &[Option<repo_view::RepoViewRepositoryPullRequestsEdges>],
        repo: &str,
    ) -> Vec<RepoViewNode> {
        // RepoViewRepositoryPullRequestsEdges iterator
        // (As well as the actual Edges)
        pr_edges_vec
            .iter()
            .map(|pr_edge_option| {
                // RepoViewRepositoryPullRequestsEdges
                pr_edge_option.as_ref().and_then(|pr_edges| {
                    // RepoViewRepositoryPullRequestsEdgesNode
                    pr_edges.node.as_ref().and_then(|pr_edges_node| {
                        // ParticipantsEdges iter
                        pr_edges_node
                            .participants
                            .edges
                            .as_ref()
                            .map(|part_edges_iter| {
                                RepoViewNode::participants_to_nodes(
                                    &part_edges_iter,
                                    repo,
                                    &pr_edges_node.created_at,
                                    &pr_edges_node.title,
                                )
                            }) // End of ParticipantsEdges iter
                    }) // End of RepoViewRepositoryPullRequestsEdgesNode
                }) // End of RepoViewRepositoryPullRequestsEdges
            }) // End of RepoViewRepositoryPullRequestsEdges iterator and the map
            // We're left with an Option<Option<Vec<RepoViewNode>>> for EACH pull request which is
            // very messy. A more erudite programmer may handle all of this better, but I've found
            // that using a flat_map to flatten the structure then flattening again followed by
            // getting the values (i.e. not references) of each RepoViewNode via into_iter then
            // flattening that iterator so collect may pick up everything from each Vec.
            .flat_map(|nested_option| nested_option.flatten().into_iter().flatten())
            .collect()
    }

    pub fn parse_nodes(data: &[repo_view::ResponseData]) -> Vec<RepoViewNode> {
        let mut parsed: Vec<RepoViewNode> = Vec::new();

        for unparsed in data.iter() {
            match unparsed.repository {
                // RepoViewRepository
                Some(ref repo) => {
                    let maybe_reponodes: Option<Vec<RepoViewNode>> =
                    // RepoViewRepositoryPullRequests (and Option<[...]Edges>)
                    repo.pull_requests.edges.as_ref().map(|pr_edges_vec| {
                        RepoViewNode::pull_reqs_vec(&pr_edges_vec, &repo.name_with_owner)
                    }); // End of RepoViewRepositoryPullRequests
                    if let Some(reponodes) = maybe_reponodes {
                        parsed.extend(reponodes);
                    }
                } // End of Some(ref repo)
                None => warn!("Empty data found while parsing. Data: {:#?}", unparsed),
            }
        }
        parsed
    }
}
