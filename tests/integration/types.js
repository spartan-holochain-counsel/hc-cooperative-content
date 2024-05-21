import {
    AgentPubKey, AnyLinkableHash,
    ActionHash, EntryHash,
}					from '@spartan-hc/holo-hash';
import {
    intoStruct,
    OptionType, VecType, MapType,
}					from '@whi/into-struct';
import {
    Zomelet,
}					from '@spartan-hc/zomelets'; // approx. 7kb


export const EntryCreationActionStruct = {
    "type":			String,
    "author":			AgentPubKey,
    "timestamp":		Number,
    "action_seq":		Number,
    "prev_action":		ActionHash,
    "original_action_address":	OptionType( ActionHash ),
    "original_entry_address":	OptionType( EntryHash ),
    "entry_type": {
	"App": {
	    "entry_index":	Number,
	    "zome_index":	Number,
	    "visibility":	"Public",
	},
    },
    "entry_hash":		EntryHash,
    "weight": {
	"bucket_id":		Number,
	"units":		Number,
	"rate_bytes":		Number,
    },
};

export const GroupStruct = {
    "admins":			VecType( AgentPubKey ),
    "members":			VecType( AgentPubKey ),

    "published_at":		Number,
    "last_updated":		Number,
    "metadata":			{},
};

export function Group ( entry ) {
    return intoStruct( entry, GroupStruct );
}

export const ContentStruct = {
    "text":			String,
    "group_ref":		{
	"id": ActionHash,
	"rev": ActionHash,
    },

    "published_at":		Number,
    "last_updated":		Number,
};

export function Content ( entry ) {
    return intoStruct( entry, ContentStruct );
}

export const CommentStruct = {
    "text":			String,
    "parent_comment":		OptionType( ActionHash ),
    "group_ref":		{
	"id": ActionHash,
	"rev": ActionHash,
    },
};

export function Comment ( entry ) {
    return intoStruct( entry, CommentStruct );
}


const functions				= {
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

    async get_group ( input ) {
	const result			= await this.call( input );

	return Group( result );
    },

    async create_content ( input ) {
	const result			= await this.call( input );

	return new ActionHash( result );
    },

    async create_comment ( input ) {
	const result			= await this.call( input );

	return new ActionHash( result );
    },

    async get_group_content ( input ) {
	const result			= await this.call({
	    "group_id": input.group_id,
	    "content_type": input.content_type,
	    "content_base": input.content_base,
	});

	return result.map( ([ctx, content]) => {
	    if ( content.type === "content" )
		content			= Content( content );
	    else if ( content.type === "comment" )
		content			= Comment( content );

	    return [
		[
		    new AnyLinkableHash( ctx[0] ),
		    new AnyLinkableHash( ctx[1] ),
		],
		content,
	    ];
	});
    },
};

export const BasicUsageCSRZomelet	= new Zomelet({
    functions,
});


export default {
    EntryCreationActionStruct,

    GroupStruct,
    Group,

    ContentStruct,
    Content,

    CommentStruct,
    Comment,

    BasicUsageCSRZomelet,
};
