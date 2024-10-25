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
    async get_all_group_content_targets ( input ) {
	const result			= await this.call( input );

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
    async add_member ( input ) {
        const add_agent                 = new AgentPubKey( input.agent );
	const group                     = await this.functions.get_group( input.group_id );

        return await this.functions.update_group({
	    "base": group.$action,
	    "entry": Object.assign({}, group, {
		"members": [
                    ...group.members,
                    add_agent,
                ],
	    }),
        });
    },
    async remove_member ( input ) {
        const remove_agent              = new AgentPubKey( input.agent );
	const group                     = await this.functions.get_group( input.group_id );
        const new_members_list          = group.members.filter( agent => String(agent) !== String(remove_agent) );

        if ( new_members_list.length === group.members.length )
            throw new Error(`No members were removed from group; '${remove_agent}' not in current members: ${group.members.map( agent => String(agent) ).join(", ")}`);

        return await this.functions.update_group({
	    "base": group.$action,
	    "entry": Object.assign({}, group, {
		"members": new_members_list,
	    }),
        });
    },
    async add_admin ( input ) {
        const add_agent                 = new AgentPubKey( input.agent );
	const group                     = await this.functions.get_group( input.group_id );

        return await this.functions.update_group({
	    "base": group.$action,
	    "entry": Object.assign({}, group, {
		"admins": [
                    ...group.admins,
                    add_agent,
                ],
	    }),
        });
    },
    async remove_admin ( input ) {
        const remove_agent              = new AgentPubKey( input.agent );
	const group                     = await this.functions.get_group( input.group_id );
        const new_admins_list          = group.admins.filter( agent => String(agent) !== String(remove_agent) );

        if ( new_admins_list.length === group.admins.length )
            throw new Error(`No admins were removed from group; '${remove_agent}' not in current admins: ${group.admins.map( agent => String(agent) ).join(", ")}`);

        if ( new_admins_list.length === 0 )
            throw new Error(`There must be at least 1 admin`);

        return await this.functions.update_group({
	    "base": group.$action,
	    "entry": Object.assign({}, group, {
		"admins": new_admins_list,
	    }),
        });
    },
});


export default {
    CoopContentZomelet,
}
