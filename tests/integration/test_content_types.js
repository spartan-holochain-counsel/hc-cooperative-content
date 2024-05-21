import { Logger }			from '@whi/weblogger';
const log				= new Logger("test-model-dna", process.env.LOG_LEVEL );

// const why				= require('why-is-node-running');

import fs				from 'node:fs';
import path				from 'path';
import crypto				from 'crypto';
import { expect }			from 'chai';
import { faker }			from '@faker-js/faker';
import msgpack				from '@msgpack/msgpack';
import json				from '@whi/json';

import { Holochain }			from '@spartan-hc/holochain-backdrop';
import {
    AppInterfaceClient,
}					from '@spartan-hc/app-interface-client';
import {
    CellZomelets,
}					from '@spartan-hc/zomelets';
import {
    expect_reject,
    linearSuite,
    createGroupInput,
    createContentInput,
    createCommentInput,
}					from '../utils.js';
import {
    EntryCreationActionStruct,
    Group,
    Content,
    Comment,
    BasicUsageCSRZomelet,
}					from './types.js';


const delay				= (n) => new Promise(f => setTimeout(f, n));
const __filename			= new URL(import.meta.url).pathname;
const __dirname				= path.dirname( __filename );
const TEST_DNA_PATH			= path.join( __dirname, "../model_dna.dna" );

const DNA_NAME				= "test_dna";
const GOOD_ZOME				= "basic_usage_csr";


export const BasicUsageCell		= new CellZomelets({
    [GOOD_ZOME]:	BasicUsageCSRZomelet,
});


let app_port;
let client;
let alice_client;
let bobby_client;
let carol_client;
let alice_basic_csr;
let bobby_basic_csr;
let carol_basic_csr;
let group, g1_addr;
let c1, c1_addr;
let c2, c2_addr;
let c3, c3_addr;


describe("Content Types", function () {
    const holochain			= new Holochain({
	"timeout": 60_000,
	"default_stdout_loggers": log.level_rank > 3,
    });

    before(async function () {
	this.timeout( 300_000 );

	const installations		= await holochain.install([
	    "alice", // admin
	    "bobby", // constant member
	    "carol", // member removed later
	], [
	    {
		"app_name": "test",
		"bundle": {
		    [DNA_NAME]:		TEST_DNA_PATH,
		},
	    },
	]);

	app_port			= await holochain.ensureAppPort();

	client				= new AppInterfaceClient( app_port, {
	    "logging": process.env.LOG_LEVEL || "fatal",
	});

	const alice_token		= installations.alice.test.auth.token;
	alice_client			= await client.app( alice_token );

	const bobby_token		= installations.bobby.test.auth.token;
	bobby_client			= await client.app( bobby_token );

	const carol_token		= installations.carol.test.auth.token;
	carol_client			= await client.app( carol_token );

	{
	    const interfaces		= alice_client.createInterface({
		[DNA_NAME]:	BasicUsageCell,
	    });
	    alice_basic_csr		= interfaces[ DNA_NAME ].zomes[ GOOD_ZOME ].functions;
	}

	{
	    const interfaces		= bobby_client.createInterface({
		[DNA_NAME]:	BasicUsageCell,
	    });
	    bobby_basic_csr		= interfaces[ DNA_NAME ].zomes[ GOOD_ZOME ].functions;
	}

	{
	    const interfaces		= carol_client.createInterface({
		[DNA_NAME]:	BasicUsageCell,
	    });
	    carol_basic_csr		= interfaces[ DNA_NAME ].zomes[ GOOD_ZOME ].functions;
	}

	// Must call whoami on each cell to ensure that init has finished.
	{
	    let whoami			= await alice_basic_csr.whoami();
	    log.normal("Alice whoami: %s", whoami.pubkey.initial );
	}
    });

    describe("Group", function () {
	linearSuite( "Phase 1", phase1_tests );
    });

    after(async () => {
	await holochain.destroy();
    });

});


function phase1_tests () {

    it("should create group via alice (A1)", async function () {
	const group_input		= createGroupInput(
	    [ alice_client.agent_id ],
	    bobby_client.agent_id
	);
	g1_addr				= await alice_client.call( DNA_NAME, GOOD_ZOME, "create_group", group_input );
	log.debug("Group ID: %s", g1_addr );

	// expect( g1_addr		).to.be.a("ActionHash");
	expect( g1_addr		).to.be.a("Uint8Array");

	group				= await alice_basic_csr.get_group( g1_addr );
	log.debug( json.debug( group ) );
    });

    it("(A1) should create each content type", async function () {
	{
	    const content_input		= createContentInput( g1_addr, g1_addr );
	    c1_addr			= await alice_basic_csr.create_content( content_input );
	    log.debug("C1 Address: %s", c1_addr );
	}

	{
	    const comment_input		= createCommentInput( g1_addr, g1_addr );
	    c2_addr			= await alice_basic_csr.create_comment( comment_input );
	    log.debug("C2 Address: %s", c2_addr );
	}

	{
	    const comment_input		= createCommentInput( g1_addr, g1_addr, {
		"parent_comment":	c2_addr,
	    });
	    c3_addr			= await alice_basic_csr.create_comment( comment_input );
	    log.debug("C3 Address: %s", c3_addr );
	}
    });

    it("should get all group content", async function () {
	const targets			= await carol_basic_csr.get_group_content({
	    "group_id": g1_addr,
	});
	log.normal("Group content targets: %s", json.debug(targets) );

	expect( targets			).to.have.lengthOf( 3 );
    });

    it("should get group comments", async function () {
	const targets			= await carol_basic_csr.get_group_content({
	    "group_id": g1_addr,
	    "content_type": "comment",
	});
	log.normal("Group content targets: %s", json.debug(targets) );

	expect( targets			).to.have.lengthOf( 2 );
    });

    it("should get group comments for parent comment", async function () {
	const targets			= await carol_basic_csr.get_group_content({
	    "group_id": g1_addr,
	    "content_type": "comment",
	    "content_base": String(c2_addr),
	});
	log.normal("Group content targets: %s", json.debug(targets) );

	expect( targets			).to.have.lengthOf( 1 );
    });

}
