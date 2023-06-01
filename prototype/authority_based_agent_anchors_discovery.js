
console.log("Integrity Modeling - Authority-based Discovery\n");

import crypto				from 'crypto';
import { expect }			from 'chai';
import { faker }			from '@faker-js/faker';
import {
    HoloHash, AgentPubKey,
    ActionHash, EntryHash,
    AnyLinkableHash,
}					from '@whi/holo-hash';
import {
    Cell,
}					from '@whi/holochain-prototyping';

import coordinator			from './coordinators/agent_anchor_links.js';
import { dna }				from './dna.js';
import { groupInput,
	 subjectInput }			from './entries.js';


function expect_failure ( cb, message ) {
    let failed				= false;
    try {
	cb()
    } catch (err) {
	failed				= true;
	try {
	    expect( String(err)		).to.have.string( message );
	} catch ( expect_err ) {
	    console.log( err );
	    throw expect_err;
	}
    }

    if ( failed === false )
	throw new Error(`Failure test did not fail`);
}


// Setup for tests
const agent_1 			= new AgentPubKey( crypto.randomBytes(32) ).addTag("A1");
const agent_2			= new AgentPubKey( crypto.randomBytes(32) ).addTag("A2");
const agent_3			= new AgentPubKey( crypto.randomBytes(32) ).addTag("A3");
const agent_4			= new AgentPubKey( crypto.randomBytes(32) ).addTag("A4");

const cell_1			= new Cell( agent_1, dna );
const cell_2			= new Cell( agent_2, dna );
const cell_3			= new Cell( agent_3, dna );
const cell_4			= new Cell( agent_4, dna );


cell_1.injectCoordinator( 0, "main", coordinator );
cell_2.injectCoordinator( 0, "main", coordinator );
cell_3.injectCoordinator( 0, "main", coordinator );
cell_4.injectCoordinator( 0, "main", coordinator );


function updateSubject ( cell, action, message ) {
    return cell.callZomeFunction("main", "update_subject", {
	"base": action,
	"changes": {
	    "message": message || faker.lorem.sentence(),
	},
    });
}

function updateGroup ( cell, action, message ) {
    return cell.callZomeFunction("main", "update_group", {
	"base": action,
	"changes": {
	    "admins": [],
	    "members": [],
	},
    });
}


// Phase 1

const group_1_input		= groupInput( agent_1, agent_2 );
const group_1_id		= cell_1.callZomeFunction("main", "create_group", group_1_input )
      .addTag("G1");

console.log("Created group: %s", group_1_id.toString(true) );


const subject_1_input		= subjectInput( group_1_id );
const content_1_id		= cell_1.callZomeFunction("main", "create_subject", subject_1_input )
      .addTag("C1");

console.log("Created content 1: %s", content_1_id.toString(true) );

const subject_2_input		= subjectInput( group_1_id );
const content_2_id		= cell_1.callZomeFunction("main", "create_subject", subject_2_input )
      .addTag("C2");

console.log("Created content 2: %s", content_2_id.toString(true) );

const content_3_id		= cell_2.callZomeFunction("main", "create_subject", subjectInput( group_1_id ) )
      .addTag("C3");

console.log("Created content 3: %s", content_3_id.toString(true) );


const content_1a_action		= updateSubject( cell_2, content_1_id )
      .addTag("C1a");
console.log("C1a update: %s", content_1a_action.toString(true) );

const content_2aa_action	= updateSubject( cell_1, content_2_id )
      .addTag("C2aa");
console.log("C2aa update: %s", content_2aa_action.toString(true) );

// Expected Failures
//
// Static
// - A1 can update admins because it is the only one.
//   - Otherwise, A1 cannot change admins without countersigned entry
// - An agent cannot create links off of another agent's anchor
//   - eg. A2 cannot link off of [G1 + A1]
// - Any agent cannot change the author group associated with content
// - A content's author group cannot change using "Update", that is considered a new "Create"
// - A content's author group revision must be related to the author group ID
// - A1 cannot link off of a group revision to an agent anchor unless that agent is an authority in
//   the group revision
//
// Dynamic
// - A2 cannot update the group (G1)
// - A3 cannot update group content (C1)
// - A2 and A3 cannot create links off of the group (G1)

// expect_failure( () => {
//     updateSubject( cell_3, content_1_id );
// }, "does not have permission" );


