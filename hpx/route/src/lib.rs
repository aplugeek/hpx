use radix_trie::Trie;
use serde::{de, Deserialize, Serialize};
use std::collections::HashMap;

use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

pub type RouteMap = HashMap<String, usize>;

pub type RouteRadixTrie = Trie<String, usize>;

pub type EndpointsMap = HashMap<String, RouteEndpoint>;

#[derive(Debug, Default)]
pub struct Route {
    pub servant: Vec<Servant>,
    pub rmap: RouteMap,
    pub rtrie: RouteRadixTrie,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Servant {
    pub name: String,
    pub servers: Vec<Server>,
    #[serde(skip_serializing, skip_deserializing)]
    pub state: Arc<ServantState>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ServantState {
    pub count: AtomicUsize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Server {
    pub addr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    weight: Option<i8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RouteKind {
    Precise,
    Fuzzy,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RouteEndpoint {
    #[serde(rename = "routes")]
    pub routes: Vec<RoutePath>,
    #[serde(rename = "endpoints")]
    pub endpoints: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoutePath {
    #[serde(rename = "path")]
    path: String,
    #[serde(rename = "kind", deserialize_with = "de_route_kind")]
    kind: RouteKind,
}

fn de_route_kind<'de, D>(deserializer: D) -> Result<RouteKind, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?.to_lowercase();
    let op = match s.as_str() {
        "precise" => RouteKind::Precise,
        "fuzzy" => RouteKind::Fuzzy,
        other => {
            return Err(de::Error::custom(format!(
                "Invalid query param kind '{}'",
                other
            )));
        }
    };
    Ok(op)
}

impl Route {
    pub fn from_endpoints(ep: &EndpointsMap) -> Result<Self, std::io::Error> {
        let mut rmap = RouteMap::new();
        let mut rtrie = RouteRadixTrie::new();
        let mut servants = Vec::new();
        let mut cursor = 0;
        ep.iter().for_each(|(k, v)| {
            let servers = v
                .endpoints
                .iter()
                .map(|server| Server {
                    addr: server.into(),
                    weight: None,
                })
                .collect::<Vec<Server>>();
            let servant = Servant {
                name: k.clone(),
                state: Arc::new(ServantState::default()),
                servers,
            };
            let index = cursor;
            servants.push(servant);
            cursor += 1;
            v.routes.iter().for_each(|r| {
                match r.kind {
                    RouteKind::Precise => rmap.insert(r.path.clone(), index),
                    RouteKind::Fuzzy => rtrie.insert(r.path.clone(), index),
                };
            });
        });
        Ok(Self {
            servant: servants,
            rmap,
            rtrie,
        })
    }
}
