import { Logger }			from '@whi/weblogger';
const log				= new Logger("test-model-dna", process.env.LOG_LEVEL );

import fs				from 'node:fs';
import path				from 'path';
import crypto				from 'crypto';
import { expect }			from 'chai';
import { faker }			from '@faker-js/faker';
import json				from '@whi/json';
import { AgentPubKey, HoloHash,
	 ActionHash, EntryHash }	from '@whi/holo-hash';
import HolochainBackdrop		from '@whi/holochain-backdrop';
const { Holochain }			= HolochainBackdrop;
import {
    intoStruct,
    OptionType, VecType, MapType,
}					from '@whi/into-struct';

// const why				= require('why-is-node-running');
import { linearSuite,
	 expect_reject }		from '../utils.js';

const delay				= (n) => new Promise(f => setTimeout(f, n));
const __filename			= new URL(import.meta.url).pathname;
const __dirname				= path.dirname( __filename );
const TEST_DNA_PATH			= path.join( __dirname, "../model_dna.dna" );

const clients				= {};
const DNA_NAME				= "test_dna";

const DEBUG_ZOME			= "debug_csr";
const COOP_ZOME				= "coop_content_csr";
const GOOD_ZOME				= "basic_usage_csr";
const EVIL_ZOME				= "corrupt_csr";


const EntryCreationActionStruct		= {
    "type":			String,
    "author":			AgentPubKey,
    "timestamp":		Number,
    "action_seq":		Number,
    "prev_action":		ActionHash,
    "original_action_address":	OptionType( ActionHash ),
    "original_entry_address":	OptionType( EntryHash ),
    "entry_type": {
	"App": {
	    "entry_index":	Number,
	    "zome_index":	Number,
	    "visibility": {
		"Public":	null,
	    },
	},
    },
    "entry_hash":		EntryHash,
    "weight": {
	"bucket_id":		Number,
	"units":		Number,
	"rate_bytes":		Number,
    },
};

const GroupStruct		= {
    "admins":			VecType( AgentPubKey ),
    "members":			VecType( AgentPubKey ),

    "published_at":		Number,
    "last_updated":		Number,
    "metadata":			{},
};

const ContentStruct		= {
    "text":			String,
    "author":			AgentPubKey,
    "group_ref":		{
	"id": ActionHash,
	"rev": ActionHash,
    },

    "published_at":		Number,
    "last_updated":		Number,
};

function createGroupInput ( admins, ...members ) {
    return {
	"admins": admins,
	"members": [ ...members ],

	"published_at":		Date.now(),
	"last_updated":		Date.now(),
	"metadata":		{},
    };
};

function createContentInput ( author, group_id, group_rev ) {
    return {
	"text":			faker.lorem.sentence(),
	"author":		author,
	"group_ref": {
	    "id":		group_id,
	    "rev":		group_rev,
	},
	// "group_ref":		[ group_id, group_rev ],

	"published_at":		Date.now(),
	"last_updated":		Date.now(),
    };
};


let group, g1_addr, g1a_addr, g1b_addr;
let c1, c1_addr, c1a_addr;
let c2, c2_addr, c2a_addr, c2aa_addr, c2b_addr;
let c3, c3_addr, c3a_addr;
let c4, c4_addr, c4a_addr;
let c5, c5_addr;


