import { Logger }			from '@whi/weblogger';
const log				= new Logger("test-minimal-dna", process.env.LOG_LEVEL );

import fs				from 'node:fs';
import path				from 'path';
import crypto				from 'crypto';
import { expect }			from 'chai';
import { faker }			from '@faker-js/faker';
import msgpack				from '@msgpack/msgpack';
import json				from '@whi/json';
import { AgentPubKey, HoloHash,
	 ExternalHash,
	 ActionHash, EntryHash }	from '@spartan-hc/holo-hash';
import { Holochain }                    from '@spartan-hc/holochain-backdrop';

import {
    CoopContentZomelet,
}					from '@spartan-hc/coop-content-zomelets';
import {
    AppInterfaceClient,
}					from '@spartan-hc/app-interface-client';

// const why				= require('why-is-node-running');
import {
    expect_reject,
    linearSuite,
    createGroupInput,
}					from '../utils.js';


const delay				= (n) => new Promise(f => setTimeout(f, n));
const __filename			= new URL(import.meta.url).pathname;
const __dirname				= path.dirname( __filename );
const TEST_DNA_PATH			= path.join( __dirname, "../minimal_dna.dna" );

const DNA_NAME				= "test_dna";
const COOP_ZOME				= "coop_content_csr";

let client, installations;

describe("Minimal (external) DNA", function () {
    const holochain			= new Holochain({
	"timeout": 60_000,
	"default_stdout_loggers": log.level_rank > 3,
    });

    before(async function () {
	this.timeout( 300_000 );

	installations                   = await holochain.install([
	    "alice",
	    "bobby",
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
    });

    describe("Group", function () {
	linearSuite( "Basic", basic_tests );
	linearSuite( "Error", error_tests );
    });

    after(async () => {
	await holochain.destroy();
    });

});


let alice_client;
let bobby_client;

let alice_coop_content;
let bobby_coop_content;

let group;
let c1_addr				= new ExternalHash( crypto.randomBytes(32) );
let c1a_addr				= new ExternalHash( crypto.randomBytes(32) );

function basic_tests () {
    before(async function () {
	this.timeout( 30_000 );

        {
	    const auth			= installations.alice.test.auth;
	    alice_client		= await client.app( auth.token, "test-alice" );

            alice_coop_content          = alice_client.createZomeInterface(
                DNA_NAME, "coop_content_csr", CoopContentZomelet
            ).functions;
        }

        {
	    const auth			= installations.bobby.test.auth;
	    bobby_client		= await client.app( auth.token, "test-bobby" );

            bobby_coop_content          = bobby_client.createZomeInterface(
                DNA_NAME, "coop_content_csr", CoopContentZomelet
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
    });


    it("should create group via alice (A1)", async function () {
	const group_input		= createGroupInput(
	    [ alice_client.agent_id ],
	    bobby_client.agent_id,
	);
	group				= await alice_coop_content.create_group( group_input );
	log.debug( json.debug( group ) );
    });

    it("should update group", async function () {
	group.members			= [];

        group                           = await alice_coop_content.update_group({
	    "base": group.$action,
	    "entry": group,
	});
	log.debug( json.debug( group ) );
    });

    it("should get group", async function () {
        group				= await alice_coop_content.get_group( group.$id );
	log.debug( json.debug( group ) );
    });

    it("should create content link", async function () {
        await alice_coop_content.create_content_link({
	    "group_id": group.$id,
	    "content_target": c1_addr,
	});
    });

    it("should get all group content", async function () {
        const latest			= await alice_coop_content.get_group_content_latest({
	    "group_id": group.$id,
	    "content_id": c1_addr,
	});
	log.debug("Latest address for C1: %s", latest );

	expect( latest			).to.deep.equal( c1_addr );
    });

    it("should create content update link", async function () {
        await alice_coop_content.create_content_update_link( {
	    "group_id": group.$id,
	    "content_id": c1_addr,
	    "content_prev": c1_addr,
	    "content_next": c1a_addr,
	});
    });

    it("should get all group content", async function () {
        const latest			= await alice_coop_content.get_group_content_latest({
	    "group_id": group.$id,
	    "content_id": c1_addr,
	});
	log.debug("Latest address for C1: %s", latest );

	expect( latest			).to.deep.equal( c1a_addr );
    });

}

function error_tests () {

    it("should fail full trace because external hash does not have an action", async function () {
	await expect_reject( async () => {
            await alice_coop_content.get_group_content_latest({
		"group_id": group.$id,
		"content_id": c1_addr,
		"full_trace": true,
	    });
	}, "will not have a corresponding action" );
    });

}
