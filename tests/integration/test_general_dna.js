import { Logger }			from '@whi/weblogger';
const log				= new Logger("test-general-dna", process.env.LOG_LEVEL );

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

// const why				= require('why-is-node-running');
import {
    expect_reject,
    linearSuite,
    createGroupInput,
    createContentInput,
}					from '../utils.js';


const delay				= (n) => new Promise(f => setTimeout(f, n));
const __filename			= new URL(import.meta.url).pathname;
const __dirname				= path.dirname( __filename );
const TEST_DNA_PATH			= path.join( __dirname, "../general_dna.dna" );

const DNA_NAME				= "test_dna";

const GEN_ZOME				= "general_csr";
const COOP_ZOME				= "coop_content_csr";

let client, installations;

describe("General DNA", function () {
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
    });

    after(async () => {
	await holochain.destroy();
    });

});


let alice_client;
let bobby_client;

let alice_coop_content;
let bobby_coop_content;

let group, g1_addr;

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

}