function phase1_tests () {

    it("should create group via alice (A1)", async function () {
	const group_input		= createGroupInput(
	    [ clients.alice.cellAgent(), clients.emily.cellAgent(), clients.felix.cellAgent() ],
	    clients.bobby.cellAgent(),
	);
	g1_addr				= await clients.alice.call( DNA_NAME, COOP_ZOME, "create_group", group_input );
	log.debug("Group ID: %s", g1_addr );

	// expect( g1_addr		).to.be.a("ActionHash");
	expect( g1_addr		).to.be.a("Uint8Array");
	expect( g1_addr		).to.have.length( 39 );

	group				= intoStruct( await clients.alice.call( DNA_NAME, COOP_ZOME, "get_group", g1_addr ), GroupStruct );
	log.debug( json.debug( group ) );
    });

    it("should create content (C1 + C2) via alice (A1)", async function () {
	{
	    const content_input		= createContentInput( clients.alice.cellAgent(), g1_addr, g1_addr );
	    c1_addr			= await clients.alice.call( DNA_NAME, GOOD_ZOME, "create_content", content_input );
	    log.debug("C1 Address: %s", new ActionHash(c1_addr) );

	    expect( c1_addr		).to.be.a("Uint8Array");
	    expect( c1_addr		).to.have.length( 39 );

	    c1				= intoStruct( await clients.alice.call( DNA_NAME, GOOD_ZOME, "get_content", c1_addr ), ContentStruct );
	    log.debug( json.debug( c1 ) );
	}
	{
	    const content_input		= createContentInput( clients.alice.cellAgent(), g1_addr, g1_addr );
	    c2_addr			= await clients.alice.call( DNA_NAME, GOOD_ZOME, "create_content", content_input );
	    log.debug("C2 Address: %s", new ActionHash(c2_addr) );

	    expect( c2_addr		).to.be.a("Uint8Array");
	    expect( c2_addr		).to.have.length( 39 );

	    c2				= intoStruct( await clients.alice.call( DNA_NAME, GOOD_ZOME, "get_content", c2_addr ), ContentStruct );
	    log.debug( json.debug( c2 ) );
	}
    });

    it("should create content (C3) via bobby (A2)", async function () {
	{
	    const content_input		= createContentInput( clients.bobby.cellAgent(), g1_addr, g1_addr );
	    c3_addr			= await clients.bobby.call( DNA_NAME, GOOD_ZOME, "create_content", content_input );
	    log.debug("C3 Address: %s", new ActionHash(c3_addr) );

	    expect( c3_addr		).to.be.a("Uint8Array");
	    expect( c3_addr		).to.have.length( 39 );

	    c3				= intoStruct( await clients.alice.call( DNA_NAME, GOOD_ZOME, "get_content", c3_addr ), ContentStruct );
	    log.debug( json.debug( c3 ) );
	}
    });

    it("should update content (C1 => C1a) via bobby (A2)", async function () {
	{
	    c1a_addr			= await clients.bobby.call( DNA_NAME, GOOD_ZOME, "update_content", {
		"base": c1_addr,
		"entry": Object.assign( c1, {
		    "text":		"(updated) " + faker.lorem.sentence(),
		}),
	    });
	    log.debug("C1a Address: %s", new ActionHash(c1a_addr) );

	    expect( c1a_addr		).to.be.a("Uint8Array");
	    expect( c1a_addr		).to.have.length( 39 );
	}
    });

    it("should update content (C2 => C2a) via alice (A1)", async function () {
	{
	    c2a_addr			= await clients.alice.call( DNA_NAME, GOOD_ZOME, "update_content", {
		"base": c2_addr,
		"entry": Object.assign( c2, {
		    "text":		"(updated) " + faker.lorem.sentence(),
		}),
	    });
	    log.debug("C2a Address: %s", new ActionHash(c2a_addr) );

	    expect( c2a_addr		).to.be.a("Uint8Array");
	    expect( c2a_addr		).to.have.length( 39 );
	}
    });

    it("should get group content and find: C1a, C2a, C3", async function () {
	const targets			= new Set(
	    (await clients.carol.call( DNA_NAME, COOP_ZOME, "get_group_content_targets", g1_addr ))
		.map( addr => String(new HoloHash(addr)) )
	);
	log.debug("Group content targets: %s", targets );

	const expected_targets	= [
	    c1a_addr,
	    c2a_addr,
	    c3_addr,
	].map( addr => String(new HoloHash(addr)) );
	expect( targets			).to.have.all.keys( ...expected_targets );
	expect( targets			).to.have.lengthOf( expected_targets.length );
    });

    linearSuite( "Phase 1 - Checks", phase1_checks_tests );
}

