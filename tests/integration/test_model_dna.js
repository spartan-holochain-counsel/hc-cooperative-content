import { Logger }			from '@whi/weblogger';
const log				= new Logger("test-model-dna", process.env.LOG_LEVEL );

import fs				from 'node:fs';
import path				from 'path';
import crypto				from 'crypto';
import { expect }			from 'chai';
import { AgentPubKey, HoloHash,
	 ActionHash, EntryHash }	from '@whi/holo-hash';
import HolochainBackdrop		from '@whi/holochain-backdrop';
const { Holochain }			= HolochainBackdrop;

// const why				= require('why-is-node-running');
import { expect_reject }		from '../utils.js';

const delay				= (n) => new Promise(f => setTimeout(f, n));
const __filename			= new URL(import.meta.url).pathname;
const __dirname				= path.dirname( __filename );
const TEST_DNA_PATH			= path.join( __dirname, "../model_dna.dna" );

const clients				= {};
const DNA_NAME				= "test_dna";
const MAIN_ZOME				= "basic_usage";


function createGroupInput ( overrides ) {
    return Object.assign({
	"admins": [ new AgentPubKey( crypto.randomBytes(32) ) ],
	"members": [ new AgentPubKey( crypto.randomBytes(32) ) ],

	"published_at":			Date.now(),
	"last_updated":			Date.now(),
	"metadata":			{},
    }, overrides );
};


let group_1;

function group_tests () {

    it("should create group profile", async function () {
	this.timeout( 30_000 );

	const group_id = group_1	= await clients.alice.call( DNA_NAME, MAIN_ZOME, "create_group", createGroupInput() );

	log.debug("Group ID: %s", group_id );
	// log.debug( json.debug( group ) );

	// expect( group_id		).to.be.a("ActionHash");
	expect( group_id		).to.be.a("Uint8Array");
	expect( group_id		).to.have.length( 39 );
    });

}

function errors_tests () {
}

describe("Model DNA", () => {
    const holochain			= new Holochain({
	"timeout": 60_000,
	"default_stdout_loggers": process.env.LOG_LEVEL === "trace",
    });

    before(async function () {
	this.timeout( 60_000 );

	const actors			= await holochain.backdrop({
	    "test_happ": {
		[DNA_NAME]:	TEST_DNA_PATH,
	    },
	}, {
	    "actors": [ "alice" ],
	});

	for ( let name in actors ) {
	    for ( let app_prefix in actors[ name ] ) {
		log.info("Upgrade client for %s => %s", name, app_prefix );
		const client		= clients[ name ]	= actors[ name ][ app_prefix ].client;
	    }
	}

	// Must call whoami on each cell to ensure that init has finished.
	{
	    let whoami			= await clients.alice.call( DNA_NAME, MAIN_ZOME, "whoami", null, 30_000 );
	    log.normal("Alice whoami: %s", String(new HoloHash( whoami.agent_initial_pubkey )) );
	}
    });

    describe("Group", group_tests.bind( this, holochain ) );
    describe("Errors", errors_tests.bind( this, holochain ) );

    after(async () => {
	await holochain.destroy();
    });

});
