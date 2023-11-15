use crate::hdi;

use std::collections::BTreeMap;
use hdi::prelude::*;



// Trait for common fields
/// Common fields that are expected on some entry structs
pub trait CommonFields<'a> {
    /// A timestamp that indicates when the original create entry was made
    fn published_at(&'a self) -> &'a u64;
    /// A timestamp that indicates when this entry was created
    fn last_updated(&'a self) -> &'a u64;
    /// A spot for holding data that is not relevant to integrity validation
    fn metadata(&'a self) -> &'a BTreeMap<String, rmpv::Value>;
}

/// Auto-implement the [`CommonFields`] trait
///
/// The input must be a struct with fields matching each common field method.
///
/// #### Example
/// ```ignore
/// struct PostEntry {
///     pub message: String,
///
///     // Common fields
///     pub published_at: u64,
///     pub last_updated: u64,
///     pub metadata: BTreeMap<String, rmpv::Value>,
/// }
/// common_fields!( PostEntry );
/// ```
#[macro_export]
macro_rules! common_fields {
    ( $name:ident ) => {
        impl<'a> CommonFields<'a> for $name {
            fn published_at(&'a self) -> &'a u64 {
                &self.published_at
            }
            fn last_updated(&'a self) -> &'a u64 {
                &self.last_updated
            }
            fn metadata(&'a self) -> &'a BTreeMap<String, rmpv::Value> {
                &self.metadata
            }
        }
    };
}



//
// Group Entry
//
/// An entry struct for defining a group and its members
#[hdk_entry_helper]
#[derive(Clone)]
pub struct GroupEntry {
    /// The list of agents with admin authority in this group
    pub admins: Vec<AgentPubKey>,
    /// The list of agents with write authority in this group
    pub members: Vec<AgentPubKey>,
    /// An indicator of whether this group is still active
    pub deleted: Option<bool>,

    // common fields
    pub published_at: u64,
    pub last_updated: u64,
    pub metadata: BTreeMap<String, rmpv::Value>,
}
common_fields!( GroupEntry );

impl GroupEntry {
    /// Get a list of the admins and members of this group
    pub fn contributors(&self) -> Vec<AgentPubKey> {
        vec![ self.admins.clone(), self.members.clone() ]
            .into_iter()
            .flatten()
            .collect()
    }

    /// Check if the given agent is an admin or member
    pub fn is_contributor(&self, agent: &AgentPubKey) -> bool {
        self.contributors().contains( agent )
    }

    /// Check if the given agent is an admin
    pub fn is_admin(&self, agent: &AgentPubKey) -> bool {
        self.admins.contains( agent )
    }

    /// Check if the given agent is a member (not an admin)
    pub fn is_member(&self, agent: &AgentPubKey) -> bool {
        self.admins.contains( agent )
    }

    /// Return the differences between this group and the given group
    pub fn contributors_diff(&self, other: &GroupEntry) -> ContributorsDiff {
        let added: Vec<AgentPubKey> = other.contributors()
            .into_iter()
            .filter(|pubkey| !self.is_contributor(pubkey) )
            .collect();

        let removed: Vec<AgentPubKey> = self.contributors()
            .into_iter()
            .filter(|pubkey| !other.is_contributor(pubkey) )
            .collect();

        let intersection: Vec<AgentPubKey> = self.contributors()
            .into_iter()
            .filter(|pubkey| other.is_contributor(pubkey) )
            .collect();

        ContributorsDiff {
            added,
            removed,
            intersection,
        }
    }
}

/// The result of a group comparison
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContributorsDiff {
    pub added: Vec<AgentPubKey>,
    pub removed: Vec<AgentPubKey>,
    pub intersection: Vec<AgentPubKey>,
}



//
// Group Member Anchor Entry
//
/// An entry struct (anchor) representing a group contributor's personal anchor
#[hdk_entry_helper]
#[derive(Clone)]
pub struct ContributionsAnchorEntry( pub ActionHash, pub AgentPubKey );

impl ContributionsAnchorEntry {
    /// Get the agent pubkey of this auth anchor
    pub fn author(&self) -> &AgentPubKey {
        &self.1
    }

    /// Get the group revision (action hash) of this auth anchor
    pub fn group(&self) -> &ActionHash {
        &self.0
    }
}



//
// Group Member Archive Anchor Entry
//
/// An entry struct (anchor) representing a former authority of a group
#[hdk_entry_helper]
#[derive(Clone)]
pub struct ArchivedContributionsAnchorEntry( String, pub ActionHash, pub AgentPubKey );

impl ArchivedContributionsAnchorEntry {
    pub fn new(group_id: ActionHash, agent: AgentPubKey) -> Self {
        ArchivedContributionsAnchorEntry("archive".to_string(), group_id, agent)
    }
}

impl ArchivedContributionsAnchorEntry {
    /// Get the agent pubkey of this auth anchor
    pub fn author(&self) -> &AgentPubKey {
        &self.2
    }

    /// Get the group revision (action hash) of this auth anchor
    pub fn group(&self) -> &ActionHash {
        &self.1
    }
}


/// An enum that represents an authority anchor (active/archived)
#[hdk_entry_helper]
#[serde(untagged)]
#[derive(Clone)]
pub enum ContributionAnchors {
    Active(ContributionsAnchorEntry),
    Archive(ArchivedContributionsAnchorEntry),
}

impl ContributionAnchors {
    /// Get the agent pubkey of this auth anchor
    pub fn author(&self) -> &AgentPubKey {
        match &self {
            ContributionAnchors::Active(anchor) => &anchor.1,
            ContributionAnchors::Archive(anchor) => &anchor.2,
        }
    }

    /// Get the group revision (action hash) of this auth anchor
    pub fn group(&self) -> &ActionHash {
        match &self {
            ContributionAnchors::Active(anchor) => &anchor.0,
            ContributionAnchors::Archive(anchor) => &anchor.1,
        }
    }

    /// Determine if this enum's item is [`ContributionAnchors::Archive`]
    pub fn is_archive(&self) -> bool {
        match &self {
            ContributionAnchors::Active(_) => false,
            ContributionAnchors::Archive(_) => true,
        }
    }
}


/// Indicates the intended contributions anchor type
///
/// Since the variable content is the same for both anchor types, this enum helps declare the
/// intended type when passing around the group/author anchor values.
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum ContributionAnchorTypes {
    Active,
    Archive,
}

impl<'de> serde::Deserialize<'de> for ContributionAnchorTypes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let input : Option<String> = Deserialize::deserialize(deserializer)?;

        Ok(
            match input {
                Some(name) => match name.to_lowercase().as_str() {
                    "active" => ContributionAnchorTypes::Active,
                    "archive" | "inactive" => ContributionAnchorTypes::Archive,
                    lw_name => Err(serde::de::Error::custom(
                        format!("No match for '{}' in ContributionAnchorTypes enum", lw_name )
                    ))?,
                },
                None => ContributionAnchorTypes::Active,
            }
        )
    }
}