function phase1_checks_tests () {

    // Static
    it("should reject group update because it requires counter-signing", async function () {
	await expect_reject( async () => {
	    await clients.alice.call( DNA_NAME, COOP_ZOME, "update_group", {
		"base": g1_addr,
		"entry": Object.assign({}, group, {
		    "admins": [ clients.alice.cellAgent() ],
		}),
	    });
	}, "requires counter-signing" ); // group admins cannot be changed without counter-signing
    });

    it("should reject content link because base is not an anchor entry", async function () {
	await expect_reject( async () => {
	    await clients.alice.call( DNA_NAME, EVIL_ZOME, "invalid_content_link_base", {
		"base": clients.bobby.cellAgent().retype("EntryHash"),
		"target": new ActionHash( crypto.randomBytes(32) ),
	    });
	}, "base address must be a group auth anchor entry" );
    });

    it("should reject auth anchor link because base is not a group entry", async function () {
	await expect_reject( async () => {
	    await clients.alice.call( DNA_NAME, EVIL_ZOME, "invalid_group_auth_link_base", {
		"base": c1_addr,
		"target": new ActionHash( crypto.randomBytes(32) ),
	    });
	}, "base address must be a group entry" );
    });

    it("should reject content link because author does not match auth anchor agent", async function () {
	await expect_reject( async () => {
	    await clients.bobby.call( DNA_NAME, EVIL_ZOME, "invalid_auth_anchor_link", {
		"group_id": g1_addr,
		"anchor_agent": clients.alice.cellAgent(),
		"target": new ActionHash( crypto.randomBytes(32) ),
	    });
	}, "link based on an auth anchor can only be made by the matching agent" );
    });

    it("should reject content update because the author group cannot be changed", async function () {
	await expect_reject( async () => {
	    await clients.alice.call( DNA_NAME, GOOD_ZOME, "update_content", {
		"base": c1_addr,
		"entry": Object.assign({}, c1, {
		    "group_ref": {
			"id": new ActionHash( crypto.randomBytes(32) ),
			"rev": new ActionHash( crypto.randomBytes(32) ),
		    },
		}),
	    });
	}, "group ID cannot be changed" );
    });

    it("should reject content create because author group ID/revision are not related", async function () {
	const g2_id			= await clients.bobby.call( DNA_NAME, COOP_ZOME, "create_group", createGroupInput(
	    [ clients.bobby.cellAgent() ],
	    clients.carol.cellAgent(),
	));
	await expect_reject( async () => {
	    await clients.bobby.call( DNA_NAME, GOOD_ZOME, "update_content", {
		"base": c1_addr,
		"entry": Object.assign({}, c1, {
		    "group_ref": {
			"id": g1_addr,
			"rev": g2_id,
		    },
		}),
	    });
	}, "group ID is not the initial action for the group revision" );
    });

    it("should reject auth anchor link because agent (A3) is not a group authority", async function () {
	await expect_reject( async () => {
	    await clients.alice.call( DNA_NAME, EVIL_ZOME, "invalid_group_auth_link", {
		"group_id": g1_addr,
		"group_rev": g1_addr,
		"anchor_agent": clients.carol.cellAgent(),
	    });
	}, "group auth anchors must match an authority in the group revision" );
    });

    // Dynamic
    it("should reject group update because agent (A2) is not an admin", async function () {
	await expect_reject( async () => {
	    await clients.bobby.call( DNA_NAME, COOP_ZOME, "update_group", {
		"base": g1_addr,
		"entry": group,
	    });
	}, "group can only be done by an admin" );
    });

    it("should reject content update because agent is not a group authority", async function () {
	await expect_reject( async () => {
	    await clients.carol.call( DNA_NAME, GOOD_ZOME, "update_content", {
		"base": c1_addr,
		"entry": Object.assign({}, c1, {
		    "text":		"(updated) " + faker.lorem.sentence(),
		}),
	    });
	}, "not authorized to update content managed by group" );
    });

    it("should reject auth anchor link because agent (A2) is not an admin", async function () {
	await expect_reject( async () => {
	    await clients.bobby.call( DNA_NAME, EVIL_ZOME, "invalid_group_auth_link", {
		"group_id": g1_addr,
		"group_rev": g1_addr,
		"anchor_agent": clients.alice.cellAgent(),
	    });
	}, "author of a group auth link must be an admin of the base group" );
    });

}


