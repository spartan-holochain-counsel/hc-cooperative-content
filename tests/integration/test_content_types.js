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
    CoopContentZomelet,
}					from '@spartan-hc/coop-content-zomelets';
import {
    AppInterfaceClient,
}					from '@spartan-hc/app-interface-client';

import {
    expect_reject,
    linearSuite,
    createGroupInput,
    createContentInput,
    createCommentInput,
}					from '../utils.js';
import {
    BasicUsageZomelet,
}					from '../types.js';


const delay				= (n) => new Promise(f => setTimeout(f, n));
const __filename			= new URL(import.meta.url).pathname;
const __dirname				= path.dirname( __filename );
const TEST_DNA_PATH			= path.join( __dirname, "../model_dna.dna" );

const DNA_NAME				= "test_dna";
const COOP_ZOME				= "coop_content_csr";
const GOOD_ZOME				= "basic_usage_csr";

let client, installations;

describe("Content Types", function () {
    const holochain			= new Holochain({
	"timeout": 60_000,
	"default_stdout_loggers": log.level_rank > 3,
    });

    before(async function () {
	this.timeout( 300_000 );

	installations                   = await holochain.install([
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
    });

    describe("Group", function () {
	linearSuite( "Phase 1", phase1_tests );
    });

    after(async () => {
	await holochain.destroy();
    });

});


let alice_client;
let bobby_client;
let carol_client;

let alice_coop_content;
let bobby_coop_content;
let carol_coop_content;

let alice_basic_csr;
let bobby_basic_csr;
let carol_basic_csr;

let group;
let c1, c1_addr;
let c2, c2_addr;
let c3, c3_addr;

function phase1_tests () {
    before(async function () {
	this.timeout( 30_000 );

        {
	    const auth			= installations.alice.test.auth;
	    alice_client		= await client.app( auth.token, "test-alice" );

            alice_coop_content          = alice_client.createZomeInterface(
                DNA_NAME, COOP_ZOME, CoopContentZomelet
            ).functions;
            alice_basic_csr             = alice_client.createZomeInterface(
                DNA_NAME, GOOD_ZOME, BasicUsageZomelet
            ).functions;
        }

        {
	    const auth			= installations.bobby.test.auth;
	    bobby_client		= await client.app( auth.token, "test-bobby" );

            bobby_coop_content          = bobby_client.createZomeInterface(
                DNA_NAME, COOP_ZOME, CoopContentZomelet
            ).functions;
            bobby_basic_csr             = bobby_client.createZomeInterface(
                DNA_NAME, GOOD_ZOME, BasicUsageZomelet
            ).functions;
        }

        {
	    const auth			= installations.carol.test.auth;
	    carol_client		= await client.app( auth.token, "test-carol" );

            carol_coop_content          = carol_client.createZomeInterface(
                DNA_NAME, COOP_ZOME, CoopContentZomelet
            ).functions;
            carol_basic_csr             = carol_client.createZomeInterface(
                DNA_NAME, GOOD_ZOME, BasicUsageZomelet
            ).functions;
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
    });


    it("should create group via alice (A1)", async function () {
	const group_input		= createGroupInput(
	    [ alice_client.agent_id ],
	    bobby_client.agent_id
	);
	group				= await alice_coop_content.create_group( group_input );
	log.debug( json.debug( group ) );
    });

    it("(A1) should create each content type", async function () {
	{
	    const content_input		= createContentInput( group.$id, group.$id );
	    c1_addr			= await alice_basic_csr.create_content( content_input );
	    log.debug("C1 Address: %s", c1_addr );
	}

	{
	    const comment_input		= createCommentInput( group.$id, group.$id );
	    c2_addr			= await alice_basic_csr.create_comment( comment_input );
	    log.debug("C2 Address: %s", c2_addr );
	}

	{
	    const comment_input		= createCommentInput( group.$id, group.$id, {
		"parent_comment":	c2_addr,
	    });
	    c3_addr			= await alice_basic_csr.create_comment( comment_input );
	    log.debug("C3 Address: %s", c3_addr );
	}
    });

    it("should get all group content", async function () {
	const targets			= await carol_basic_csr.get_group_content({
	    "group_id": group.$id,
	});
	log.normal("Group content targets: %s", json.debug(targets) );

	expect( targets			).to.have.lengthOf( 3 );
    });

    it("should get group comments", async function () {
	const targets			= await carol_basic_csr.get_group_content({
	    "group_id": group.$id,
	    "content_type": "comment",
	});
	log.normal("Group content targets: %s", json.debug(targets) );

	expect( targets			).to.have.lengthOf( 2 );
    });

    it("should get group comments for parent comment", async function () {
	const targets			= await carol_basic_csr.get_group_content({
	    "group_id": group.$id,
	    "content_type": "comment",
	    "content_base": String(c2_addr),
	});
	log.normal("Group content targets: %s", json.debug(targets) );

	expect( targets			).to.have.lengthOf( 1 );
    });

}
