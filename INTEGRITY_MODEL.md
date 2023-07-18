[back to CONTRIBUTING.md](CONTRIBUTING.md)


# Integrity Model

One key realization in designing a pattern that would allow a group to evolve, without eroding or
contaminating its historical contributions, is that we can only control the pathways for discovery.

Every time the group evolves new pathways are created using the group's new state as the base.


## TL;DR

- Any agent can create a new group
- Any group admin can edit group members
- Any group contributor (admin or member) can create contributions for group content
- Contribution anchors are created for each contributor (ie. each unique agent/group pair)
- Content state is determined by following contribution links
- Links based from a contributions anchor can only be created by the matching agent whereas links
  based from an archived contributions anchor can only be made by admins of the corresponding group
- Each group entry has links to all of it's contributors (admins + members)
- When a contributor is removed from a group, their contribution links are copied to a corresponding
  archived anchor. Otherwise, contribution links do not change during group updates.

![](https://drive.google.com/a/webheroes.ca/thumbnail?sz=w1000&id=1gOWOThwxkcDEIqEbb9Gnrbeo8jcT8a12)



## Validation Logic

#### Agent Roles

- Agent — *any agent within the membrane*
- Group Admin — *agent's with permission to update the group and make contributions in the group*
- Group Member — *agent's with permission to make contributions in the group*
- Group Contributor — *either a 'Group Admin' or a 'Group Member'*

### Permissions by Role

- Agent
  - Create group
- Group Admin
  - Update group
  - Create contributions anchor for group auths
  - Create links to anchors
- Group Contributor
  - Create contribution link
  - Create contribution update link

### CRUD Rules

#### Entry Types
Entry creation limitations for entry types.

##### Group
- Anyone can create a new group
- Only group admins can update a group

##### Contributions Anchor
- No requirements for create
- Anchors cannot be updated

##### Archived Contributions Anchor
- No requirements for create
- Anchors cannot be updated


#### Link Types

##### Group

###### Agent —> Group
- Only the matching agent can create this link


##### Group Auth

###### Group —> Contribution Anchor
- Only admins of the group can create this link


##### Group Auth Archive

###### Group —> Archived Contribution Anchor
- Only admins of the group can create this link


##### Contribution

###### Contribution Anchor —> *[target]*
- Only the matching anchor agent can create this link

###### Archived Contribution Anchor —> *[target]*
- Only admins of the group can create this link


##### Contribution Update
- The link tag must be a UTF-8 string with 2 hashes (`AnyLinkableHash`) separated by `::`
  - eg. `<create hash>::<revision hash>`
  - If the hash types are `Action` then an additional check is made to ensure that the "create hash"
    is the root create of the "revision hash"

###### Contribution Anchor —> *[target]*
- Only the matching anchor agent can create this link

###### Archived Contribution Anchor —> *[target]*
- Only admins of the group can create this link



## Example #1
In this example narrative, we will go through the basic usage scenario that involves create and
update for a group, as well as create and update for contributions.

- Stage 1 - Initial group
- Stage 2 - Remove & Add Members
- Stage 3 - Re-add Previous Member

#### Diagram Legend
> If you cannot see the diagrams, try viewing this page in an incognito tab.

| Line format | Meaning                                     |
|-------------|---------------------------------------------|
| Red line    | *Valid paths from the latest group state*   |
| Grey line   | *Expired paths from the latest group state* |
| Dashed line | *Links that cannot be seen in valiation*    |
| Solid line  | *Pointers that can be seen in validation*   |


### Stage 1 - Initial group

- Group is created with
  - 1 admin [Agent 1] (creator of the group must be in the admin list)
  - 1 member [Agent 2]
- Each contributor has a unique base for their content links

| This diagram represents the entry relationships from the perspective of "Group 1" after stage 1      |
|------------------------------------------------------------------------------------------------------|
| ![](https://drive.google.com/a/webheroes.ca/thumbnail?sz=w1000&id=19ZovM5Zae1pcPDI6ziMOteW2rQxwsU5k) |


### Stage 2 - Remove & Add Members

- The group is updated
  - 1 member is removed [Agent 2]
  - 1 member is added [Agent 3]
- Each removed member's contributions are copied to a unique "archive" anchor

| This diagram represents the entry relationships from the perspective of "Group 1a" after stage 2     |
|------------------------------------------------------------------------------------------------------|
| ![](https://drive.google.com/a/webheroes.ca/thumbnail?sz=w1000&id=1xqEdR8kkDG4RFpupCJkoFrg8Ddn0JKb9) |


### Stage 3 - Re-add Previous Member

- The group is updated
  - 1 member is re-added [Agent 2]
- The re-added member's original active anchor is used instead of their archived anchor

| This diagram represents the entry relationships from the perspective of "Group 1b" after stage 2     |
|------------------------------------------------------------------------------------------------------|
| ![](https://drive.google.com/a/webheroes.ca/thumbnail?sz=w1000&id=1BEzmNg-fX53_apcpxA6Rmqno4kVTEtE4) |


#### Final Snapshot
As you can see in the diagram, all historical pathways are preserved and it is a matter of the
viewer choosing what perspective to follow.  When following a group, it makes sense to choose the
latest group revision as the perspective for resolving content state.  However, there are many
possible resolution patterns that can be designed using this integrity model.  For example, it would
be possible to design a coordinator that allows the viewer to override the contributors list for
their own perspective.

| This diagram represents the entry relationships state after stage 3                                  |
|------------------------------------------------------------------------------------------------------|
| ![](https://drive.google.com/a/webheroes.ca/thumbnail?sz=w1000&id=1CwWJ8nHt97IkPVNnxIpXrUqlN_y3F1cP) |


### Change admin(s)
> **WARNING:** *not implemented yet*

Changing the admin list requires counter-signing by other admins.



## Decision Logs
Architectural decisions (most importantly "why not" questions) that are ambiguous, or cannot be
deduced, when reading the code.

Please submit additional questions using GitHub issues.


#### Why not use the G1 ID (group create action hash) as the base for all content links?

TLDR;
1. Members added later will not have permission to link off that base

When links are created, validation can only see the group entry that is the base address.  Since
validation cannot see group updates, there is no way to see if members were added later (or removed
later).  This means that the base address must lead to the group revision that the validation needs
to see.


##### Why not use an additive approach by creating new links from the latest group revision?

TLDR;
1. Removed members would be able to pollute the base (ie. DDoS attack)

Though this would avoid redundancy and create the least amount of links, it would leave a
vulnerability for past members to pollute older entries with contribution links.  Perhaps this could
be solved with some form of rate-limiting, but it seemed simpler to have a little redundancy in
"group auth" links in order to avoid pollution.


#### Why and when to change the content author group ID?

TLDR;
1. Members added later will fail validation unless the "group ref" points to a revision where they
   have contribution authority.

When a group is updated, it is not required that you update all content's "group reference" to point
at the newest group revision.  However, when a newer member wants to create an update to some group
content, they will need to update the "group reference" so that it points to the group revision
where they are a contributor.

This leaves a little opening for past members to cause update pollution on any entries that have
"group references" pointing to where they had contributor authority.  This could be solved by
limiting each agent to creating only 1 update per entry so that the most pollution possible is no
more than the number of former (disgruntled) members.  It is also solved by using the shortcut
methods for contribution aggregation where it does not need to follow entry updates to determine the
latest state.


#### Why a contributions anchor uses the group ID but an archived anchor uses a group revision?

TLDR;
1. Validation rules for an archived member need to know the groups admin list; therefore, it needs
   to point at a specific group revision.

Unlike a conributions anchor, which only allow the matching agent to create links, the archive
anchors only allow admins of the group to make links.  The contributions anchor only has a group ID
to help an agent separate their contributions into group categories.

It is important for the admin who commits the group update to get a snapshot of the current
contributions of members being removed.  Any admin can make these contribution links at a later
time, but they wouldn't have the benefit of knowing exactly what the view was at the moment in time
when the group was updated.  Specifically, the attack we are preventing here is a former member
backdating some entry or links timestamp to a time before the group is updated.