function phase2_tests () {

    it("should update group", async function () {
	group.members			= [
	    clients.carol.cellAgent(),
	];

	const addr = g1a_addr		= await clients.alice.call( DNA_NAME, COOP_ZOME, "update_group", {
	    "base": g1_addr,
	    "entry": group,
	});
	log.debug("New Group address: %s", addr );

	expect( addr			).to.be.a("Uint8Array");
	expect( addr			).to.have.length( 39 );

	group				= intoStruct( await clients.alice.call( DNA_NAME, COOP_ZOME, "get_group", g1_addr ), GroupStruct );
	log.debug( json.debug( group ) );
    });

    it("should A2 update content (C2 -> C2aa)", async function () {
	c2aa_addr			= await clients.bobby.call( DNA_NAME, GOOD_ZOME, "update_content", {
	    "base": c2_addr,
	    "entry": Object.assign( c2, {
		"text":	"(updated) " + faker.lorem.sentence(),
	    }),
	});
	log.debug("C2aa Address: %s", new ActionHash(c2aa_addr) );

	expect( c2aa_addr		).to.be.a("Uint8Array");
	expect( c2aa_addr		).to.have.length( 39 );
    });

    it("should A2 update content (C3 -> C3a)", async function () {
	c3a_addr			= await clients.bobby.call( DNA_NAME, GOOD_ZOME, "update_content", {
	    "base": c3_addr,
	    "entry": Object.assign( c3, {
		"text":	"(updated) " + faker.lorem.sentence(),
	    }),
	});
	log.debug("C3a Address: %s", new ActionHash(c3a_addr) );

	expect( c3a_addr		).to.be.a("Uint8Array");
	expect( c3a_addr		).to.have.length( 39 );
    });

    it("should A3 update content (C2a -> C2b)", async function () {
	c2b_addr			= await clients.carol.call( DNA_NAME, GOOD_ZOME, "update_content", {
	    "base": c2a_addr,
	    "entry": Object.assign( c2, {
		"text":	"(updated) " + faker.lorem.sentence(),
		"group_ref": {
		    "id": g1_addr,
		    "rev": g1a_addr,
		},
	    }),
	});
	log.debug("C2b Address: %s", new ActionHash(c2b_addr) );

	expect( c2b_addr		).to.be.a("Uint8Array");
	expect( c2b_addr		).to.have.length( 39 );
    });

    it("should A3 create content (C4)", async function () {
	{
	    const content_input		= createContentInput( clients.carol.cellAgent(), g1_addr, g1a_addr );
	    c4_addr			= await clients.carol.call( DNA_NAME, GOOD_ZOME, "create_content", content_input );
	    log.debug("C4 Address: %s", new ActionHash(c4_addr) );

	    expect( c4_addr		).to.be.a("Uint8Array");
	    expect( c4_addr		).to.have.length( 39 );

	    c4				= intoStruct( await clients.alice.call( DNA_NAME, GOOD_ZOME, "get_content", c4_addr ), ContentStruct );
	    log.debug( json.debug( c4 ) );
	}
    });

    it("should get group content and find: C1a, C2b, C3, C4", async function () {
	const targets			= new Set(
	    (await clients.carol.call( DNA_NAME, COOP_ZOME, "get_group_content_targets", g1_addr ))
		.map( addr => String(new HoloHash(addr)) )
	);
	log.debug("Group content targets: %s", targets );

	const expected_targets	= [
	    c1a_addr,
	    c2b_addr,
	    c3_addr,
	    c4_addr,
	].map( addr => String(new HoloHash(addr)) );
	expect( targets			).to.have.all.keys( ...expected_targets );
	expect( targets			).to.have.lengthOf( expected_targets.length );
    });

    linearSuite( "Phase 2 - Checks", phase2_checks_tests );

}