{
    let subjects		= cell_2.callZomeFunction("main", "get_group_subjects", {
	"id": group_1_id,
    });
    console.log("Found %s subjects", Object.values(subjects).length );
    console.log("%s", JSON.stringify(subjects,null,4) );

    const actions		= new Set( Object.values( subjects ).map( entity => String(entity.action) ) );

    const expected_actions	= [
	content_1a_action,
	content_2aa_action,
	content_3_id,
    ].map( hash => String(hash) );
    expect( actions		).to.have.lengthOf( expected_actions.length );
    expect( actions		).to.have.all.keys( expected_actions );
}


expect_failure( () => {
    updateGroup( cell_2, group_1_id );
}, "must be in group admin list" );

expect_failure( () => {
    updateGroup( cell_1, group_1_id );
}, "Cannot change admin list without countersigning" );


// Phase 2

const group_1a_action		= cell_1.callZomeFunction("main", "update_group", {
    "base": group_1_id,
    "changes": {
	"members": [ agent_3 ],
    },
}).addTag("G1a");
console.log("G1a update: %s", group_1a_action.toString(true) );

console.log("Group: %s", cell_2.callZomeFunction("main", "get_group", group_1_id ).authorities );

const content_2ab_action	= updateSubject( cell_2, content_2_id )
      .addTag("C2ab");
console.log("C2ab update: %s", content_2ab_action.toString(true) );

const content_3a_action		= updateSubject( cell_2, content_3_id )
      .addTag("C3a");
console.log("C3a update: %s", content_3a_action.toString(true) );

const content_2b_action		= updateSubject( cell_3, content_2aa_action )
      .addTag("C2b");
console.log("C2b update: %s", content_2b_action.toString(true) );

const content_4_id		= cell_3.callZomeFunction("main", "create_subject", subjectInput( group_1_id ) )
      .addTag("C4");

console.log("Created content 4: %s", content_4_id.toString(true) );

// Expected Failures
//
// Dynamic
// - A2 cannot link off of the A2 archive anchor
// - A2 + A3 cannot create links off of G1a
// - A3 cannot update content (C2aa) from phase 1 without updating the content's author group revision
// - A2 cannot update content (C2b) because they are not in the group revision's authorities

{
    let subjects		= cell_2.callZomeFunction("main", "get_group_subjects", {
	"id": group_1_id,
    });
    console.log("Found %s subjects", Object.values(subjects).length );
    console.log("%s", JSON.stringify(subjects,null,4) );

    const actions		= new Set( Object.values( subjects ).map( entity => String(entity.action) ) );

    const expected_actions	= [
	content_1a_action,
	content_2b_action,
	content_3_id,
	content_4_id,
    ];
    console.log( expected_actions.map( hash => hash.toString(true) ) );
    console.log( Object.values( subjects ).map( entity => [ entity.author.toString(true), entity.action.toString(true) ] ) );
    expect( actions		).to.have.all.keys( expected_actions.map( hash => String(hash) ) );
    expect( actions		).to.have.lengthOf( expected_actions.length );
}


// Phase 2

const group_1b_action		= cell_1.callZomeFunction("main", "update_group", {
    "base": group_1a_action,
    "changes": {
	"members": [ agent_2, agent_3 ],
    },
}).addTag("G1b");
console.log("G1b update: %s", group_1b_action.toString(true) );

console.log("Group: %s", cell_2.callZomeFunction("main", "get_group", group_1_id ).authorities );

const content_4a_action		= updateSubject( cell_2, content_4_id )
      .addTag("C4a");
console.log("C4a update: %s", content_4a_action.toString(true) );

const content_5_id		= cell_2.callZomeFunction("main", "create_subject", subjectInput( group_1_id ) )
      .addTag("C5");

console.log("Created content 5: %s", content_5_id.toString(true) );

// Expected Failures
//
// Dynamic
// - ?

{
    let subjects		= cell_2.callZomeFunction("main", "get_group_subjects", {
	"id": group_1_id,
    });
    console.log("Found %s subjects", Object.values(subjects).length );
    console.log("%s", JSON.stringify(subjects,null,4) );

    const actions		= new Set( Object.values( subjects ).map( entity => String(entity.action) ) );

    const expected_actions	= [
	content_1a_action,
	content_2b_action,
	content_3a_action,
	content_4a_action,
	content_5_id,
    ];
    // console.log( expected_actions.map( hash => hash.toString(true) ) );
    // console.log( Object.values( subjects ).map( entity => [ entity.author.toString(true), entity.action.toString(true) ] ) );
    expect( actions		).to.have.all.keys( expected_actions.map( hash => String(hash) ) );
    expect( actions		).to.have.lengthOf( expected_actions.length );
}
