import {
    AnyDhtHash, AnyLinkableHash,
    AgentPubKey,
    ActionHash, EntryHash,
}					from '@spartan-hc/holo-hash'; // approx. 11kb
import {
    Zomelet,
}					from '@spartan-hc/zomelets'; // approx. 7kb
import {
    GroupEntry,
    Group,
}					from './types.js';


export const CoopContentZomelet		= new Zomelet({
    "whoami": {
	output ( response ) {
	    // Struct - https://docs.rs/hdk/*/hdk/prelude/struct.AgentInfo.html
	    return {
		"pubkey": {
		    "initial":		new AgentPubKey( response.agent_initial_pubkey ),
		    "latest":		new AgentPubKey( response.agent_latest_pubkey ),
		},
		"chain_head": {
		    "action":		new ActionHash( response.chain_head[0] ),
		    "sequence":		response.chain_head[1],
		    "timestamp":	response.chain_head[2],
		},
	    };
	},
    },

    //
    // Groups
    //
    async create_group ( input ) {
	const result			= await this.call( input );

	return new Group( result, this );
    },
    async get_group ( input ) {
	const result			= await this.call( input );

	return new Group( result, this );
    },
    async update_group ( input ) {
	const result			= await this.call( input );

	return new Group( result, this );
    },


    //
    // Links
    //
    async create_content_link ( input ) {
	const result			= await this.call( input );

	return new ActionHash( result );
    },
    async create_content_update_link ( input ) {
	const result			= await this.call( input );

	return new ActionHash( result );
    },
    async get_group_content_latest ( input ) {
	const result			= await this.call( input );

	return new AnyLinkableHash( result );
    },
    async get_group_content_evolutions ( input ) {
	const result			= await this.call( input );

	return result.map( hash => new AnyLinkableHash( hash ) );
    },
    async get_all_group_content_targets ( group_id ) {
	const result			= await this.call({
            "group_id":     new ActionHash( group_id ),
            "full_trace":   false,
        });

	return result.map( ([id_addr, latest_addr]) => {
            return [
                new AnyLinkableHash( id_addr ),
                new AnyLinkableHash( latest_addr ),
            ];
        });
    },


    //
    // Virtual functions
    //
    async new_group ( input ) {
	return await this.functions.create_group( input );
    },
});


export default {
    CoopContentZomelet,
}