function phase2_checks_tests () {

    // Dynamic
    it("should reject archive content link because agent is not an admin", async function () {
	await expect_reject( async () => {
	    await clients.bobby.call( DNA_NAME, EVIL_ZOME, "invalid_archive_link", {
		"group_rev": g1a_addr,
		"archived_agent": clients.bobby.cellAgent(),
		"target": c2aa_addr,
	    });
	}, "auth archive anchor can only be made by group admins" );
    });

    it("should reject auth anchor link because agent (A2 + A3) is not an admin", async function () {
	await expect_reject( async () => {
	    await clients.bobby.call( DNA_NAME, EVIL_ZOME, "invalid_group_auth_link", {
		"group_id": g1_addr,
		"group_rev": g1a_addr,
		"anchor_agent": clients.alice.cellAgent(),
	    });
	}, "author of a group auth link must be an admin of the base group" );

	await expect_reject( async () => {
	    await clients.carol.call( DNA_NAME, EVIL_ZOME, "invalid_group_auth_link", {
		"group_id": g1_addr,
		"group_rev": g1a_addr,
		"anchor_agent": clients.alice.cellAgent(),
	    });
	}, "author of a group auth link must be an admin of the base group" );
    });

    it("should reject content update because agent (A3) did not update the author group revision", async function () {
	await expect_reject( async () => {
	    await clients.carol.call( DNA_NAME, GOOD_ZOME, "update_content", {
		"base": c2a_addr,
		"entry": Object.assign( c2, {
		    "text":	"(updated) " + faker.lorem.sentence(),
		    "group_ref": {
			"id": g1_addr,
			"rev": g1_addr,
		    },
		}),
	    });
	}, "not authorized to update content managed by group" );
    });

    it("should reject content update because agent (A2) is not an authority in the author group revision", async function () {
	await expect_reject( async () => {
	    await clients.bobby.call( DNA_NAME, GOOD_ZOME, "update_content", {
		"base": c2_addr,
		"entry": Object.assign( c2, {
		    "text":	"(updated) " + faker.lorem.sentence(),
		    "group_ref": {
			"id": g1_addr,
			"rev": g1a_addr,
		    },
		}),
	    });
	}, "not authorized to update content managed by group" );
    });

}


function phase3_tests () {

    it("should update group", async function () {
	group.members			= [
	    clients.bobby.cellAgent(), clients.carol.cellAgent(),
	];

	const addr = g1b_addr		= await clients.alice.call( DNA_NAME, COOP_ZOME, "update_group", {
	    "base": g1a_addr,
	    "entry": group,
	});
	log.debug("New Group address: %s", addr );

	expect( addr			).to.be.a("Uint8Array");
	expect( addr			).to.have.length( 39 );

	group				= intoStruct( await clients.alice.call( DNA_NAME, COOP_ZOME, "get_group", g1_addr ), GroupStruct );
	log.debug( json.debug( group ) );
    });

    it("should A2 update content (C4 -> C4a)", async function () {
	c4a_addr			= await clients.bobby.call( DNA_NAME, GOOD_ZOME, "update_content", {
	    "base": c4_addr,
	    "entry": Object.assign( c4, {
		"text":	"(updated) " + faker.lorem.sentence(),
		"group_ref": {
		    "id": g1_addr,
		    "rev": g1b_addr,
		},
	    }),
	});
	log.debug("C4a Address: %s", new ActionHash(c4a_addr) );

	expect( c4a_addr		).to.be.a("Uint8Array");
	expect( c4a_addr		).to.have.length( 39 );
    });

    it("should create content (C5) via bobby (A2)", async function () {
	{
	    const content_input		= createContentInput( clients.bobby.cellAgent(), g1_addr, g1b_addr );
	    c5_addr			= await clients.bobby.call( DNA_NAME, GOOD_ZOME, "create_content", content_input );
	    log.debug("C5 Address: %s", new ActionHash(c5_addr) );

	    expect( c5_addr		).to.be.a("Uint8Array");
	    expect( c5_addr		).to.have.length( 39 );

	    c5				= intoStruct( await clients.alice.call( DNA_NAME, GOOD_ZOME, "get_content", c5_addr ), ContentStruct );
	    log.debug( json.debug( c5 ) );
	}
    });

    it("should get group content and find: C1a, C2b, C3a, C4a, C5", async function () {
	const targets			= new Set(
	    (await clients.carol.call( DNA_NAME, COOP_ZOME, "get_group_content_targets", g1_addr ))
		.map( addr => String(new HoloHash(addr)) )
	);
	log.debug("Group content targets: %s", targets );

	const expected_targets	= [
	    c1a_addr,
	    c2b_addr,
	    c3a_addr,
	    c4a_addr,
	    c5_addr,
	].map( addr => String(new HoloHash(addr)) );
	expect( targets			).to.have.all.keys( ...expected_targets );
	expect( targets			).to.have.lengthOf( expected_targets.length );
    });

    linearSuite( "Phase 3 - Checks", phase3_checks_tests );
}

