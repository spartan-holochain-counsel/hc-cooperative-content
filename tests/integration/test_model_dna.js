import { Logger }			from '@whi/weblogger';
const log				= new Logger("test-model-dna", process.env.LOG_LEVEL );

import fs				from 'node:fs';
import path				from 'path';
import crypto				from 'crypto';
import { expect }			from 'chai';
import { faker }			from '@faker-js/faker';
import msgpack				from '@msgpack/msgpack';
import json				from '@whi/json';
import { AgentPubKey, HoloHash,
	 ActionHash, EntryHash }	from '@spartan-hc/holo-hash';
import { Holochain }                    from '@spartan-hc/holochain-backdrop';

import {
    CoopContentZomelet,
}					from '@spartan-hc/coop-content-zomelets';
import {
    AppInterfaceClient,
}					from '@spartan-hc/app-interface-client';
import {
    intoStruct,
    OptionType, VecType, MapType,
}					from '@whi/into-struct';

// const why				= require('why-is-node-running');
import {
    expect_reject,
    linearSuite,
    createGroupInput,
    createContentInput,
    delay,
}					from '../utils.js';
import {
    EntryCreationActionStruct,
    ContentEntry,
    BasicUsageZomelet,
}					from '../types.js';


const __filename			= new URL(import.meta.url).pathname;
const __dirname				= path.dirname( __filename );
const TEST_DNA_PATH			= path.join( __dirname, "../model_dna.dna" );

const DNA_NAME				= "test_dna";

const DEBUG_ZOME			= "debug_csr";
const GEN_ZOME				= "general_csr";
const COOP_ZOME				= "coop_content_csr";
const GOOD_ZOME				= "basic_usage_csr";
const EVIL_ZOME				= "corrupt_csr";

let client, installations;

