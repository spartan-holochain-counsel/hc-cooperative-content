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
    async get_my_invites ( input ) {
	const result			= await this.call( input );

	return result.map( ([link, group]) => {
            return {
                "link":     link,
                "group":    new Group( group, this ),
            };
        });
    },
    async get_my_groups ( input ) {
	const result			= await this.call( input );

	return result.map( group => new Group( group, this ) );
    },
    async accept_group_invite ( input ) {
	const result			= await this.call( input );

	return new ActionHash( result );
    },
    async reject_group_invite ( input ) {
	const result			= await this.call( input );

	return new ActionHash( result );
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
    async remove_group_links ( input ) {
	const result			= await this.call( input );

	return result.map( hash => new ActionHash( hash ) );
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
    async purge_old_groups () {
	const groups			= await this.functions.get_my_groups();
        const removed_groups            = [];

	for ( let group of groups ) {
            if ( !group.isContributor( this.cell.client.agent_id ) ) {
                await this.functions.remove_group_links( group.$id );
                removed_groups.push( group );
            }
        }

        return removed_groups;
    },
    async accept_invitation_to_group ( group_id ) {
        const my_invites		= await this.functions.get_my_invites();
        const accepted_invites          = [];

        for ( let invite of my_invites ) {
            if ( String(invite.group.$id) == String(new ActionHash(group_id)) ) {
                await this.functions.accept_group_invite( invite.link.create_link_hash );
                accepted_invites.push( invite );
            }
        }

        if ( accepted_invites.length === 0 )
            throw new Error(`No invites were accepted`);

	return accepted_invites;
    },
});


export default {
    CoopContentZomelet,
}
