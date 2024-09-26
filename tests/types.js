import {
    AnyDhtHash, AnyLinkableHash,
    AgentPubKey,
    ActionHash, EntryHash,
}					from '@spartan-hc/holo-hash'; // approx. 11kb
import {
    Zomelet,
}					from '@spartan-hc/zomelets'; // approx. 7kb
import {
    ScopedEntity,
}					from '@spartan-hc/entities';
import {
    intoStruct,
    OptionType, VecType,
}					from '@whi/into-struct';


//
// Types & Structs
//
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

export const ContentStruct = {
    "text":			String,
    "group_ref":		{
	"id": ActionHash,
	"rev": ActionHash,
    },

    "published_at":		Number,
    "last_updated":		Number,
};

export function ContentEntry ( entry ) {
    return intoStruct( entry, ContentStruct );
}

export class Content extends ScopedEntity {
    static STRUCT		= ContentStruct;
}



export const CommentStruct = {
    "text":			String,
    "parent_comment":		OptionType( ActionHash ),
    "group_ref":		{
	"id": ActionHash,
	"rev": ActionHash,
    },
};

export function CommentEntry ( entry ) {
    return intoStruct( entry, CommentStruct );
}


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



//
// Zomelets
//
export const BasicUsageZomelet	        = new Zomelet({
    //
    // Content
    //
    async create_content ( input ) {
	const result			= await this.call( input );

	return new ActionHash( result );
    },
    async get_content ( input ) {
	const result			= await this.call( input );

	return ContentEntry( result );
    },
    async update_content ( input ) {
	const result			= await this.call( input );

	return new ActionHash( result );
    },

    //
    // Comment
    //
    async create_comment ( input ) {
	const result			= await this.call( input );

	return new ActionHash( result );
    },

    //
    // Group
    //
    async get_group ( input ) {
	const result			= await this.call( input );

	return Group( result );
    },
    async get_group_content ( input ) {
	const result			= await this.call( input );

	return result.map( ([[origin_addr, latest_addr], data]) => {
	    if ( data.type === "content" )
		data			= ContentEntry( data );
	    else if ( data.type === "comment" )
		data			= CommentEntry( data );

            return [
                [
                    new ActionHash(origin_addr),
                    new ActionHash(latest_addr),
                ],
                data,
            ]
        });
    },
});


export default {
    BasicUsageZomelet,
}
