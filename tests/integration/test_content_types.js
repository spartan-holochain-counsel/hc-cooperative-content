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
import HolochainBackdrop		from '@spartan-hc/holochain-backdrop';
const { Holochain }			= HolochainBackdrop;
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
    createCommentInput,
}					from '../utils.js';
import {
    EntryCreationActionStruct,
    GroupStruct,
    ContentStruct,
    CommentStruct,
}					from './types.js';


const delay				= (n) => new Promise(f => setTimeout(f, n));
const __filename			= new URL(import.meta.url).pathname;
const __dirname				= path.dirname( __filename );
const TEST_DNA_PATH			= path.join( __dirname, "../model_dna.dna" );

const DNA_NAME				= "test_dna";

const COOP_ZOME				= "coop_content_csr";
const GOOD_ZOME				= "basic_usage_csr";


let app_port;
let client;
let alice_client;
let bobby_client;
let carol_client;
let group, g1_addr;
let c1, c1_addr;
let c2, c2_addr;


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

	group				= intoStruct( await alice_client.call( DNA_NAME, GOOD_ZOME, "get_group", g1_addr ), GroupStruct );
	log.debug( json.debug( group ) );
    });

    it("(A1) should create each content type", async function () {
	{
	    const content_input		= createContentInput( g1_addr, g1_addr );
	    c1_addr			= await alice_client.call( DNA_NAME, GOOD_ZOME, "create_content", content_input );
	    log.debug("C1 Address: %s", new ActionHash(c1_addr) );
	}

	{
	    const comment_input		= createCommentInput( g1_addr, g1_addr );
	    c2_addr			= await alice_client.call( DNA_NAME, GOOD_ZOME, "create_comment", comment_input );
	    log.debug("C2 Address: %s", new ActionHash(c2_addr) );
	}
    });

    it("should get all group content", async function () {
	const targets			= await carol_client.call( DNA_NAME, GOOD_ZOME, "get_group_content", {
	    "group_id": g1_addr,
	});
	log.normal("Group content targets: %s", json.debug(targets) );

	expect( targets			).to.have.lengthOf( 2 );
    });

    it("should get group comments", async function () {
	const targets			= await carol_client.call( DNA_NAME, GOOD_ZOME, "get_group_content", {
	    "group_id": g1_addr,
	    "content_type": "comment",
	});
	log.normal("Group content targets: %s", json.debug(targets) );

	expect( targets			).to.have.lengthOf( 1 );
    });

}



describe("Model DNA", function () {
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

	// Must call whoami on each cell to ensure that init has finished.
	{
	    let whoami			= await alice_client.call( DNA_NAME, GOOD_ZOME, "whoami", null, 300_000 );
	    log.normal("Alice whoami: %s", String(new HoloHash( whoami.agent_initial_pubkey )) );
	}
    });

    describe("Group", function () {
	linearSuite( "Phase 1", phase1_tests );
    });

    after(async () => {
	await holochain.destroy();
    });

});