function phase3_checks_tests () {

    // Dynamic
    it("should reject auth anchor link because agent (A2 + A3) is not an admin", async function () {
	await expect_reject( async () => {
	    await clients.bobby.call( DNA_NAME, EVIL_ZOME, "invalid_group_auth_archive_link", {
		"group_rev": c1_addr,
		"anchor_agent": clients.alice.cellAgent(),
	    });
	}, "link base address must be a group entry" );
    });

}


function general_tests () {

    it("should trace origin", async function () {
	const result			= await clients.alice.call( DNA_NAME, DEBUG_ZOME, "trace_origin", g1a_addr );
	const history			= result.map( ([addr, action]) => [ new ActionHash(addr), intoStruct( action, EntryCreationActionStruct ) ]);
	log.debug("Group history: %s", json.debug( history ) );

	expect( history			).to.have.length( 2 );
    });

    it("should trace evolutions using group authorities", async function () {
	const result			= await clients.alice.call( DNA_NAME, DEBUG_ZOME, "trace_evolutions_using_authorities", {
	    "content_id": c2_addr,
	    "authorities": [ ...group.admins, ...group.members ],
	});
	const evolutions		= result.map( addr => new ActionHash(addr) );
	log.debug("Content evolutions: %s", json.debug( evolutions ) );

	expect( evolutions		).to.have.length( 3 );
    });

    it("should trace evolutions using group authorities with exceptions", async function () {
	const result			= await clients.alice.call( DNA_NAME, DEBUG_ZOME, "trace_evolutions_using_authorities_with_exceptions", {
	    "content_id": c2_addr,
	    "authorities": [ ...group.admins, ...group.members ],
	    "exceptions": [ c2aa_addr, c3a_addr ],
	});
	const evolutions		= result.map( addr => new ActionHash(addr) );
	log.debug("Content evolutions: %s", json.debug( evolutions ) );

	expect( evolutions		).to.have.length( 3 );
    });

    //
    // Failure checks
    //
    it("should fail because record does not exist", async function () {
	await expect_reject( async () => {
	    await clients.alice.call( DNA_NAME, COOP_ZOME, "get_group", new ActionHash(crypto.randomBytes(32)) );
	}, "Record not found" );
    });

}


describe("Model DNA", function () {
    const holochain			= new Holochain({
	"timeout": 60_000,
	"default_stdout_loggers": process.env.LOG_LEVEL === "trace",
    });

    before(async function () {
	this.timeout( 300_000 );

	const actors			= await holochain.backdrop({
	    "test_happ": {
		[DNA_NAME]:		TEST_DNA_PATH,
	    },
	}, {
	    "actors": [
		"alice", // admin
		"bobby", // constant member
		"carol", // member removed later
		"david", // member added later
		"emily", // admin
		"felix", // admin removed later
	    ],
	});

	for ( let name in actors ) {
	    for ( let app_prefix in actors[ name ] ) {
		log.info("Upgrade client for %s => %s", name, app_prefix );
		const client		= clients[ name ]	= actors[ name ][ app_prefix ].client;
	    }
	}

	// Must call whoami on each cell to ensure that init has finished.
	{
	    let whoami			= await clients.alice.call( DNA_NAME, GOOD_ZOME, "whoami", null, 300_000 );
	    log.normal("Alice whoami: %s", String(new HoloHash( whoami.agent_initial_pubkey )) );
	}
    });

    describe("Group", function () {
	linearSuite( "Phase 1", phase1_tests );
	linearSuite( "Phase 2", phase2_tests );
	linearSuite( "Phase 3", phase3_tests );
    });
    describe("General",			general_tests.bind( this ) );

    after(async () => {
	await holochain.destroy();
    });

});