describe("Model DNA", function () {
    const holochain			= new Holochain({
	"timeout": 60_000,
	"default_stdout_loggers": log.level_rank > 3,
    });

    before(async function () {
	this.timeout( 300_000 );

	installations			= await holochain.install([
	    "alice", // admin
	    "bobby", // constant member
	    "carol", // member removed later
	    "david", // member added later
	    "emily", // admin
	    "felix", // admin removed later
	], [
	    {
		"app_name": "test",
		"bundle": {
		    [DNA_NAME]:		TEST_DNA_PATH,
		},
	    },
	]);

	const app_port			= await holochain.ensureAppPort();

	client				= new AppInterfaceClient( app_port, {
	    "logging": process.env.LOG_LEVEL || "fatal",
	});

	const alice_token		= installations.alice.test.auth.token;
	alice_client			= await client.app( alice_token );

	const bobby_token		= installations.bobby.test.auth.token;
	bobby_client			= await client.app( bobby_token );

	const carol_token		= installations.carol.test.auth.token;
	carol_client			= await client.app( carol_token );

	const david_token		= installations.david.test.auth.token;
	david_client			= await client.app( david_token );

	const emily_token		= installations.emily.test.auth.token;
	emily_client			= await client.app( emily_token );

	const felix_token		= installations.felix.test.auth.token;
	felix_client			= await client.app( felix_token );
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


let alice_client;
let bobby_client;
let carol_client;
let david_client;
let emily_client;
let felix_client;

let alice_coop_content;
let bobby_coop_content;
let carol_coop_content;
let david_coop_content;

let alice_good_zome;
let bobby_good_zome;
let carol_good_zome;
let david_good_zome;

let group, g1_addr, g1a_addr, g1b_addr;
let c1, c1_addr, c1a_addr;
let c2, c2_addr, c2a_addr, c2aa_addr, c2b_addr;
let c3, c3a, c3_addr, c3a_addr;
let c4, c4_addr, c4a_addr;
let c5, c5_addr;

function phase1_tests () {

    before(async function () {
	this.timeout( 30_000 );

        {
	    const auth			= installations.alice.test.auth;
	    alice_client		= await client.app( auth.token, "test-alice" );

            alice_coop_content          = alice_client.createZomeInterface(
                DNA_NAME, COOP_ZOME, CoopContentZomelet
            ).functions;
            alice_good_zome             = alice_client.createZomeInterface(
                DNA_NAME, GOOD_ZOME, BasicUsageZomelet
            ).functions;
        }

        {
	    const auth			= installations.bobby.test.auth;
	    bobby_client		= await client.app( auth.token, "test-bobby" );

            bobby_coop_content          = bobby_client.createZomeInterface(
                DNA_NAME, COOP_ZOME, CoopContentZomelet
            ).functions;
            bobby_good_zome             = bobby_client.createZomeInterface(
                DNA_NAME, GOOD_ZOME, BasicUsageZomelet
            ).functions;
        }

        {
	    const auth			= installations.carol.test.auth;
	    carol_client		= await client.app( auth.token, "test-carol" );

            carol_coop_content          = carol_client.createZomeInterface(
                DNA_NAME, COOP_ZOME, CoopContentZomelet
            ).functions;
            carol_good_zome             = carol_client.createZomeInterface(
                DNA_NAME, GOOD_ZOME, BasicUsageZomelet
            ).functions;
        }

        {
	    const auth			= installations.david.test.auth;
	    david_client		= await client.app( auth.token, "test-david" );

            david_coop_content          = david_client.createZomeInterface(
                DNA_NAME, COOP_ZOME, CoopContentZomelet
            ).functions;
            david_good_zome             = david_client.createZomeInterface(
                DNA_NAME, GOOD_ZOME, BasicUsageZomelet
            ).functions;
        }

        {
	    const auth			= installations.emily.test.auth;
	    emily_client		= await client.app( auth.token, "test-emily" );
        }

        {
	    const auth			= installations.felix.test.auth;
	    felix_client		= await client.app( auth.token, "test-felix" );
        }

	{
	    let whoami			= await alice_coop_content.whoami();
	    log.normal("Alice whoami: %s", whoami.pubkey.initial );
	}
	{
	    let whoami			= await bobby_coop_content.whoami();
	    log.normal("Bobby whoami: %s", whoami.pubkey.initial );
	}
	{
	    let whoami			= await carol_coop_content.whoami();
	    log.normal("Carol whoami: %s", whoami.pubkey.initial );
	}
	{
	    let whoami			= await david_coop_content.whoami();
	    log.normal("David whoami: %s", whoami.pubkey.initial );
	}
    });


    it("should create group via alice (A1)", async function () {
	const group_input		= createGroupInput(
	    [
		alice_client.agent_id,
		emily_client.agent_id,
		felix_client.agent_id,
	    ],
	    bobby_client.agent_id, carol_client.agent_id,
	);
	group				= await alice_coop_content.create_group( group_input );
	log.debug( json.debug( group ) );
    });

    it("should create content (C1 + C2) via alice (A1)", async function () {
	{
	    const content_input		= createContentInput( group.$id, group.$id );
	    c1_addr			= await alice_good_zome.create_content( content_input );
	    log.debug("C1 Address: %s", c1_addr );
	    c1				= await alice_good_zome.get_content({
		"group_id": group.$id,
		"content_id": c1_addr,
	    });
	    log.debug( json.debug( c1 ) );
	}
	{
	    const content_input		= createContentInput( group.$id, group.$id );
	    c2_addr			= await alice_good_zome.create_content( content_input );
	    log.debug("C2 Address: %s", c2_addr );
	    c2				= await alice_good_zome.get_content({
		"group_id": group.$id,
		"content_id": c2_addr,
	    });
	    log.debug( json.debug( c2 ) );
	}
    });

    it("should create content (C3) via carol (A3)", async function () {
	{
	    const content_input		= createContentInput( group.$id, group.$id );
	    c3_addr			= await carol_good_zome.create_content( content_input );
	    log.debug("C3 Address: %s", c3_addr );

            await delay();

	    c3				= await alice_good_zome.get_content({
		"group_id": group.$id,
		"content_id": c3_addr,
	    });
	    log.debug( json.debug( c3 ) );
	}
    });

    it("should update content (C1 => C1a) via carol (A3)", async function () {
	{
	    c1a_addr			= await carol_good_zome.update_content({
		"base": c1_addr,
		"entry": Object.assign( c1, {
		    "text":		"(updated) " + faker.lorem.sentence(),
		}),
	    });
	    log.debug("C1a Address: %s", c1a_addr );
	}
    });

    it("should update content (C2 => C2a) via alice (A1)", async function () {
	{
	    c2a_addr			= await alice_good_zome.update_content({
		"base": c2_addr,
		"entry": Object.assign( c2, {
		    "text":		"(updated) " + faker.lorem.sentence(),
		}),
	    });
	    log.debug("C2a Address: %s", c2a_addr );
	}
    });

    it("should get group content and find: C1a, C2a, C3", async function () {
	const targets			= new Set(
	    (await david_good_zome.get_group_content({
		"group_id": group.$id,
	    }))
		.map( pair => String( pair[0][1] ) )
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
            await alice_coop_content.update_group({
		"base": group.$id,
		"entry": Object.assign({}, group, {
		    "admins": [ alice_client.agent_id ],
		}),
	    });
	}, "requires counter-signing" ); // group admins cannot be changed without counter-signing
    });

    it("should reject content link because base is not an anchor entry", async function () {
	await expect_reject( async () => {
	    await alice_client.call( DNA_NAME, EVIL_ZOME, "invalid_content_link_base", {
		"base": carol_client.agent_id.retype("EntryHash"),
		"target": new ActionHash( crypto.randomBytes(32) ),
	    });
	}, "has no serialized bytes" );
    });

    it("should reject auth anchor link because base is not a group entry", async function () {
	await expect_reject( async () => {
	    await alice_client.call( DNA_NAME, EVIL_ZOME, "invalid_group_auth_link_base", {
		"base": c1_addr,
		"target": new ActionHash( crypto.randomBytes(32) ),
	    });
	}, "Could not deserialize any-linkable address to expected type: missing field `admins`" );
    });

    it("should reject content link because author does not match auth anchor agent", async function () {
	await expect_reject( async () => {
	    await carol_client.call( DNA_NAME, EVIL_ZOME, "invalid_auth_anchor_link", {
		"group_id": group.$id,
		"anchor_agent": alice_client.agent_id,
		"target": new ActionHash( crypto.randomBytes(32) ),
	    });
	}, "link based on an auth anchor can only be made by the matching agent" );
    });

    it("should reject content update because the author group cannot be changed", async function () {
	await expect_reject( async () => {
	    await alice_good_zome.update_content({
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
	const group2			= await bobby_coop_content.create_group( createGroupInput(
	    [ bobby_client.agent_id ],
	    carol_client.agent_id,
	));
	await expect_reject( async () => {
	    await bobby_good_zome.update_content({
		"base": c1_addr,
		"entry": Object.assign({}, c1, {
		    "group_ref": {
			"id": group.$id,
			"rev": group2.$action,
		    },
		}),
	    });
	}, "group ID is not the initial action for the group revision" );
    });

    it("should reject auth anchor link because agent (A4) is not in the group's contributors", async function () {
	await expect_reject( async () => {
	    await alice_client.call( DNA_NAME, EVIL_ZOME, "invalid_group_auth_link", {
		"group_id": group.$id,
		"group_rev": group.$id,
		"anchor_agent": david_client.agent_id,
	    });
	}, "contributions anchor must match a contributor in the group base" );
    });

    // Dynamic
    it("should reject group update because agent (A3) is not an admin", async function () {
	await expect_reject( async () => {
            await carol_coop_content.update_group({
		"base": group.$id,
		"entry": group,
	    });
	}, "group can only be done by an admin" );
    });

    it("should reject content update because agent is not in the group's contributors", async function () {
	await expect_reject( async () => {
	    await david_good_zome.update_content({
		"base": c1_addr,
		"entry": Object.assign({}, c1, {
		    "text":		"(updated) " + faker.lorem.sentence(),
		}),
	    });
	}, "not authorized to update content managed by group" );
    });

    it("should reject auth anchor link because agent (A3) is not an admin", async function () {
	await expect_reject( async () => {
	    await carol_client.call( DNA_NAME, EVIL_ZOME, "invalid_group_auth_link", {
		"group_id": group.$id,
		"group_rev": group.$id,
		"anchor_agent": alice_client.agent_id,
	    });
	}, "author of a group auth link must be an admin of the base group" );
    });

}


function phase2_tests () {

    it("should update group", async function () {
	group.members			= [
	    bobby_client.agent_id, david_client.agent_id,
	];

        group                           = await alice_coop_content.update_group({
	    "base": group.$id,
	    "entry": group,
	});
        g1a_addr                        = group.$action;
	log.debug("New Group address: %s", g1a_addr );
	log.debug( json.debug( group ) );
    });

    it("should A3 update content (C2 -> C2aa)", async function () {
	c2aa_addr			= await carol_good_zome.update_content({
	    "base": c2_addr,
	    "entry": Object.assign( c2, {
		"text":	"(updated) " + faker.lorem.sentence(),
	    }),
	});
	log.debug("C2aa Address: %s", c2aa_addr );
    });

    it("should A3 update content (C3 -> C3a)", async function () {
	c3a				= {};
	c3a_addr			= await carol_good_zome.update_content({
	    "base": c3_addr,
	    "entry": Object.assign( c3a, c3, {
		"text":	"(updated) " + faker.lorem.sentence(),
	    }),
	});
	log.debug("C3a Address: %s", c3a_addr );
    });

    it("should A4 update content (C2a -> C2b)", async function () {
	c2b_addr			= await david_good_zome.update_content({
	    "base": c2a_addr,
	    "entry": Object.assign( c2, {
		"text":	"(updated) " + faker.lorem.sentence(),
		"group_ref": {
		    "id": group.$id,
		    "rev": g1a_addr,
		},
	    }),
	});
	log.debug("C2b Address: %s", c2b_addr );

	let entry			= await carol_client.call( DNA_NAME, GEN_ZOME, "fetch_entry", c2b_addr );
	let decoded			= msgpack.decode( entry.entry );

	c2				= ContentEntry( decoded );
    });

    it("should A4 create content (C4)", async function () {
	{
	    const content_input		= createContentInput( group.$id, g1a_addr );
	    c4_addr			= await david_good_zome.create_content( content_input );
	    log.debug("C4 Address: %s", c4_addr );

            await delay();

	    c4				= await alice_good_zome.get_content({
		"group_id": group.$id,
		"content_id": c4_addr,
	    });
	    log.debug( json.debug( c4 ) );
	}
    });

    it("should get group content and find: C1a, C2b, C3, C4", async function () {
	const targets			= new Set(
	    (await david_good_zome.get_group_content({
		"group_id": group.$id,
	    }))
		.map( pair => String( pair[0][1] ) )
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

    it("should get content (C3) latest revision (C3)", async function () {
	{
	    let content			= await carol_good_zome.get_content({
		"group_id": group.$id,
		"content_id": c3_addr,
	    });

	    expect( c3			).to.deep.equal( content );
	}
    });

    linearSuite( "Phase 2 - Checks", phase2_checks_tests );

}

function phase2_checks_tests () {

    // Dynamic
    it("should reject archive content link because agent is not an admin", async function () {
	await expect_reject( async () => {
	    await carol_client.call( DNA_NAME, EVIL_ZOME, "invalid_archive_link", {
		"group_rev": g1a_addr,
		"archived_agent": carol_client.agent_id,
		"target": c2aa_addr,
	    });
	}, "auth archive anchor can only be made by group admins" );
    });

    it("should reject auth anchor link because agent (A3 + A4) is not an admin", async function () {
	await expect_reject( async () => {
	    await carol_client.call( DNA_NAME, EVIL_ZOME, "invalid_group_auth_link", {
		"group_id": group.$id,
		"group_rev": g1a_addr,
		"anchor_agent": alice_client.agent_id,
	    });
	}, "author of a group auth link must be an admin of the base group" );

	await expect_reject( async () => {
	    await david_client.call( DNA_NAME, EVIL_ZOME, "invalid_group_auth_link", {
		"group_id": group.$id,
		"group_rev": g1a_addr,
		"anchor_agent": alice_client.agent_id,
	    });
	}, "author of a group auth link must be an admin of the base group" );
    });

    it("should reject content update because agent (A4) did not update the author group revision", async function () {
	await expect_reject( async () => {
	    await david_good_zome.update_content({
		"base": c2a_addr,
		"entry": Object.assign( {}, c2, {
		    "text":	"(updated) " + faker.lorem.sentence(),
		    "group_ref": {
			"id": group.$id,
			"rev": group.$id,
		    },
		}),
	    });
	}, "not authorized to update content managed by group" );
    });

    it("should reject content update because agent (A3) is not a contributor in the author group revision", async function () {
	await expect_reject( async () => {
	    await carol_good_zome.update_content({
		"base": c2_addr,
		"entry": Object.assign( {}, c2, {
		    "text":	"(updated) " + faker.lorem.sentence(),
		    "group_ref": {
			"id": group.$id,
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
	    bobby_client.agent_id, carol_client.agent_id, david_client.agent_id,
	];

        group                           = await alice_coop_content.update_group({
	    "base": g1a_addr,
	    "entry": group,
	});
        g1b_addr                        = group.$action;
	log.debug("New Group address: %s", g1b_addr );
	log.debug( json.debug( group ) );
    });

    it("should A3 update content (C4 -> C4a)", async function () {
	c4a_addr			= await carol_good_zome.update_content({
	    "base": c4_addr,
	    "entry": Object.assign( c4, {
		"text":	"(updated) " + faker.lorem.sentence(),
		"group_ref": {
		    "id": group.$id,
		    "rev": g1b_addr,
		},
	    }),
	});
	log.debug("C4a Address: %s", c4a_addr );
    });

    it("should create content (C5) via carol (A3)", async function () {
	{
	    const content_input		= createContentInput( group.$id, g1b_addr );
	    c5_addr			= await carol_good_zome.create_content( content_input );
	    log.debug("C5 Address: %s", c5_addr );

            await delay();

	    c5				= await alice_good_zome.get_content({
		"group_id": group.$id,
		"content_id": c5_addr,
	    });
	    log.debug( json.debug( c5 ) );
	}
    });

    it("should get group content and find: C1a, C2b, C3a, C4a, C5", async function () {
	const contents			= await david_good_zome.get_group_content({
	    "group_id": group.$id,
	});
	const targets			= new Set( contents.map( pair => String(pair[0][1]) ) );
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

    it("should get group content using full trace and find: C1a, C2b, C3a, C4a, C5", async function () {
	const targets			= new Set(
	    (await david_good_zome.get_group_content({
		"group_id": group.$id,
		"full_trace": true,
	    }))
		.map( pair => String( pair[0][1] ) )
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

    it("should get content (C3) latest revision (C3a)", async function () {
	{
	    let content			= await carol_good_zome.get_content({
		"group_id": group.$id,
		"content_id": c3_addr,
	    });

	    expect( c3a			).to.deep.equal( content );
	}
    });

    linearSuite( "Phase 3 - Checks", phase3_checks_tests );
}

function phase3_checks_tests () {

    // Dynamic
    it("should reject auth archive anchor link because base is not a group entry", async function () {
	await expect_reject( async () => {
	    await carol_client.call( DNA_NAME, EVIL_ZOME, "invalid_group_auth_archive_link", {
		"group_rev": c1_addr,
		"anchor_agent": alice_client.agent_id,
	    });
	}, "Could not deserialize any-linkable address to expected type: missing field `admins`" );
    });

    it("should reject auth anchor link delete", async function () {
	await expect_reject( async () => {
	    await alice_client.call( DNA_NAME, EVIL_ZOME, "delete_group_auth_link", {
		"group_id": group.$id,
		"group_rev": group.$id,
		"anchor_agent": alice_client.agent_id,
	    });
	}, "group auth links cannot be deleted" );
    });

    it("should reject content link delete because author did not create the link", async function () {
	await expect_reject( async () => {
	    await alice_client.call( DNA_NAME, COOP_ZOME, "delete_group_auth_anchor_content_links", [
		{
		    "group_id": group.$id,
		    "author": carol_client.agent_id,
		},
		c3_addr,
	    ]);
	}, "contributions anchor can only be deleted" );
    });

    it("should reject content link delete because author is not an admin", async function () {
	await expect_reject( async () => {
	    await carol_client.call( DNA_NAME, COOP_ZOME, "delete_group_auth_anchor_content_links", [
		{
		    "group_id": g1a_addr,
		    "author": carol_client.agent_id,
		    "anchor_type": "archive",
		},
		c3_addr,
	    ]);
	}, "can only be deleted by an admin" );
    });

}


function general_tests () {
	// let evolutions			= await carol_client.call( DNA_NAME, GEN_ZOME, "follow_evolutions", c3_addr );
	// const history			= await Promise.all(
	//     evolutions
	// 	.map( addr => new ActionHash(addr) )
	// 	.map( async addr => {
	// 	    let action		= await carol_client.call( DNA_NAME, GEN_ZOME, "get_action", addr );
	// 	    return [
	// 		new ActionHash(addr),
	// 		intoStruct( action, EntryCreationActionStruct ),
	// 	    ];
	// 	})
	// );
	// console.log( json.debug( history ) );


    it("should trace origin", async function () {
	const result			= await alice_client.call( DNA_NAME, DEBUG_ZOME, "trace_origin", g1a_addr );
	const history			= result.map( ([addr, action]) => [ new ActionHash(addr), intoStruct( action, EntryCreationActionStruct ) ]);
	log.debug("Group history: %s", json.debug( history ) );

	expect( history			).to.have.length( 2 );
    });

    it("should trace evolutions using group authorities", async function () {
	const result			= await alice_client.call( DNA_NAME, DEBUG_ZOME, "follow_evolutions_using_authorities", {
	    "content_id": c2_addr,
	    "authorities": [ ...group.admins, ...group.members ],
	});
	const evolutions		= result.map( addr => new ActionHash(addr) );
	log.debug("Content evolutions: %s", json.debug( evolutions ) );

	expect( evolutions		).to.have.length( 3 );
    });

    it("should trace evolutions using group authorities with exceptions", async function () {
	const result			= await alice_client.call( DNA_NAME, DEBUG_ZOME, "follow_evolutions_using_authorities_with_exceptions", {
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
            await alice_coop_content.get_group( new ActionHash(crypto.randomBytes(32)) );
	}, "Failed to get Record" );
    });

    it("should reject group delete", async function () {
	await expect_reject( async () => {
	    await alice_client.call( DNA_NAME, EVIL_ZOME, "delete_group", group.$id );
	}, "cannot be deleted" );
    });

}
